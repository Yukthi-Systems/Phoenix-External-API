use tokio_postgres::row::Row;
use serde::Serialize;
use uuid::Uuid;



#[derive(Serialize)]
pub struct OrgInfo {
    pub organization_id: Uuid,
    pub organization_name: String,
    pub organization_info: serde_json::Value,
    pub is_active: bool,

    pub allocated_email_identities: i32,
    pub utilized_email_identities: i32,

    pub quota_allocated: f64,
    pub quota_utilized: f64,

    pub chat_service_enabled: bool,
    pub email_service_enabled: bool,
    pub file_service_enabled: bool,

    pub created_at: chrono::DateTime<chrono::Utc>,
}


// ------- Implementations ------- //


impl From<Row> for OrgInfo {
    fn from(row: Row) -> Self {
        OrgInfo {
            organization_id: row.get("organization_id"),
            organization_name: row.get("organization_name"),
            organization_info: row.get("organization_info"),
            is_active: row.get("is_active"),

            allocated_email_identities: row.get("allocated_email_identities"),
            utilized_email_identities: row.get("utilized_email_identities"),

            quota_allocated: row.get("quota_allocated"),
            quota_utilized: row.get("quota_utilized"),

            chat_service_enabled: row.get("chat_service_enabled"),
            email_service_enabled: row.get("email_service_enabled"),
            file_service_enabled: row.get("file_service_enabled"),

            created_at: row.get("created_at"),
        }
    }
}
