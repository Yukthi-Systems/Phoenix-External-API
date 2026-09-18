use deadpool_postgres::{
    PoolError as PgError,
    Pool as PgPool
};
use crate::models::api_key::ApiSession;

pub mod departments;
pub mod identities;
pub mod mailboxes;
pub mod domains;
pub mod orgs;


// DB working state Check
pub async fn health_check(db_pool: &PgPool) -> Result<(), PgError> {
    // Simple query to check if the database is responsive
    let client = db_pool.get().await?;
    let _ = client.query("SELECT 1", &[]).await?;
    Ok(())
}


/// Retrieves information about an API key from the database.
pub async fn get_api_key_info(db_pool: &PgPool, api_key: &uuid::Uuid) -> Result<Option<ApiSession>, PgError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT
                api_key,
                organization_id,
                permissions
            FROM api_keys
            WHERE api_key = $1
            AND is_active = TRUE
            -- TODO: After adding that field in DB uncomment it here
            -- TODO: Check if org is active or not too
            -- AND expired_at > CURRENT_TIMESTAMP
            "#,
            &[&api_key],
        )
        .await?;

    Ok(row.map(ApiSession::from))
}
