use axum::http::StatusCode;
use chrono::{Duration, Utc};
use dotenvy_macro::dotenv;
use jsonwebtoken::{decode, encode, errors, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::error::AppError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    exp: usize,
    iat: usize,
}

pub fn create_jwt() -> Result<String, StatusCode> {
    let mut now = Utc::now();
    let iat = now.timestamp() as usize;
    let expires_in = Duration::seconds(30);
    now += expires_in;
    let exp = now.timestamp() as usize;
    let claim = Claims { exp, iat };

    let secret = EncodingKey::from_secret(dotenv!("JWT_SECRET_KEY").as_bytes());
    encode(&Header::default(), &claim, &secret).map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn verify_jwt(token: &str) -> Result<bool, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(dotenv!("JWT_SECRET_KEY").as_bytes()),
        &Validation::default(),
    )
    .map_err(|error| match error.kind() {
        errors::ErrorKind::ExpiredSignature => AppError::new(
            StatusCode::UNAUTHORIZED,
            "Your session has expired. Please login again",
        ),
        _ => AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
    })?;

    Ok(true)
}
