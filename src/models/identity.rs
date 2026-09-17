use serde::{Serialize, Deserialize};
use tokio_postgres::row::Row;
use super::errors::AppError;
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


#[derive(Deserialize)]
pub struct CreateIdentityRequest {
    pub email_prefix: String,
    pub domain_name: String,

    pub first_name: String,
    pub last_name: Option<String>,
    pub primary_phone: String,
    pub secondary_email: Option<String>,

    pub encoded_password: String,

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


impl IdentityEditRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.email.trim().is_empty() {
            return Err(AppError::BadRequest("Email cannot be empty".into()));
        }
        if self.domain_name.trim().is_empty() {
            return Err(AppError::BadRequest("Domain name cannot be empty".into()));
        }
        if self.first_name.trim().is_empty() {
            return Err(AppError::BadRequest("First name cannot be empty".into()));
        }
        if self.primary_phone.trim().is_empty() {
            return Err(AppError::BadRequest("Primary phone cannot be empty".into()));
        }
        if self.email.chars().any(|c| c.is_uppercase()) {
            return Err(AppError::BadRequest("Email should not contain uppercase characters".into()));
        }
        if self.domain_name.chars().any(|c| c.is_uppercase()) {
            return Err(AppError::BadRequest("Domain name should not contain uppercase characters".into()));
        }

        // Check by splitting the email and checking the domain part
        if let Some(at_pos) = self.email.find('@') {
            let email_domain = &self.email[at_pos + 1..];
            if email_domain != self.domain_name {
                return Err(AppError::BadRequest("Email domain does not match the specified domain name".into()));
            }
        } else {
            return Err(AppError::BadRequest("Invalid email format".into()));
        }

        Ok(())
    }
}


impl CreateIdentityRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.encoded_password.trim().is_empty() {
            return Err(AppError::BadRequest("Encoded password cannot be empty".into()));
        }
        if self.email_prefix.trim().is_empty() {
            return Err(AppError::BadRequest("Email prefix cannot be empty".into()));
        }
        if self.domain_name.trim().is_empty() {
            return Err(AppError::BadRequest("Domain name cannot be empty".into()));
        }
        if self.first_name.trim().is_empty() {
            return Err(AppError::BadRequest("First name cannot be empty".into()));
        }
        if self.primary_phone.trim().is_empty() {
            return Err(AppError::BadRequest("Primary phone cannot be empty".into()));
        }
        if self.email_prefix.contains("@") {
            return Err(AppError::BadRequest("Email prefix should not contain '@'".into()));
        }
        if self.email_prefix.chars().any(|c| c.is_uppercase()) {
            return Err(AppError::BadRequest("Email prefix should not contain uppercase characters".into()));
        }
        if self.domain_name.chars().any(|c| c.is_uppercase()) {
            return Err(AppError::BadRequest("Domain name should not contain uppercase characters".into()));
        }

        // Build the full email address from the local part and domain name
        let full_email = format!("{}@{}", self.email_prefix, self.domain_name);

        // Check if the constructed full email is valid
        if full_email.len() > 254 {
            return Err(AppError::BadRequest("Constructed email is too long".into()));
        }

        Ok(())
    }
}
