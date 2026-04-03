mod models;
mod dto;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use crate::dto::CreateTicketDto;
use validator::Validate;

#[derive(Deserialize)]
struct TicketFilters {
    page: Option<u32>,
    limit: Option<u32>,
    #[serde(default)]
    status: Option<String>,
    priority: Option<String>,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_tickets(Query(filters): Query<TicketFilters>) -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "page": filters.page.unwrap_or(1),
        "limit": filters.limit.unwrap_or(10),
        "status": filters.status,
        "priority": filters.priority
    }))
}

async fn get_ticket(Path(id): Path<u32>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "id": id, "title": "Contoh ticket" }
    })))
}

async fn create_ticket(Json(body): Json<CreateTicketDto>) -> (StatusCode, Json<Value>) {
    // Validasi input sebelum proses
    if let Err(errors) = body.validate() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "success": false,
                "message": "Validasi gagal",
                "errors": errors.to_string()
            })),
        );
    }

    // Data sudah bersih, lanjut proses bisnis
    println!("Ticket baru: subject={}, category={}", body.subject, body.category);

    (StatusCode::CREATED, Json(json!({
        "success": true,
        "message": "Ticket berhasil dibuat"
    })))
}

async fn delete_ticket(Path(id): Path<u32>) -> StatusCode {
    println!("Menghapus ticket {}", id);
    StatusCode::NO_CONTENT
}

fn ticket_routes() -> Router {
    Router::new()
        .route("/", get(get_tickets))
        .route("/", axum::routing::post(create_ticket))
        .route("/{id}", get(get_ticket))
        .route("/{id}", axum::routing::delete(delete_ticket))
}

async fn get_users() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": []
    }))
}

async fn get_user(Path(id): Path<u32>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "id": id, "name": "Contoh user" }
    })))
}

fn user_routes() -> Router {
    Router::new()
        .route("/", get(get_users))
        .route("/{id}", get(get_user))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/tickets", ticket_routes())
        .nest("/users", user_routes());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server berjalan di http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
