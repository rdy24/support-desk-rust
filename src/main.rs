use tokio::net::TcpListener;
use support_desk::db::create_pool;
use support_desk::create_app;

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

    // Buat app dengan pool dan JWT secret
    let app = create_app(pool, jwt_secret);

    // Baca PORT dari environment, default 3000 jika tidak ada
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server berjalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
