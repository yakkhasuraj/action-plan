use axum::http::StatusCode;
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: String) -> Result<String, StatusCode> {
    hash(password, DEFAULT_COST).map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn verify_password(password: String, hash: &str) -> Result<bool, StatusCode> {
    verify(password, hash).map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)
}
