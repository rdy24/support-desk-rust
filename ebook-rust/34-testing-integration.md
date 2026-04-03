# Bab 34: Testing — Integration Test

Bayangkan kamu beli mobil baru. Pabrik sudah tes setiap komponen: mesin, rem, lampu — semuanya lolos tes satuan. Tapi apakah kamu langsung percaya mobil itu aman dikendarai di jalan raya? Belum tentu.

Makanya ada **test drive**, yaitu tes di mana semua komponen bekerja bareng dalam kondisi nyata. Di dunia software, itulah **integration test**: bukan tes satu fungsi saja, tapi tes seluruh sistem berjalan bersama, dari HTTP request masuk, lewat handler, service, database, sampai response keluar.

---

## Unit Test vs Integration Test

**Unit test** = tes satu komponen secara terisolasi. Seperti tes mesin mobil di bangku uji — tidak perlu roda, tidak perlu bensin sungguhan.

**Integration test** = tes sistem end-to-end. Seperti test drive di sirkuit — pakai semua komponen, kondisi mendekati nyata.

| | Unit Test | Integration Test |
|---|---|---|
| Scope | Satu fungsi/modul | Seluruh sistem |
| Database | Di-mock | Database nyata (test DB) |
| Kecepatan | Sangat cepat | Lebih lambat |
| Lokasi di Rust | `#[cfg(test)]` dalam file | Folder `tests/` |

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 33:**
- 37 unit tests sudah ada untuk auth_service, ticket_service, dto validators
- Semua test running dengan `cargo test`
- Business logic dan JWT parsing ter-test tanpa database

**State Sebelumnya:**
Dari Bab 33, aplikasi sudah punya comprehensive unit tests. Bab 34 fokus ke integration test untuk verify seluruh flow: HTTP request → handler → service → repository → database → response.

---

## Folder `tests/`

Di Rust, integration test tinggal di folder `tests/` di root project — sejajar dengan `src/`.

```
support-desk/
├── src/
│   ├── lib.rs          ← Module exports + create_app()
│   ├── main.rs         ← Entry point saja
│   ├── handlers/
│   ├── services/
│   └── ...
├── tests/
│   ├── common/
│   │   └── mod.rs      ← Test setup helpers
│   ├── auth_test.rs    ← Integration test untuk auth
│   └── ticket_test.rs  ← Integration test untuk ticket
├── Cargo.toml
└── .env
```

File di `tests/` diperlakukan Rust sebagai **crate terpisah**, artinya mereka mengakses aplikasimu seperti konsumen eksternal. Ini ideal untuk integration test.

---

## Architecture: Library + Binary Pattern

Karena integration tests adalah crate terpisah, mereka tidak bisa import dari `src/main.rs`. Solusinya: **pisahkan main.rs menjadi dua**:
- **`src/lib.rs`** — Module re-export + `create_app()` function (bisa di-import oleh tests)
- **`src/main.rs`** — Entry point saja (import dari lib.rs)

Ini standar Rust pattern yang memungkinkan code reuse.

---

## Step 1: Create `src/lib.rs` — Re-export Modules & create_app()

Buat file baru `src/lib.rs` yang berisi semua module declaration dan AppState:

```rust
pub mod models;
pub mod dto;
pub mod common;
pub mod db;
pub mod repositories;
pub mod services;
pub mod handlers;
pub mod middleware;

use axum::{routing::{get, post, patch}, Router};
use tower_http::cors::CorsLayer;
use sqlx::PgPool;
use crate::repositories::{
    UserRepository, TicketRepository, ResponseRepository, DashboardRepository,
};
use crate::services::{AuthService, TicketService, UserService, DashboardService};

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

/// Fungsi publik untuk membuat full app router
pub fn create_app(pool: PgPool, jwt_secret: String) -> Router {
    let state = AppState::new(pool, jwt_secret);

    let stateful_routes = Router::new()
        .route("/auth/register", post(handlers::auth_handler::register))
        .route("/auth/login", post(handlers::auth_handler::login))
        .route("/me", get(handlers::user_handler::get_me))
        .route("/users", get(handlers::user_handler::get_all_users))
        .route("/users/:id", get(handlers::user_handler::get_user))
        .route("/users/:id", patch(handlers::user_handler::update_user))
        .route("/users/:id", axum::routing::delete(handlers::user_handler::delete_user))
        .route("/agents", get(handlers::user_handler::get_agents))
        .route("/customers", get(handlers::user_handler::get_customers))
        .route("/tickets", post(handlers::ticket_handler::create_ticket))
        .route("/tickets", get(handlers::ticket_handler::get_tickets))
        .route("/tickets/:id", get(handlers::ticket_handler::get_ticket))
        .route("/tickets/:id", patch(handlers::ticket_handler::update_ticket))
        .route("/tickets/:id", axum::routing::delete(handlers::ticket_handler::delete_ticket))
        .route("/tickets/:id/responses", post(handlers::ticket_handler::add_response))
        .route("/tickets/:id/responses", get(handlers::ticket_handler::get_responses))
        .route("/dashboard/stats", get(handlers::dashboard_handler::get_stats))
        .with_state(state);

    let cors = CorsLayer::permissive();

    Router::new()
        .route("/health", get(health_check))
        .merge(stateful_routes)
        .layer(cors)
}
```

**Key points:**
- `pub mod` untuk setiap module — agar tests bisa import
- `pub struct AppState` — agar tests bisa buat instance
- `pub fn create_app()` — agar tests bisa buat router tanpa instansiasi manual

---

## Step 2: Simplify `src/main.rs` — Use lib.rs

Update `src/main.rs` untuk import dari lib.rs dan jalankan server:

```rust
use tokio::net::TcpListener;
use support_desk::db::create_pool;
use support_desk::create_app;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL harus di-set di .env");

    let pool = create_pool(&database_url).await;

    // Verify koneksi
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => println!("✓ Database connected"),
        Err(e) => {
            eprintln!("✗ Database connection failed: {}", e);
            return;
        }
    }

    // Jalankan migrations
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("✓ Migrations executed"),
        Err(e) => {
            eprintln!("✗ Migrations failed: {}", e);
            return;
        }
    }

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET harus di-set di .env");

    let app = create_app(pool, jwt_secret);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server berjalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
```

---

## Step 3: Add dev-dependencies

Update `Cargo.toml` dengan dev-dependencies untuk testing:

```toml
[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
```

`tokio` sudah ada di [dependencies] dengan `full` features, jadi tidak perlu duplikasi.

---

## Step 4: Create Test Setup Helper — `tests/common/mod.rs`

Fungsi helper untuk setup test database dan spawn server:

```rust
use support_desk::db::create_pool;
use support_desk::create_app;
use tokio::net::TcpListener;
use sqlx::PgPool;
use reqwest::Client;

/// Setup test app — spawn server di random port, return (base_url, pool)
pub async fn setup_test_app() -> (String, PgPool) {
    dotenvy::dotenv().ok();

    // Gunakan TEST_DATABASE_URL dari .env, atau default
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| 
            "postgres://postgres:postgres@localhost:5432/support_desk_test".to_string()
        );

    let pool = create_pool(&database_url).await;

    // Jalankan migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Bersihkan data dari test sebelumnya (urutan: FK dulu)
    sqlx::query("DELETE FROM ticket_responses")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM tickets")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await
        .ok();

    // Buat app
    let jwt_secret = "test-jwt-secret-for-testing".to_string();
    let app = create_app(pool.clone(), jwt_secret);

    // Spawn server di background dengan port random (0 = OS choose)
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");

    let addr = listener.local_addr().expect("Failed to get addr");
    let base_url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server error");
    });

    // Tunggu server ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (base_url, pool)
}

/// Register user dan login untuk dapat token
pub async fn register_and_login(
    base_url: &str,
    email: &str,
    password: &str,
) -> String {
    let client = Client::new();

    // Register dengan role "customer"
    client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Test User",
            "email": email,
            "password": password,
            "role": "customer"
        }))
        .send()
        .await
        .expect("Failed to register")
        .text()
        .await
        .ok();

    // Login
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login");

    let body: serde_json::Value = response.json().await.expect("Failed to parse");
    body["data"]["token"].as_str().expect("No token").to_string()
}
```

**Key points:**
- `TcpListener::bind("127.0.0.1:0")` — OS memilih port tersedia, hindari konflik
- `tokio::spawn()` — jalankan server di background task
- `DELETE` dengan urutan: foreign keys dulu (ticket_responses → tickets → users)
- `register_and_login()` helper — avoid duplikasi di setiap test

---

## Step 5: Integration Tests — `tests/auth_test.rs`

Test full auth flow dari HTTP:

```rust
mod common;

use common::setup_test_app;
use reqwest::Client;

#[tokio::test]
async fn test_register_success() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "password123",
            "role": "customer"
        }))
        .send()
        .await
        .expect("Failed to send");

    assert_eq!(response.status(), 201); // Created
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse");
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["email"], "test@example.com");
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register pertama
    client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "First",
            "email": "dup@example.com",
            "password": "password123",
            "role": "customer"
        }))
        .send().await.ok();

    // Register kedua dengan email sama
    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Second",
            "email": "dup@example.com",
            "password": "password456",
            "role": "customer"
        }))
        .send().await.expect("Failed");

    assert_eq!(response.status(), 409); // Conflict
}

#[tokio::test]
async fn test_login_success() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register dulu
    client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Login User",
            "email": "login@example.com",
            "password": "password123",
            "role": "customer"
        }))
        .send().await.ok();

    // Login
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": "login@example.com",
            "password": "password123"
        }))
        .send().await.expect("Failed");

    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse");
    assert!(body["data"]["token"].is_string()); // Token ada
}

#[tokio::test]
async fn test_login_wrong_password() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Wrong Pass",
            "email": "wrongpass@example.com",
            "password": "correctpassword",
            "role": "customer"
        }))
        .send().await.ok();

    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": "wrongpass@example.com",
            "password": "wrongpassword"
        }))
        .send().await.expect("Failed");

    assert_eq!(response.status(), 401); // Unauthorized
}
```

---

## Step 6: Integration Tests — `tests/ticket_test.rs`

Test ticket flow dengan authentication:

```rust
mod common;

use common::{setup_test_app, register_and_login};
use reqwest::Client;
use uuid::Uuid;

#[tokio::test]
async fn test_create_ticket_authenticated() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    let email = format!("user_{}@example.com", Uuid::new_v4());
    let token = register_and_login(&base_url, &email, "password123").await;

    let response = client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "subject": "Cannot login",
            "description": "Unable to access my account",
            "category": "technical",
            "priority": "high"
        }))
        .send().await.expect("Failed");

    assert_eq!(response.status(), 201); // Created
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse");
    assert_eq!(body["data"]["subject"], "Cannot login");
    assert_eq!(body["data"]["category"], "technical");
}

#[tokio::test]
async fn test_create_ticket_unauthenticated() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Tanpa token
    let response = client
        .post(format!("{}/tickets", base_url))
        .json(&serde_json::json!({
            "subject": "Some problem",
            "description": "Description",
            "category": "general",
            "priority": "low"
        }))
        .send().await.expect("Failed");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_get_tickets_list() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    let email = format!("user_{}@example.com", Uuid::new_v4());
    let token = register_and_login(&base_url, &email, "password123").await;

    // Buat 2 ticket
    client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "subject": "First",
            "description": "First ticket",
            "category": "general",
            "priority": "low"
        }))
        .send().await.ok();

    client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "subject": "Second",
            "description": "Second ticket",
            "category": "billing",
            "priority": "medium"
        }))
        .send().await.ok();

    // List tiket
    let response = client
        .get(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .send().await.expect("Failed");

    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed");
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}
```

---

## Running Integration Tests

```bash
# Build tests (verify compile without running)
cargo build --tests

# Run semua integration tests
cargo test --test auth_test
cargo test --test ticket_test

# Run tests secara sequential (hindari race conditions)
cargo test --test auth_test -- --test-threads=1

# Run semua tests (unit + integration)
cargo test
```

Output akan menunjukkan:
- 37 unit tests dari Ch33 (auth_service, ticket_dto, ticket_service)
- 6+ integration tests dari Ch34

---

## Troubleshooting

**Error: "database not found"**
- Buat test database: `CREATE DATABASE support_desk_test;` di PostgreSQL
- Set `TEST_DATABASE_URL` di `.env`

**Error: "address already in use"**
- Gunakan `TcpListener::bind("127.0.0.1:0")` — sudah di-handle di setup helper

**Tests berjalan lambat atau fail**
- Gunakan `--test-threads=1` untuk sequential execution
- Pastikan cleanup di setup_test_app() berhasil (DELETE statements)

---

## Tips: Isolasi Test

**1. Unique emails per test:**
```rust
let email = format!("user_{}@example.com", uuid::Uuid::new_v4());
```

**2. Cleanup sebelum test:**
```rust
// Di setup_test_app(), DELETE dari semua table
sqlx::query("DELETE FROM ticket_responses").execute(&pool).await.ok();
sqlx::query("DELETE FROM tickets").execute(&pool).await.ok();
sqlx::query("DELETE FROM users").execute(&pool).await.ok();
```

**3. Test database terpisah:**
Jangan pakai production database. Setup test database khusus.

---

## Latihan

1. Tambahkan test untuk endpoint `GET /me` — pastikan user bisa ambil profil sendiri dengan token.

2. Buat test untuk skenario authorization: customer coba akses ticket orang lain, harus return `403 Forbidden`. (Hint: buat 2 users, user1 buat ticket, user2 coba get ticket, assert 403)

3. Tulis test untuk update ticket: hanya agent/admin yang boleh update status. Customer update ticket harus return `403`.

4. **Tantangan**: Refactor `setup_test_app()` agar setiap test dapat database yang benar-benar isolated menggunakan PostgreSQL schema atau transaksi rollback.

---

## Hasil Akhir

Setelah bab 34, project structure menjadi:

```
src/
├── lib.rs           ← Module exports + create_app()
├── main.rs          ← Entry point (thin)
├── handlers/
├── services/
├── repositories/
└── ...

tests/
├── common/
│   └── mod.rs       ← setup_test_app(), register_and_login()
├── auth_test.rs     ← 5 auth integration tests
└── ticket_test.rs   ← 3 ticket integration tests

Cargo.toml           ← Dengan [dev-dependencies] reqwest
```

**Test Statistics:**
- Unit tests: 37 (dari Ch33)
- Integration tests: 8 (dari Ch34)
- Total: 45 tests covering unit + integration + end-to-end flows

**Run semua:**
```bash
cargo test
```

Output:
```
running 45 tests
test auth_test::test_login_success ... ok
test auth_test::test_register_duplicate_email ... ok
test auth_test::test_register_success ... ok
...
test services::ticket_service::tests::test_check_access_admin_allowed ... ok
...
test result: ok. 45 passed; 0 failed
```

Aplikasi sudah fully tested dari unit level sampai end-to-end integration. Siap untuk deployment!

Fase 2-4 selesai: Architecture (Ch27-29) ✓ → Services & Routes (Ch30-32) ✓ → Unit Testing (Ch33) ✓ → Integration Testing (Ch34) ✓
