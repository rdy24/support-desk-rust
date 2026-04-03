use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::common::AppError;
use crate::services::{Claims, verify_token};

/// Custom extractor untuk authenticated users
/// Digunakan di handler sebagai parameter untuk mengakses JWT claims
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Token diperlukan".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                AppError::Unauthorized("Format token salah, gunakan: Bearer <token>".to_string())
            })?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET tidak dikonfigurasi".to_string()))?;

        let claims = verify_token(token, &jwt_secret)?;
        Ok(AuthUser(claims))
    }
}
