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


#[derive(Deserialize)]
pub struct MailBoxEditRequest {
    pub email: String,
    pub domain_name: String,
    pub is_enabled: bool,
    pub forwarding_policy_id: Option<Uuid>,
    pub distribution_policy_id: Option<Uuid>,
    pub general_policy_id: Option<Uuid>,
}


#[derive(Deserialize)]
pub struct MailBoxCreateRequest {
    pub email: String,
    pub domain_name: String,
    pub forwarding_policy_id: Option<Uuid>,
    pub distribution_policy_id: Option<Uuid>,
    pub general_policy_id: Option<Uuid>,
    pub quota_allocated: f64,
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


impl MailBoxEditRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.email.trim().is_empty() {
            return Err(AppError::BadRequest("Email cannot be empty".into()));
        }
        if self.domain_name.trim().is_empty() {
            return Err(AppError::BadRequest("Domain name cannot be empty".into()));
        }

        // Make sure the email is in a valid format and exactly matches the domain name
        let email_parts: Vec<&str> = self.email.split('@').collect();
        if email_parts.len() != 2 || email_parts[1] != self.domain_name {
            return Err(AppError::BadRequest("Email must match the domain name".into()));
        }
        Ok(())
    }
}


impl MailBoxCreateRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.email.trim().is_empty() {
            return Err(AppError::BadRequest("Email cannot be empty".into()));
        }
        if self.domain_name.trim().is_empty() {
            return Err(AppError::BadRequest("Domain name cannot be empty".into()));
        }

        // Make sure the email is in a valid format and exactly matches the domain name
        let email_parts: Vec<&str> = self.email.split('@').collect();
        if email_parts.len() != 2 || email_parts[1] != self.domain_name {
            return Err(AppError::BadRequest("Email must match the domain name".into()));
        }

        // Check if the quota allocated is a positive number
        if self.quota_allocated < 0.1 {
            return Err(AppError::BadRequest("Quota allocated must be a positive number greater than 0.1".into()));
        }

        Ok(())
    }
}
