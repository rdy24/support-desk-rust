mod models;
mod dto;
mod common;
mod db;
mod repositories;
mod services;
mod handlers;
mod middleware;

use axum::{
    routing::{get, post, patch},
    Json, Router,
};
use serde_json::json;
use tokio::net::TcpListener;
use sqlx::PgPool;
use db::create_pool;
use crate::repositories::{
    UserRepository, TicketRepository, ResponseRepository, DashboardRepository,
};
use crate::services::{AuthService, TicketService, UserService, DashboardService};

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
    pub ticket_service: TicketService,
    pub user_service: UserService,
    pub dashboard_service: DashboardService,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = UserRepository::new(pool.clone());
        let ticket_repo = TicketRepository::new(pool.clone());
        let response_repo = ResponseRepository::new(pool.clone());
        let dashboard_repo = DashboardRepository::new(pool.clone());

        Self {
            db: pool,
            user_repo: user_repo.clone(),
            ticket_repo: ticket_repo.clone(),
            response_repo: response_repo.clone(),
            dashboard_repo: dashboard_repo.clone(),
            auth_service: AuthService::new(user_repo.clone(), jwt_secret.clone()),
            ticket_service: TicketService::new(ticket_repo.clone(), response_repo),
            user_service: UserService::new(user_repo),
            dashboard_service: DashboardService::new(dashboard_repo),
            jwt_secret,
        }
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_current_user(
    crate::middleware::AuthUser(claims): crate::middleware::AuthUser,
) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": {
            "id": claims.sub,
            "email": claims.email,
            "role": claims.role
        }
    }))
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

    // Setup semua routes dengan state
    let stateful_routes = Router::new()
        // Auth routes
        .route("/auth/register", post(handlers::auth_handler::register))
        .route("/auth/login", post(handlers::auth_handler::login))
        // User routes
        .route("/me", get(handlers::user_handler::get_me))
        .route("/users", get(handlers::user_handler::get_all_users))
        .route("/users/:id", get(handlers::user_handler::get_user))
        .route("/users/:id", patch(handlers::user_handler::update_user))
        .route("/users/:id", axum::routing::delete(handlers::user_handler::delete_user))
        .route("/agents", get(handlers::user_handler::get_agents))
        .route("/customers", get(handlers::user_handler::get_customers))
        // Ticket routes
        .route("/tickets", post(handlers::ticket_handler::create_ticket))
        .route("/tickets", get(handlers::ticket_handler::get_tickets))
        .route("/tickets/:id", get(handlers::ticket_handler::get_ticket))
        .route("/tickets/:id", patch(handlers::ticket_handler::update_ticket))
        .route("/tickets/:id", axum::routing::delete(handlers::ticket_handler::delete_ticket))
        .route("/tickets/:id/responses", post(handlers::ticket_handler::add_response))
        .route("/tickets/:id/responses", get(handlers::ticket_handler::get_responses))
        // Dashboard routes
        .route("/dashboard/stats", get(handlers::dashboard_handler::get_stats))
        .with_state(state);

    // Setup router dengan semua routes
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(stateful_routes);

    // Baca PORT dari environment, default 3000 jika tidak ada
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server berjalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
