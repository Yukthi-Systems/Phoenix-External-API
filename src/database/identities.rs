use crate::models::identity::IdentityInfo;
use deadpool_postgres::Pool as PgPool;
use crate::models::errors::AppError;
use crate::models::ListResponse;
use uuid::Uuid;



pub async fn list_domain_identities(db_pool: &PgPool, domain_name: &str, limit: i64, offset: i64) -> Result<ListResponse<IdentityInfo>, AppError> {
    let client = db_pool.get().await?;

    let rows = client
        .query(
            r#"
            SELECT
                email,
                domain_name,
                first_name,
                last_name,
                primary_phone,
                secondary_email,
                is_app_2fa_enabled,
                is_sms_2fa_enabled,
                is_email_2fa_enabled,
                restriction_policy_id,
                department_id,
                is_password_expired,
                is_enabled,
                password_updated_at,
                created_at,
                updated_at,
                COUNT(*) OVER() AS total_count
            FROM email_identities
            WHERE domain_name = $1
            ORDER BY email
            LIMIT $2
            OFFSET $3
            "#,
            &[&domain_name, &limit, &offset],
        )
        .await?;

    Ok(ListResponse::from_rows(
        rows,
        limit,
        offset,
        IdentityInfo::from,
    ))
}


pub async fn get_org_identity(db_pool: &PgPool, org_id: &Uuid, email_id: &str) -> Result<Option<IdentityInfo>, AppError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT
                ei.email,
                ei.domain_name,
                ei.first_name,
                ei.last_name,
                ei.primary_phone,
                ei.secondary_email,
                ei.is_app_2fa_enabled,
                ei.is_sms_2fa_enabled,
                ei.is_email_2fa_enabled,
                ei.restriction_policy_id,
                ei.department_id,
                ei.is_password_expired,
                ei.is_enabled,
                ei.password_updated_at,
                ei.created_at,
                ei.updated_at
            FROM email_identities AS ei
            INNER JOIN domains AS d
                ON d.domain_name = ei.domain_name
            WHERE d.managed_by = $1
            AND ei.email = $2
            "#,
            &[org_id, &email_id],
        )
        .await?;

    Ok(row.map(IdentityInfo::from))
}


pub async fn update_identity_by_email(
    db_pool: &PgPool,
    email: &str,
    domain_name: &str,
    first_name: &str,
    last_name: &Option<String>,
    primary_phone: &str,
    secondary_email: &Option<String>,
    is_app_2fa_enabled: &bool,
    is_sms_2fa_enabled: &bool,
    is_email_2fa_enabled: &bool,
    restriction_policy_id: &Option<Uuid>,
    department_id: &Option<Uuid>,
    is_enabled: &bool,
) -> Result<u64, AppError> {
    let client = db_pool.get().await?;

    let result = client
        .execute(
            r#"
            UPDATE email_identities
            SET
                first_name = $1,
                last_name = $2,
                primary_phone = $3,
                secondary_email = $4,
                is_app_2fa_enabled = $5,
                is_sms_2fa_enabled = $6,
                is_email_2fa_enabled = $7,
                restriction_policy_id = $8,
                department_id = $9,
                is_enabled = $10,
                updated_at = NOW()
            WHERE email = $11
            AND domain_name = $12
            "#,
            &[
                &first_name,
                last_name,
                &primary_phone,
                secondary_email,
                is_app_2fa_enabled,
                is_sms_2fa_enabled,
                is_email_2fa_enabled,
                restriction_policy_id,
                department_id,
                is_enabled,
                &email,
                &domain_name,
            ],
        )
        .await?;

    Ok(result)
}


async fn get_associated_services_for_identity(db_pool: &PgPool, email: &str) -> Result<(bool, bool, bool), AppError> {
    let client = db_pool.get().await?;

    let service_usage = client
        .query_one(
            r#"
            SELECT
                EXISTS (
                    SELECT 1 FROM mailboxes WHERE email = $1
                ) AS has_mailbox,
                EXISTS (
                    SELECT 1 FROM chat_users WHERE email = $1
                ) AS has_chat,
                EXISTS (
                    SELECT 1 FROM file_users WHERE email = $1
                ) AS has_file
            "#,
            &[&email],
        )
        .await?;

    let has_mailbox: bool = service_usage.get("has_mailbox");
    let has_chat: bool = service_usage.get("has_chat");
    let has_file: bool = service_usage.get("has_file");

    Ok((has_mailbox, has_chat, has_file))
}


pub async fn delete_identity_by_email(db_pool: &PgPool, org_id: &Uuid, email_id: &str) -> Result<u64, AppError> {
    // Make sure there are no associated services for the identity before deleting it
    let (has_mailbox, has_chat, has_file) = get_associated_services_for_identity(db_pool, email_id).await?;
    if has_mailbox || has_chat || has_file {
        return Err(AppError::Conflict("Identity has associated services and cannot be deleted".into()));
    }

    let mut client = db_pool.get().await?;
    let txn = client.transaction().await?;

    let result = txn
        .execute(
            r#"
            DELETE FROM email_identities
            USING domains AS d
            WHERE d.domain_name = email_identities.domain_name
            AND d.managed_by = $1
            AND email_identities.email = $2
            "#,
            &[org_id, &email_id],
        )
        .await?;

    if result != 0 {
        // Update the ID count to org
        txn
            .execute(
                r#"
                UPDATE organizations
                SET utilized_email_identities = utilized_email_identities - 1
                WHERE organization_id = $1
                "#,
                &[org_id],
            )
            .await?;
    }

    txn.commit().await?;

    Ok(result)
}


pub async fn update_identity_password_by_email(db_pool: &PgPool, email: &str, org_id: &Uuid, bcrypt_hash: &str, ssha1_hash: &str) -> Result<u64, AppError> {
    let client = db_pool.get().await?;

    let result = client
        .execute(
            r#"
            UPDATE email_identities
            SET password_bcrypt = $1,
                password_hash_ssha1 = $2,
                updated_at = NOW(),
                password_updated_at = NOW(),
                is_password_expired = FALSE
            FROM domains AS d
            WHERE d.domain_name = email_identities.domain_name
            AND d.managed_by = $3
            AND email_identities.email = $4
            "#,
            &[&bcrypt_hash, &ssha1_hash, &org_id, &email],
        )
        .await?;

    Ok(result)
}


async fn get_org_identity_count(db_pool: &PgPool, org_id: &Uuid) -> Result<(i32, i32), AppError> {
    let client = db_pool.get().await?;

    let result = client
        .query_one(
            r#"
            SELECT allocated_email_identities, utilized_email_identities
            FROM organizations
            WHERE organization_id = $1
            "#,
            &[org_id],
        )
        .await?;

    let allocated = result.get::<_, i32>("allocated_email_identities");
    let utilized = result.get::<_, i32>("utilized_email_identities");

    Ok((allocated, utilized))
}


pub async fn create_new_identity(
    db_pool: &PgPool,
    org_id: &Uuid,
    email: &str,
    domain_name: &str,
    first_name: &str,
    last_name: &Option<String>,
    primary_phone: &str,
    secondary_email: &Option<String>,
    bcrypt_hash: &str,
    ssha1_hash: &str,
    is_app_2fa_enabled: bool,
    is_sms_2fa_enabled: bool,
    is_email_2fa_enabled: bool,
    restriction_policy_id: &Option<Uuid>,
    department_id: &Option<Uuid>,
    is_enabled: bool,
) -> Result<u64, AppError> {
    // Check if the org can create a new identity
    let (allocated, utilized) = get_org_identity_count(db_pool, org_id).await?;
    if allocated != -1 {    // If it is -1 then there is no limit on email identities (Unlimited)
        if utilized >= allocated {
            return Err(AppError::Forbidden("Organization has reached its limit for email identities".into()));
        }
    }

    let mut client = db_pool.get().await?;
    let txn = client.transaction().await?;

    // Insert the new identity into the email_identities table
    let result = txn
        .execute(
            r#"
            INSERT INTO email_identities (
                email,
                domain_name,
                first_name,
                last_name,
                primary_phone,
                secondary_email,
                password_bcrypt,
                password_hash_ssha1,
                is_app_2fa_enabled,
                is_sms_2fa_enabled,
                is_email_2fa_enabled,
                restriction_policy_id,
                department_id,
                is_enabled
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
            )
            "#,
            &[
                &email,
                &domain_name,
                &first_name,
                &last_name,
                &primary_phone,
                &secondary_email,
                &bcrypt_hash,
                &ssha1_hash,
                &is_app_2fa_enabled,
                &is_sms_2fa_enabled,
                &is_email_2fa_enabled,
                &restriction_policy_id,
                &department_id,
                &is_enabled,            ],
        )
        .await?;

    // Increment the utilized email identities count for the organization (if successful)
    if result > 0 {
        txn
            .execute(
                "UPDATE organizations SET utilized_email_identities = utilized_email_identities + 1 WHERE organization_id = $1",
                &[&org_id],
            )
            .await?;
    }

    txn.commit().await?;

    Ok(result)
}
