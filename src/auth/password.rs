use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher as ArgonPasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};

use crate::error::{AppError, AppResult};

pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    pub fn hash(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;

        Ok(hash.to_string())
    }

    pub fn verify(&self, password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Invalid hash format: {}", e)))?;

        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map(|_| true)
            .or_else(|_| Ok(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let hasher = PasswordHasher::new();
        let password = "my_secure_password_123";

        let hash = hasher.hash(password).unwrap();

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hasher = PasswordHasher::new();

        let hash1 = hasher.hash("password1").unwrap();
        let hash2 = hasher.hash("password2").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_different_hashes() {
        let hasher = PasswordHasher::new();
        let password = "same_password";

        let hash1 = hasher.hash(password).unwrap();
        let hash2 = hasher.hash(password).unwrap();

        assert_ne!(hash1, hash2);
        assert!(hasher.verify(password, &hash1).unwrap());
        assert!(hasher.verify(password, &hash2).unwrap());
    }
}
