use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dto::{UpdateUserDto, UserFilters},
    middleware::{AuthUser, AdminOnly},
    AppState,
};

/// GET /users/me — Ambil profil sendiri
pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let user = state.user_service.get_me(&claims).await?;

    Ok(Json(json!({
        "success": true,
        "data": user
    })))
}

/// GET /users — Ambil semua user
pub async fn get_all_users(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Query(filters): Query<UserFilters>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let (users, total) = state
        .user_service
        .get_all(None, filters.page.unwrap_or(1) as i64, filters.limit.unwrap_or(10) as i64)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": users,
        "total": total
    })))
}

/// GET /agents — Ambil semua agent
pub async fn get_agents(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Query(filters): Query<UserFilters>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let (agents, total) = state
        .user_service
        .get_all(Some("agent"), filters.page.unwrap_or(1) as i64, filters.limit.unwrap_or(10) as i64)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": agents,
        "total": total
    })))
}

/// GET /customers — Ambil semua customer
pub async fn get_customers(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Query(filters): Query<UserFilters>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let (customers, total) = state
        .user_service
        .get_all(Some("customer"), filters.page.unwrap_or(1) as i64, filters.limit.unwrap_or(10) as i64)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": customers,
        "total": total
    })))
}

/// GET /users/:id — Ambil user berdasarkan ID
pub async fn get_user(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let user = state.user_service.get_by_id(user_id).await?;

    Ok(Json(json!({
        "success": true,
        "data": user
    })))
}

/// PATCH /users/:id — Update user
pub async fn update_user(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Path(user_id): Path<Uuid>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let user = state.user_service.update(user_id, dto).await?;

    Ok(Json(json!({
        "success": true,
        "data": user
    })))
}

/// DELETE /users/:id — Hapus user
pub async fn delete_user(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, crate::common::AppError> {
    state.user_service.delete(user_id, &claims).await?;
    Ok(StatusCode::NO_CONTENT)
}
