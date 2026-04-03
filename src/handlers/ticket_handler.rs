use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dto::{CreateTicketDto, UpdateTicketDto, TicketFilters},
    models::CreateTicketResponseDto,
    middleware::{AuthUser, AdminOrAgent},
    AppState,
};

/// POST /tickets — Buat tiket baru
pub async fn create_ticket(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(dto): Json<CreateTicketDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), crate::common::AppError> {
    let ticket = state.ticket_service.create(dto, &claims).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": ticket
        })),
    ))
}

/// GET /tickets — Ambil list tiket
pub async fn get_tickets(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(filters): Query<TicketFilters>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let (tickets, total) = state.ticket_service.get_many(filters, &claims).await?;

    Ok(Json(json!({
        "success": true,
        "data": tickets,
        "total": total
    })))
}

/// GET /tickets/{id} — Ambil tiket berdasarkan ID
pub async fn get_ticket(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let ticket = state.ticket_service.get_by_id(id, &claims).await?;

    Ok(Json(json!({
        "success": true,
        "data": ticket
    })))
}

/// PATCH /tickets/{id} — Update tiket (hanya agent/admin)
pub async fn update_ticket(
    State(state): State<AppState>,
    AdminOrAgent(claims): AdminOrAgent,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateTicketDto>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let ticket = state.ticket_service.update(id, dto, &claims).await?;

    Ok(Json(json!({
        "success": true,
        "data": ticket
    })))
}

/// DELETE /tickets/{id} — Hapus tiket
pub async fn delete_ticket(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, crate::common::AppError> {
    state.ticket_service.delete(id, &claims).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /tickets/{id}/responses — Tambah response ke tiket
pub async fn add_response(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(ticket_id): Path<Uuid>,
    Json(dto): Json<CreateTicketResponseDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), crate::common::AppError> {
    let response = state.ticket_service.add_response(ticket_id, dto, &claims).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": response
        })),
    ))
}

/// GET /tickets/{id}/responses — Ambil semua response untuk tiket
pub async fn get_responses(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(ticket_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let responses = state.ticket_service.get_responses(ticket_id, &claims).await?;

    Ok(Json(json!({
        "success": true,
        "data": responses
    })))
}
