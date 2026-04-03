# Bab 34: Testing - Integration Test

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

**Kunci Jawaban Latihan Bab 32:**
- Wiring semua routes di main.rs sudah lengkap di "Hasil Akhir Bab 32"
- AppState dengan semua repositories sudah initialized
- CORS layer sudah dikonfigurasi
- Semuanya sudah connected end-to-end

**State Sebelumnya:**
Dari Bab 32, aplikasi sudah sepenuhnya ter-wire. Semua endpoint sudah connected dari HTTP request → handler → service → repository → database → response. Bab 34 fokus ke integration test untuk verify seluruh flow.

---

Keduanya penting. Unit test untuk logic detail, integration test untuk memastikan semuanya nyambung.

---

## Folder `tests/`

Di Rust, integration test tinggal di folder `tests/` di root project — sejajar dengan `src/`.

```
support-desk/
├── src/
│   ├── main.rs
│   ├── routes/
│   └── ...
├── tests/
│   ├── auth_test.rs      ← integration test auth
│   └── ticket_test.rs    ← integration test ticket
├── Cargo.toml
└── .env
```

File di `tests/` diperlakukan Rust sebagai **crate terpisah**, artinya mereka mengakses aplikasimu seperti konsumen eksternal, bukan insider. Ini yang bikin mereka ideal sebagai integration test.

Setiap file di `tests/` otomatis jadi test binary tersendiri. Tidak perlu konfigurasi tambahan di `Cargo.toml`.

---

## Setup: Test Database Terpisah

[ILUSTRASI: diagram dua database — satu production (berwarna merah bertulis "DATA ASLI - JANGAN DISENTUH") dan satu test (berwarna hijau bertulis "Boleh dihapus kapan saja")]

Ini aturan wajib: **jangan pernah jalankan test di database production**. Test akan insert, update, bahkan delete data. Kalau salah database, data user asli bisa rusak.

Buat database terpisah khusus test. Tambahkan ke `.env`:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/support_desk
TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/support_desk_test
```

Buat database-nya di PostgreSQL:

```sql
CREATE DATABASE support_desk_test;
```

---

## Setup Test App Helper

Sebelum setiap test, kita perlu menyiapkan server yang berjalan di background. Buat fungsi helper `setup_test_app` yang bisa dipanggil di setiap test:

```rust
// tests/auth_test.rs
use axum::Router;
use reqwest::Client;
// reqwest = "0.12" (saat ebook ini ditulis, Maret 2026)

async fn setup_test_app() -> (String, PgPool) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/support_desk_test".to_string());

    let pool = create_pool(&database_url).await;
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // Bersihkan data test sebelum mulai
    sqlx::query!("DELETE FROM ticket_responses").execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM tickets").execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM users").execute(&pool).await.unwrap();

    let app = create_app(pool.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(axum::serve(listener, app));

    (format!("http://{}", addr), pool)
}
```

Ada beberapa hal penting di sini. **`TcpListener::bind("127.0.0.1:0")`**: port `0` artinya OS yang pilihkan port tersedia, mencegah konflik kalau beberapa test berjalan bersamaan. **`tokio::spawn(...)`** menjalankan server di background task sehingga test bisa lanjut sementara server mendengarkan request. **DELETE sebelum test** membersihkan data dari test sebelumnya, urutan DELETE penting, hapus tabel dengan foreign key lebih dulu (ticket_responses → tickets → users). **`sqlx::migrate!`** memastikan schema database selalu up-to-date sebelum test dimulai.

---

## Test Register dan Login

Dengan helper di atas, test jadi sangat bersih:

```rust
#[tokio::test]
async fn test_register_success() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    let response = client
        .post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["email"].as_str().unwrap() == "test@example.com");
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register pertama kali
    client.post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Test",
            "email": "dup@example.com",
            "password": "password123"
        }))
        .send().await.unwrap();

    // Register kedua dengan email sama
    let response = client.post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Test2",
            "email": "dup@example.com",
            "password": "password456"
        }))
        .send().await.unwrap();

    assert_eq!(response.status(), 409); // Conflict
}
```

Pola yang diikuti setiap test: **Setup**, panggil `setup_test_app()`, dapat URL dan pool. **Act**, kirim HTTP request seperti client sungguhan. **Assert**, cek status code dan body response. **Cleanup**, sudah dihandle di `setup_test_app()` di awal test berikutnya.

---

## Test Ticket dengan Auth

Banyak endpoint butuh JWT token. Buat helper tambahan untuk login dan ambil token:

```rust
async fn login_and_get_token(base_url: &str, email: &str, password: &str) -> String {
    let client = Client::new();
    let response = client
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = response.json().await.unwrap();
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_ticket_authenticated() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    // Register dulu
    client.post(format!("{}/auth/register", base_url))
        .json(&serde_json::json!({
            "name": "Ticket User",
            "email": "ticket@example.com",
            "password": "password123"
        }))
        .send().await.unwrap();

    // Login, ambil token
    let token = login_and_get_token(&base_url, "ticket@example.com", "password123").await;

    // Buat ticket dengan token
    let response = client
        .post(format!("{}/tickets", base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "title": "Masalah Login",
            "description": "Tidak bisa login ke sistem"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn test_create_ticket_unauthenticated() {
    let (base_url, _pool) = setup_test_app().await;
    let client = Client::new();

    let response = client
        .post(format!("{}/tickets", base_url))
        .json(&serde_json::json!({
            "title": "Masalah",
            "description": "Deskripsi"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401); // Unauthorized
}
```

---

## Async Test: `#[tokio::test]`

[ILUSTRASI: perbandingan `#[test]` biasa (synchronous, linear) vs `#[tokio::test]` (async, bisa menunggu tanpa blocking)]

Test biasa di Rust pakai `#[test]`. Tapi karena aplikasi kita async (pakai Tokio), test juga harus async. Caranya cukup ganti atributnya:

```rust
// Test biasa (synchronous)
#[test]
fn test_something() { ... }

// Test async dengan Tokio
#[tokio::test]
async fn test_something_async() { ... }
```

`#[tokio::test]` otomatis membungkus test dalam Tokio runtime — kita tidak perlu buat runtime manual. Pastikan `tokio` di `Cargo.toml` punya feature `macros`:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
# (saat ebook ini ditulis, Maret 2026)
```

---

## Tips: Isolasi Test

Integration test bisa flaky (tidak stabil) kalau tidak hati-hati. Beberapa tips:

**Jalankan test secara sequential untuk satu file:**

```bash
cargo test --test auth_test -- --test-threads=1
```

Secara default Rust menjalankan test paralel. Kalau dua test pakai database yang sama dan saling overwrite data, hasilnya tidak konsisten. `--test-threads=1` paksa sequential.

**Gunakan email unik per test.** Kalau dua test berjalan paralel dan sama-sama register dengan `test@example.com`, salah satu akan gagal karena duplicate. Pakai UUID atau timestamp:

```rust
let email = format!("test_{}@example.com", uuid::Uuid::new_v4());
```

**Jalankan semua integration test:**

```bash
cargo test --test auth_test
cargo test --test ticket_test

# Atau semua sekaligus
cargo test
```

---

## Latihan

1. Tambahkan test untuk endpoint `GET /tickets` — pastikan user hanya bisa lihat tiket miliknya sendiri, bukan tiket user lain.

2. Buat test untuk skenario login gagal: email tidak terdaftar harus return `404`, password salah harus return `401`.

3. Tulis test untuk RBAC: pastikan user biasa tidak bisa mengakses endpoint admin (misal `GET /admin/users`), dan harus return `403 Forbidden`.

4. **Tantangan**: Refactor `setup_test_app()` agar setiap test dapat database yang benar-benar terisolasi menggunakan PostgreSQL schema terpisah atau transaksi yang di-rollback setelah test selesai.
