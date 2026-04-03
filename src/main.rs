mod models;
mod dto;
mod common;
mod db;
mod repositories;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::dto::CreateTicketDto;
use validator::Validate;
use tokio::net::TcpListener;
use db::create_pool;
use sqlx::PgPool;
use repositories::{UserRepository, TicketRepository, ResponseRepository};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
}

#[derive(Deserialize)]
struct TicketFilters {
    page: Option<u32>,
    limit: Option<u32>,
    status: Option<String>,
    priority: Option<String>,  // dari latihan #2
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
        "priority": filters.priority
    }))
}

async fn get_ticket(Path(id): Path<u32>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "id": id, "title": "Contoh ticket" }
    })))
}

async fn create_ticket(
    Json(body): Json<CreateTicketDto>,  // ← CHANGE: Json<Value> → Json<CreateTicketDto>
) -> (StatusCode, Json<Value>) {
    // ← TAMBAH: Validasi input sebelum proses
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

// dari latihan #1
async fn delete_ticket(Path(id): Path<u32>) -> StatusCode {
    println!("Menghapus ticket {}", id);
    StatusCode::NO_CONTENT
}

fn ticket_routes() -> Router {
    Router::new()
        .route("/", get(get_tickets).post(create_ticket))
        .route("/{id}", get(get_ticket).delete(delete_ticket))
}

// dari latihan #3
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_user_serialization() {
        let user = models::User {
            id: Uuid::new_v4(),
            name: "Budi".to_string(),
            email: "budi@example.com".to_string(),
            password: "secret_password".to_string(),
            role: models::UserRole::Customer,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string_pretty(&user).unwrap();
        println!("User JSON:\n{}", json);
        
        // Verifikasi password tidak ada di JSON
        assert!(!json.contains("secret_password"));
        assert!(!json.contains("password"));
        assert!(json.contains("Budi"));
        assert!(json.contains("budi@example.com"));
    }
}


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL harus di-set di .env");

    let pool = create_pool(&database_url).await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Gagal menjalankan migrations");

    // Create app state for use in handlers (will be used in Bab 26-27)
    let _app_state = AppState {
        db: pool.clone(),
        user_repo: UserRepository::new(pool.clone()),
        ticket_repo: TicketRepository::new(pool.clone()),
        response_repo: ResponseRepository::new(pool.clone()),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/tickets", ticket_routes())
        .nest("/users", user_routes());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server berjalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}