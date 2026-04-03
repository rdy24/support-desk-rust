use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use crate::{dto::{LoginDto, RegisterDto}, AppState};

/// Handler untuk register (POST /auth/register)
pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), crate::common::AppError> {
    let user = state.auth_service.register(dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": user
        })),
    ))
}

/// Handler untuk login (POST /auth/login)
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let token = state.auth_service.login(dto).await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "token": token
        }
    })))
}
