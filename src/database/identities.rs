use crate::models::identity::IdentityInfo;
use deadpool_postgres::Pool as PgPool;
use crate::models::errors::AppError;
use crate::models::ListResponse;
use uuid::Uuid;



// -- E-Mail ID's - Users
// CREATE TABLE email_identities (
//     email VARCHAR(254) PRIMARY KEY,
//     domain_name VARCHAR(254) NOT NULL REFERENCES domains(domain_name) ON DELETE CASCADE,

//     first_name TEXT NOT NULL,
//     last_name TEXT,
//     primary_phone VARCHAR(20) NOT NULL, -- Used for 2FA/recovery/notifications
//     secondary_email VARCHAR(254),   -- Used for 2FA/recovery/notifications

//     password_hash_ssha1 TEXT NOT NULL,
//     password_bcrypt TEXT NOT NULL,

//     is_app_2fa_enabled BOOLEAN DEFAULT FALSE NOT NULL,  -- Is app-based 2FA enabled
//     is_sms_2fa_enabled BOOLEAN DEFAULT FALSE NOT NULL,  -- Is SMS-based 2FA enabled
//     is_email_2fa_enabled BOOLEAN DEFAULT FALSE NOT NULL,  -- Is Email-based 2FA enabled
//     -- TODO: Add a TOTP and Backup Codes too

//     restriction_policy_id UUID REFERENCES restriction_policies(policy_id) ON DELETE SET NULL,
//     department_id UUID REFERENCES departments(department_id) ON DELETE SET NULL,

//     is_password_expired BOOLEAN DEFAULT FALSE NOT NULL,
//     is_enabled BOOLEAN DEFAULT TRUE NOT NULL,

//     password_updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,

//     created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
//     updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
// );

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
    let client = db_pool.get().await?;

    // Make sure there are no associated services for the identity before deleting it
    let (has_mailbox, has_chat, has_file) = get_associated_services_for_identity(db_pool, email_id).await?;
    if has_mailbox || has_chat || has_file {
        return Err(AppError::Conflict("Identity has associated services and cannot be deleted".into()));
    }

    let result = client
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
        client
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
