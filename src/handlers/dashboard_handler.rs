use axum::{extract::State, Json};
use serde_json::json;

use crate::{middleware::AdminOrAgent, AppState};

/// GET /dashboard/stats — Ambil dashboard statistics
pub async fn get_stats(
    State(state): State<AppState>,
    AdminOrAgent(claims): AdminOrAgent,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let stats = state.dashboard_service.get_stats().await?;

    Ok(Json(json!({
        "success": true,
        "data": stats
    })))
}
