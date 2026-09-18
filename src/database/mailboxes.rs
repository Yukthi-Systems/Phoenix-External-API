use crate::models::mailbox::MailBoxInfo;
use deadpool_postgres::Pool as PgPool;
use crate::models::errors::AppError;
use crate::models::ListResponse;
use uuid::Uuid;



pub async fn list_domain_mailboxes(db_pool: &PgPool, domain_name: &str, limit: i64, offset: i64) -> Result<ListResponse<MailBoxInfo>, AppError> {
    let client = db_pool.get().await?;

    let rows = client
        .query(
            r#"
            SELECT
                email,
                domain_name,
                is_enabled,
                forwarding_policy_id,
                distribution_policy_id,
                general_policy_id,
                quota_allocated::DOUBLE PRECISION,
                quota_utilized_bytes,
                total_messages_count,
                COUNT(*) OVER() AS total_count
            FROM mailboxes
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
        MailBoxInfo::from,
    ))
}


pub async fn get_org_mailbox(db_pool: &PgPool, org_id: &Uuid, email_id: &str) -> Result<Option<MailBoxInfo>, AppError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT
                m.email,
                m.domain_name,
                m.is_enabled,
                m.forwarding_policy_id,
                m.distribution_policy_id,
                m.general_policy_id,
                m.quota_allocated::DOUBLE PRECISION,
                m.quota_utilized_bytes,
                m.total_messages_count
            FROM mailboxes AS m
            INNER JOIN domains AS d
                ON d.domain_name = m.domain_name
            WHERE d.managed_by = $1
            AND m.email = $2
            "#,
            &[org_id, &email_id],
        )
        .await?;

    Ok(row.map(MailBoxInfo::from))
}


pub async fn update_mailbox_info(
    db_pool: &PgPool,
    email: &str,
    is_enabled: bool,
    forwarding_policy_id: Option<Uuid>,
    distribution_policy_id: Option<Uuid>,
    general_policy_id: Option<Uuid>,
) -> Result<u64, AppError> {
    let client = db_pool.get().await?;

    let result = client
        .execute(
            r#"
            UPDATE mailboxes
            SET
                is_enabled = $1,
                forwarding_policy_id = $2,
                distribution_policy_id = $3,
                general_policy_id = $4
            WHERE email = $5
            "#,
            &[&is_enabled, &forwarding_policy_id, &distribution_policy_id, &general_policy_id, &email],
        )
        .await?;

    Ok(result)
}


pub async fn create_new_mailbox(
    db_pool: &PgPool,
    email: &str,
    domain_name: &str,
    org_id: &Uuid,
    forwarding_policy_id: Option<Uuid>,
    distribution_policy_id: Option<Uuid>,
    general_policy_id: Option<Uuid>,
    quota_allocated: f64,
) -> Result<u64, AppError> {
    let default_server_id: Uuid = Uuid::parse_str("0d069ed3-daaa-534f-9d2d-bd6c34503b84").unwrap();

    let mut client = db_pool.get().await?;

    let txn = client.transaction().await?;

    // First create a new mailbox in the mailboxes table
    txn.execute(
        r#"
        INSERT INTO mailboxes (
            email,
            domain_name,
            is_enabled,
            quota_allocated,
            quota_utilized_bytes,
            server_id,
            general_policy_id,
            forwarding_policy_id,
            distribution_policy_id
        )
        VALUES ($1, $2, $3, $4::DOUBLE PRECISION, $5, $6, $7, $8, $9)
        "#,
        &[
            &email,
            &domain_name,
            &false,
            &quota_allocated,
            &0i64,
            &default_server_id,
            &general_policy_id,
            &forwarding_policy_id,
            &distribution_policy_id,
        ],
    )
    .await?;

    // Then update the organization's quota utilization
    let result = txn
        .execute(
            r#"
            UPDATE organizations SET quota_utilized = quota_utilized + $1::DOUBLE PRECISION WHERE organization_id = $2
            "#,
            &[&quota_allocated, &org_id],
        )
        .await?;

    txn.commit().await?;

    Ok(result)
}
