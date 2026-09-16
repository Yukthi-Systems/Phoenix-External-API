use deadpool_postgres::Pool as PgPool;
use crate::models::errors::AppError;
use crate::models::orgs::OrgInfo;
use uuid::Uuid;



pub async fn get_org_info(db_pool: &PgPool, org_id: &Uuid) -> Result<OrgInfo, AppError> {
    let client = db_pool.get().await?;

    let row = client
        .query_one(
            r#"
            SELECT
                organization_id,
                organization_name,
                organization_info,
                is_active,
                allocated_email_identities,
                utilized_email_identities,
                quota_allocated::DOUBLE PRECISION,
                quota_utilized::DOUBLE PRECISION,
                chat_service_enabled,
                email_service_enabled,
                file_service_enabled,
                created_at
            FROM organizations
            WHERE organization_id = $1
            AND is_active = TRUE
            "#,
            &[org_id],
        )
        .await?;

    Ok(OrgInfo::from(row))
}
