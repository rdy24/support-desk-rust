pub mod models;
pub mod dto;
pub mod common;
pub mod db;
pub mod repositories;
pub mod services;
pub mod handlers;
pub mod middleware;

use axum::{
    routing::{get, post, patch},
    Router,
};
use tower_http::cors::CorsLayer;
use sqlx::PgPool;
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

/// Create the full Axum router with all routes, state, and middleware
pub fn create_app(pool: PgPool, jwt_secret: String) -> Router {
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

    // Setup CORS layer
    let cors = CorsLayer::permissive();

    // Setup router dengan semua routes
    Router::new()
        .route("/health", get(health_check))
        .merge(stateful_routes)
        .layer(cors)
}
