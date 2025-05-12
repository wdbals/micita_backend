use crate::errors::ApiError;
use crate::models::enums::UserRole;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // user id
    pub role: UserRole,
    pub exp: usize, // expiry timestamp
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_jwt(user_id: i32, role: &UserRole) -> Result<String, ApiError> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| ApiError::InternalServerError("JWT_SECRET no declarado".into()))?;
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        role: role.clone(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalServerError(e.to_string()))
}

pub fn decode_jwt(token: &str) -> Result<Claims, ApiError> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| ApiError::InternalServerError("JWT_SECRET no declarado".into()))?;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|e| ApiError::Unauthorized(e.to_string()))
}
