use crate::models::departments::DepartmentInfo;
use deadpool_postgres::Pool as PgPool;
use crate::models::errors::AppError;
use crate::models::ListResponse;
use uuid::Uuid;



pub async fn list_org_departments(db_pool: &PgPool, org_id: &Uuid, limit: i64, offset: i64) -> Result<ListResponse<DepartmentInfo>, AppError> {
    let client = db_pool.get().await?;

    let rows = client
        .query(
            r#"
            SELECT
                department_id,
                department_name,
                details,
                created_at,
                updated_at,
                COUNT(*) OVER() AS total_count
            FROM departments
            WHERE organization_id = $1
            ORDER BY department_name
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
        DepartmentInfo::from,
    ))
}


pub async fn get_department_details(db_pool: &PgPool, org_id: &Uuid, department_id: &Uuid) -> Result<Option<DepartmentInfo>, AppError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT
                department_id,
                department_name,
                details,
                created_at,
                updated_at
            FROM departments
            WHERE organization_id = $1
            AND department_id = $2
            "#,
            &[org_id, department_id],
        )
        .await?;

    Ok(row.map(DepartmentInfo::from))
}
