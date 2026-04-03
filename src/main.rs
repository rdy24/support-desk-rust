mod models;
mod dto;
mod common;
mod db;
mod repositories;
mod services;
mod handlers;
mod middleware;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use crate::dto::CreateTicketDto;
use validator::Validate;
use sqlx::PgPool;
use db::create_pool;
use crate::repositories::{
    UserRepository, TicketRepository, ResponseRepository, DashboardRepository,
};
use crate::services::AuthService;

// ============================================
// AppState — berbagi repositories, services, dan pool ke semua handler
// ============================================
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
    pub auth_service: AuthService,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = UserRepository::new(pool.clone());

        Self {
            user_repo: user_repo.clone(),
            ticket_repo: TicketRepository::new(pool.clone()),
            response_repo: ResponseRepository::new(pool.clone()),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            auth_service: AuthService::new(user_repo, jwt_secret.clone()),
            jwt_secret,
            db: pool,
        }
    }
}

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
    // Load .env file
    dotenvy::dotenv().ok();

    // Baca DATABASE_URL dari environment (.env file atau system env var)
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL harus di-set di .env");

    // Buat connection pool ke database
    let pool = create_pool(&database_url).await;

    // Verifikasi koneksi berhasil dengan test query
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => println!("✓ Database connected successfully"),
        Err(e) => eprintln!("✗ Database connection failed: {}", e),
    }

    // Jalankan migrations otomatis
    match sqlx::migrate!("./migrations")
        .run(&pool)
        .await {
        Ok(_) => println!("✓ Migrations executed successfully"),
        Err(e) => {
            eprintln!("✗ Migrations failed: {}", e);
            return;
        }
    }

    // Baca JWT_SECRET dari environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET harus di-set di .env");

    // Buat AppState dengan semua repositories dan services
    let state = AppState::new(pool, jwt_secret);

    // Setup auth routes dengan state
    let auth_routes = Router::new()
        .route("/auth/register", post(handlers::auth_handler::register))
        .route("/auth/login", post(handlers::auth_handler::login))
        .with_state(state);

    // Setup router dengan semua routes
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(auth_routes)
        .nest("/tickets", ticket_routes())
        .nest("/users", user_routes());

    // Baca PORT dari environment, default 3000 jika tidak ada
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server berjalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
