use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::common::AppError;
use crate::services::{Claims, verify_token, parse_claims_role};
use crate::models::UserRole;

/// Custom extractor untuk authenticated users (any role)
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

/// Custom extractor untuk admin users saja
pub struct AdminOnly(pub Claims);

impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;

        let role = parse_claims_role(&claims.role)?;
        if role != UserRole::Admin {
            return Err(AppError::Forbidden(
                "Hanya admin yang boleh akses endpoint ini".to_string(),
            ));
        }

        Ok(AdminOnly(claims))
    }
}

/// Custom extractor untuk admin atau agent
pub struct AdminOrAgent(pub Claims);

impl<S> FromRequestParts<S> for AdminOrAgent
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;

        let role = parse_claims_role(&claims.role)?;
        if role != UserRole::Admin && role != UserRole::Agent {
            return Err(AppError::Forbidden(
                "Endpoint ini hanya untuk admin atau agent".to_string(),
            ));
        }

        Ok(AdminOrAgent(claims))
    }
}

/// Custom extractor untuk customer users saja
pub struct CustomerOnly(pub Claims);

impl<S> FromRequestParts<S> for CustomerOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;

        let role = parse_claims_role(&claims.role)?;
        if role != UserRole::Customer {
            return Err(AppError::Forbidden(
                "Hanya customer yang boleh akses endpoint ini".to_string(),
            ));
        }

        Ok(CustomerOnly(claims))
    }
}
