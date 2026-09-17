use base64::{engine::general_purpose::STANDARD, Engine};
use crate::models::errors::AppError;
use sha1::{Sha1, Digest};
use rand::{rng, RngExt};
use serde::Deserialize;


#[derive(Deserialize)]
pub struct PasswordHasher {
    // It stores the Base64-encoded password
    pub encoded_password: String,
}


impl PasswordHasher {
    #[inline]
    fn password_bytes(&self) -> Vec<u8> {
        STANDARD.decode(&self.encoded_password).unwrap_or_default()
    }

    /// Generates a bcrypt hash of the password and returns it as a string
    pub fn generate_bcrypt_hash(&self) -> String {
        bcrypt::hash(self.password_bytes(), bcrypt::DEFAULT_COST).unwrap()
    }

    /// Generates a salted SHA-1 (SSHA) hash of the password. The resulting hash is
    /// encoded in Base64 and prefixed with "{SSHA}" and returned as a string
    pub fn generate_ssha1_hash(&self) -> String {
        let password = self.password_bytes();

        let mut salt = [0u8; 4];
        rng().fill(&mut salt);

        let mut hasher = Sha1::new();
        hasher.update(password);
        hasher.update(salt);

        let digest = hasher.finalize();

        let mut data = Vec::with_capacity(24);
        data.extend_from_slice(&digest);
        data.extend_from_slice(&salt);

        format!("{{SSHA}}{}", STANDARD.encode(data))
    }

    /// Validate the password and basic checks
    pub fn validate(&self) -> Result<(), AppError> {
        let password = self.password_bytes();

        // If the decoded password bytes are empty or invalid, return an error
        if password.is_empty() {
            return Err(AppError::Unprocessable("Password cannot be empty".into()));
        }

        let password_string = String::from_utf8(password).unwrap_or_default();

        // Password should be at least 8 characters long
        if password_string.len() < 8 {
            return Err(AppError::Unprocessable("Password must be at least 8 characters long".into()));
        }

        // Must contain at least one uppercase letter, one lowercase letter, and one digit
        if !password_string.chars().any(|c| c.is_uppercase()) {
            return Err(AppError::Unprocessable("Password must contain at least one uppercase letter".into()));
        }
        if !password_string.chars().any(|c| c.is_lowercase()) {
            return Err(AppError::Unprocessable("Password must contain at least one lowercase letter".into()));
        }
        if !password_string.chars().any(|c| c.is_digit(10)) {
            return Err(AppError::Unprocessable("Password must contain at least one digit".into()));
        }

        // Password must contain at least one special character
        if !password_string.chars().any(|c| !c.is_alphanumeric()) {
            return Err(AppError::Unprocessable("Password must contain at least one special character".into()));
        }

        // Password should not contain whitespace characters
        if password_string.chars().any(|c| c.is_whitespace()) {
            return Err(AppError::Unprocessable("Password must not contain whitespace characters".into()));
        }

        Ok(())
    }
}
