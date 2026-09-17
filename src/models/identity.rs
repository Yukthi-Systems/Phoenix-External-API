use serde::{Serialize, Deserialize};
use tokio_postgres::row::Row;
use uuid::Uuid;


type ChronoUtc = chrono::DateTime<chrono::Utc>;


#[derive(Serialize)]
pub struct IdentityInfo {
    pub email: String,
    pub domain_name: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub primary_phone: String,
    pub secondary_email: Option<String>,

    pub is_app_2fa_enabled: bool,
    pub is_sms_2fa_enabled: bool,
    pub is_email_2fa_enabled: bool,

    pub restriction_policy_id: Option<Uuid>,
    pub department_id: Option<Uuid>,

    pub is_password_expired: bool,
    pub is_enabled: bool,

    pub password_updated_at: ChronoUtc,
    pub created_at: ChronoUtc,
    pub updated_at: ChronoUtc,
}


#[derive(Deserialize)]
pub struct IdentityEditRequest {
    pub email: String,
    pub domain_name: String,

    pub first_name: String,
    pub last_name: Option<String>,
    pub primary_phone: String,
    pub secondary_email: Option<String>,

    pub is_app_2fa_enabled: bool,
    pub is_sms_2fa_enabled: bool,
    pub is_email_2fa_enabled: bool,

    pub restriction_policy_id: Option<Uuid>,
    pub department_id: Option<Uuid>,

    pub is_enabled: bool,
}


// ------- Implementations ------- //


impl From<Row> for IdentityInfo {
    fn from(row: Row) -> Self {
        IdentityInfo {
            email: row.get("email"),
            domain_name: row.get("domain_name"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            primary_phone: row.get("primary_phone"),
            secondary_email: row.get("secondary_email"),

            is_app_2fa_enabled: row.get("is_app_2fa_enabled"),
            is_sms_2fa_enabled: row.get("is_sms_2fa_enabled"),
            is_email_2fa_enabled: row.get("is_email_2fa_enabled"),

            restriction_policy_id: row.get("restriction_policy_id"),
            department_id: row.get("department_id"),

            is_password_expired: row.get("is_password_expired"),
            is_enabled: row.get("is_enabled"),

            password_updated_at: row.get("password_updated_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
