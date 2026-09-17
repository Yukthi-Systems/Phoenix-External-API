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
