use deadpool_postgres::Pool as PgPool;
use crate::models::domain::DomainInfo;
use crate::models::errors::AppError;
use crate::models::ListResponse;
use uuid::Uuid;



pub async fn list_org_domains(db_pool: &PgPool, org_id: &Uuid, limit: i64, offset: i64) -> Result<ListResponse<DomainInfo>, AppError> {
    let client = db_pool.get().await?;

    let rows = client
        .query(
            r#"
            SELECT
                domain_name,
                anti_phishing_secret_code,
                details,
                is_active,
                is_dns_txt_verified,
                dns_txt_verification_key,
                spam_destination,
                spam_destination_properties,
                filter_policy_id,
                attachment_policy_id,
                catch_all,
                catch_all_forward_to_email,
                is_hybrid,
                connector_properties,
                max_password_age,
                max_password_age_properties,
                session_timeout,
                disclaimer_id,
                caution_id,
                created_at,
                COUNT(*) OVER() AS total_count
            FROM domains
            WHERE managed_by = $1
            ORDER BY domain_name
            LIMIT $2
            OFFSET $3
            "#,
            &[org_id, &limit, &offset],
        )
        .await?;

    Ok(ListResponse::from_rows(
        rows,
        limit,
        offset,
        DomainInfo::from,
    ))
}


pub async fn get_domain_details(db_pool: &PgPool, org_id: &Uuid, domain_name: &str) -> Result<Option<DomainInfo>, AppError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT
                domain_name,
                anti_phishing_secret_code,
                details,
                is_active,
                is_dns_txt_verified,
                dns_txt_verification_key,
                spam_destination,
                spam_destination_properties,
                filter_policy_id,
                attachment_policy_id,
                catch_all,
                catch_all_forward_to_email,
                is_hybrid,
                connector_properties,
                max_password_age,
                max_password_age_properties,
                session_timeout,
                disclaimer_id,
                caution_id,
                created_at
            FROM domains
            WHERE managed_by = $1
            AND domain_name = $2
            "#,
            &[org_id, &domain_name],
        )
        .await?;

    Ok(row.map(DomainInfo::from))
}


pub async fn update_domain_by_name(
    db_pool: &PgPool,
    org_id: &Uuid,
    domain_name: &str,
    details: &serde_json::Value,
    is_active: bool,
    filter_policy_id: Option<Uuid>,
    attachment_policy_id: Option<Uuid>,
    disclaimer_id: Option<Uuid>,
    caution_id: Option<Uuid>,
) -> Result<u64, AppError> {
    let client = db_pool.get().await?;

    let result = client
        .execute(
            r#"
            UPDATE domains
            SET details = $1,
                is_active = $2,
                filter_policy_id = $3,
                attachment_policy_id = $4,
                disclaimer_id = $5,
                caution_id = $6
            WHERE managed_by = $7
            AND domain_name = $8
            "#,
            &[&details, &is_active, &filter_policy_id, &attachment_policy_id, &disclaimer_id, &caution_id, org_id, &domain_name],
        )
        .await?;

    Ok(result)
}


pub async fn get_available_domains(db_pool: &PgPool, org_id: &Uuid) -> Result<Vec<String>, AppError> {
    let client = db_pool.get().await?;

    let rows = client
        .query(
            r#"
            SELECT domain_name
            FROM domains
            WHERE managed_by = $1
            AND is_active = true
            AND is_dns_txt_verified = true
            "#,
            &[org_id],
        )
        .await?;

    Ok(rows.iter().map(|row| row.get("domain_name")).collect())
}
