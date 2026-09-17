use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use tokio_postgres::row::Row;
use uuid::Uuid;


type ChronoUtc = chrono::DateTime<chrono::Utc>;


#[derive(Serialize)]
pub struct DomainInfo {
    pub domain_name: String,
    pub anti_phishing_secret_code: String,
    pub details: JsonValue,

    pub is_active: bool,
    pub is_dns_txt_verified: bool,
    pub dns_txt_verification_key: String,

    pub spam_destination: String,
    pub spam_destination_properties: JsonValue,

    pub filter_policy_id: Option<Uuid>,
    pub attachment_policy_id: Option<Uuid>,

    pub catch_all: bool,
    pub catch_all_forward_to_email: Option<String>,

    pub is_hybrid: bool,
    pub connector_properties: JsonValue,

    pub max_password_age: i32,
    pub max_password_age_properties: JsonValue,
    pub session_timeout: i32,

    pub disclaimer_id: Option<Uuid>,
    pub caution_id: Option<Uuid>,

    pub created_at: ChronoUtc,
}


#[derive(Deserialize)]
pub struct DomainEditRequest {
    pub details: JsonValue,
    pub is_active: bool,

    pub filter_policy_id: Option<Uuid>,
    pub attachment_policy_id: Option<Uuid>,

    pub disclaimer_id: Option<Uuid>,
    pub caution_id: Option<Uuid>,
}


// ------- Implementations ------- //


impl From<Row> for DomainInfo {
    fn from(row: Row) -> Self {
        DomainInfo {
            domain_name: row.get("domain_name"),
            anti_phishing_secret_code: row.get("anti_phishing_secret_code"),
            details: row.get("details"),

            is_active: row.get("is_active"),
            is_dns_txt_verified: row.get("is_dns_txt_verified"),
            dns_txt_verification_key: row.get("dns_txt_verification_key"),

            spam_destination: row.get("spam_destination"),
            spam_destination_properties: row.get("spam_destination_properties"),

            filter_policy_id: row.get("filter_policy_id"),
            attachment_policy_id: row.get("attachment_policy_id"),

            catch_all: row.get("catch_all"),
            catch_all_forward_to_email: row.get("catch_all_forward_to_email"),

            is_hybrid: row.get("is_hybrid"),
            connector_properties: row.get("connector_properties"),

            max_password_age: row.get("max_password_age"),
            max_password_age_properties: row.get("max_password_age_properties"),
            session_timeout: row.get("session_timeout"),

            disclaimer_id: row.get("disclaimer_id"),
            caution_id: row.get("caution_id"),

            created_at: row.get("created_at"),
        }
    }
}
