use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Membuat connection pool ke PostgreSQL
/// Connection pool menjalankan beberapa koneksi siap pakai,
/// sehingga tidak perlu membuat koneksi baru untuk setiap request
pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Gagal koneksi ke database")
}
