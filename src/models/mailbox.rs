use serde::{Serialize, Deserialize};
use tokio_postgres::row::Row;
use super::errors::AppError;
use uuid::Uuid;



#[derive(Serialize)]
pub struct MailBoxInfo {
    pub email: String,
    pub domain_name: String,
    pub is_enabled: bool,

    pub forwarding_policy_id: Option<Uuid>,
    pub distribution_policy_id: Option<Uuid>,
    pub general_policy_id: Option<Uuid>,

    pub quota_allocated: f64,
    pub quota_utilized_bytes: i64,
    pub total_messages_count: i64,
}


// ------- Implementations ------- //


impl From<Row> for MailBoxInfo {
    fn from(row: Row) -> Self {
        MailBoxInfo {
            email: row.get("email"),
            domain_name: row.get("domain_name"),
            is_enabled: row.get("is_enabled"),
            forwarding_policy_id: row.get("forwarding_policy_id"),
            distribution_policy_id: row.get("distribution_policy_id"),
            general_policy_id: row.get("general_policy_id"),
            quota_allocated: row.get("quota_allocated"),
            quota_utilized_bytes: row.get("quota_utilized_bytes"),
            total_messages_count: row.get("total_messages_count"),
        }
    }
}
