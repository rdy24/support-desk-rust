use support_desk::db::create_pool;
use support_desk::create_app;
use tokio::net::TcpListener;
use sqlx::PgPool;
use reqwest::Client;

/// Setup test app — spawn server on random port, return base URL and pool
pub async fn setup_test_app() -> (String, PgPool) {
    dotenvy::dotenv().ok();

    // Gunakan TEST_DATABASE_URL jika ada, jika tidak gunakan default
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/support_desk_test".to_string());

    // Buat connection pool ke test database
    let pool = create_pool(&database_url).await;

    // Jalankan migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Bersihkan data test sebelum mulai (hapus dengan urutan: foreign keys terlebih dulu)
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
    let jwt_secret = "test-jwt-secret-key-for-testing-purposes-only".to_string();
    let app = create_app(pool.clone(), jwt_secret);

    // Spawn server di background dengan random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind test server");

    let addr = listener.local_addr().expect("Failed to get local addr");
    let base_url = format!("http://{}", addr);

    // Jalankan server di background
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server error");
    });

    // Tunggu sebentar agar server siap
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (base_url, pool)
}

/// Register user dan ambil token (helper untuk authenticated tests)
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

    // Login dan ambil token
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login");

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse login response");

    body["data"]["token"]
        .as_str()
        .expect("Token not found in response")
        .to_string()
}
