use serde::{Serialize, Deserialize};
use tokio_postgres::row::Row;
use uuid::Uuid;


type ChronoUtc = chrono::DateTime<chrono::Utc>;


#[derive(Serialize)]
pub struct DepartmentInfo {
    pub department_id: Uuid,
    pub department_name: String,
    pub details: serde_json::Value,

    pub created_at: ChronoUtc,
    pub updated_at: ChronoUtc,
}


#[derive(Deserialize)]
pub struct DepartmentCreateRequest {
    pub department_name: String,
    pub details: serde_json::Value,
}


// ------- Implementations ------- //


impl From<Row> for DepartmentInfo {
    fn from(row: Row) -> Self {
        DepartmentInfo {
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            details: row.get("details"),

            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
