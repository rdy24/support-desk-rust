# Bab 32: Menyatukan Semua Routes

Bayangkan kamu baru selesai membangun gedung bertingkat. Setiap ruangan sudah ada: ada ruang server, ruang arsip, ruang resepsionis, ruang manajer. Tapi semua ruangan itu masih gelap dan terputus satu sama lain. Listriknya belum dinyalakan. Kabelnya belum disambungkan ke panel utama.

Itulah yang kita lakukan di bab ini.

Handler sudah jadi. Service sudah jadi. Repository sudah jadi. Middleware auth sudah jaga pintu. Sekarang kita sambungkan semuanya ke panel utama, yaitu `main.rs`, lalu nyalakan servernya.

Ini bab klimaks Fase 2. Setelah ini, aplikasi kita benar-benar bisa diakses dari browser atau Postman.

[ILUSTRASI: Diagram gedung bertingkat dengan label setiap lantai (Repository, Service, Handler, Middleware, main.rs sebagai panel listrik di basement yang menyambungkan segalanya)]

---

## Kunci Jawaban & State Sebelumnya

**State Sebelumnya dari Bab 31:**
Semua folder sudah lengkap:
- `src/repositories/` (4 repositories lengkap)
- `src/services/` (auth service, ticket service, user service, dashboard service)
- `src/handlers/` (auth, ticket, user, agent, customer, dashboard handlers)
- `src/middleware/` (JWT auth middleware)
- `src/models/`, `src/dto/`, `src/common/`, `src/db/`

Kesemuanya belum ter-integrate di `main.rs`. Bab 32 adalah wiring semuanya.

---

## Arsitektur Final

Sebelum nulis kode, lihat gambaran besar dulu. Di proyek ini ada beberapa lapisan (*layer*) yang bekerja sama:

| Lapisan | Tugasnya |
|---|---|
| `main.rs` | Titik masuk. Inisialisasi semua hal, jalankan server |
| `handlers/` | Terima request HTTP, panggil service, kirim response |
| `services/` | Logika bisnis — validasi, transformasi data |
| `repositories/` | Akses database, query SQL |
| `middleware/` | Penjaga pintu — cek token JWT sebelum handler dipanggil |
| `common/` | Tipe bersama seperti `AppState`, error types |
| `db/` | Koneksi ke database |

`AppState` adalah struktur yang berisi semua repository. Dia di-*share* ke seluruh handler via Axum. Jadi semua handler bisa akses database tanpa harus membuat koneksi baru setiap saat.

---

## `main.rs` Lengkap

Ini file paling penting, panel listrik gedung kita:

```rust
use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod models;
mod handlers;
mod services;
mod repositories;
mod middleware;
mod common;
mod db;

use db::create_pool;
use common::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL harus di-set");

    let pool = create_pool(&database_url).await;
    sqlx::migrate!("./migrations").run(&pool).await
        .expect("Gagal menjalankan migration");

    let state = AppState {
        user_repo: repositories::UserRepository::new(pool.clone()),
        ticket_repo: repositories::TicketRepository::new(pool.clone()),
        response_repo: repositories::ResponseRepository::new(pool.clone()),
        dashboard_repo: repositories::DashboardRepository::new(pool.clone()),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/auth", handlers::auth_routes())
        .nest("/tickets", handlers::ticket_routes())
        .nest("/users", handlers::user_routes())
        .nest("/agents", handlers::agent_routes())
        .nest("/customers", handlers::customer_routes())
        .nest("/dashboard", handlers::dashboard_routes())
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("🚀 Server berjalan di http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}
```

Bedah bagian-bagian pentingnya:

### `mod` declarations

```rust
mod models;
mod handlers;
mod services;
// ...
```

`mod` di sini artinya "Rust, tolong load modul ini dari folder/file yang namanya sama." Tanpa ini, Rust tidak tahu bahwa folder `handlers/` itu bagian dari proyek kita. Ini seperti mendaftarkan semua divisi perusahaan ke kantor pusat.

### `dotenvy::dotenv().ok()`

Ini membaca file `.env` dan memasukkan semua variabelnya ke environment. Kenapa `.ok()`? Karena kalau file `.env` tidak ada (misalnya di production yang pakai env dari sistem), kita tidak mau crash, cukup diabaikan.

### `create_pool` dan migrasi

```rust
let pool = create_pool(&database_url).await;
sqlx::migrate!("./migrations").run(&pool).await
    .expect("Gagal menjalankan migration");
```

`create_pool` membuat koneksi pool ke PostgreSQL. *Pool* artinya Rust menjaga beberapa koneksi database yang siap pakai, bukan buka-tutup koneksi tiap request, sehingga lebih efisien.

`sqlx::migrate!` menjalankan semua file SQL di folder `migrations/` yang belum dijalankan. Jadi setiap kali server nyala, schema database selalu up-to-date.

### `AppState` dan `.clone()`

```rust
let state = AppState {
    user_repo: repositories::UserRepository::new(pool.clone()),
    ticket_repo: repositories::TicketRepository::new(pool.clone()),
    // ...
};
```

`pool.clone()` bukan menyalin seluruh koneksi. `Pool` di sqlx itu sudah *reference-counted* di dalamnya, jadi clone hanya menambah hitungan referensi. Aman dan murah.

### `.nest()` — Mengelompokkan Routes

```rust
let app = Router::new()
    .nest("/auth", handlers::auth_routes())
    .nest("/tickets", handlers::ticket_routes())
    // ...
```

`.nest("/auth", ...)` artinya semua route yang dikembalikan `auth_routes()` akan punya prefix `/auth`. Jadi kalau di dalam `auth_routes()` ada route `POST /register`, maka URL finalnya jadi `POST /auth/register`.

Ini seperti nomor gedung dan nomor ruangan. `/auth` itu gedungnya, `/register` itu ruangannya.

---

## CORS: Izinkan Frontend Akses API

*CORS* singkatan dari *Cross-Origin Resource Sharing*. Ini mekanisme browser yang memblokir request dari domain berbeda secara default. Misalnya, frontend di `http://localhost:5173` tidak bisa langsung akses API di `http://localhost:3000` tanpa izin eksplisit dari server.

```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

Kita pakai `tower-http` (saat ebook ini ditulis, Maret 2026) untuk setup CORS. `allow_origin(Any)` mengizinkan request dari domain manapun, `allow_methods(Any)` mengizinkan semua HTTP method (GET, POST, PATCH, DELETE, dll), dan `allow_headers(Any)` mengizinkan semua header termasuk `Authorization` yang kita pakai untuk JWT.

> **Catatan untuk production:** `Any` itu terlalu permisif untuk production. Di sana kamu sebaiknya ganti dengan domain frontend spesifik, misalnya `.allow_origin("https://app.namakamu.com".parse::<HeaderValue>().unwrap())`.

`.layer(cors)` harus diletakkan **setelah** semua route didaftarkan tapi **sebelum** `.with_state()`. Urutan ini penting, karena Axum memproses layer dari bawah ke atas saat menerima request.

[ILUSTRASI: Diagram request dari browser melewati CORS layer dulu sebelum sampai ke handler, seperti pos satpam di gerbang gedung]

---

## Checklist Endpoint

Verifikasi bahwa semua endpoint proyek asli sudah tercakup:

### Auth
| Method | Path | Handler | Status |
|---|---|---|---|
| POST | `/auth/register` | `auth_routes()` | ✅ |
| POST | `/auth/login` | `auth_routes()` | ✅ |

### Tickets
| Method | Path | Handler | Status |
|---|---|---|---|
| GET | `/tickets` | `ticket_routes()` | ✅ |
| POST | `/tickets` | `ticket_routes()` | ✅ |
| GET | `/tickets/{id}` | `ticket_routes()` | ✅ |
| PATCH | `/tickets/{id}` | `ticket_routes()` | ✅ |
| DELETE | `/tickets/{id}` | `ticket_routes()` | ✅ |
| POST | `/tickets/{id}/responses` | `ticket_routes()` | ✅ |
| GET | `/tickets/{id}/responses` | `ticket_routes()` | ✅ |

### Users
| Method | Path | Handler | Status |
|---|---|---|---|
| GET | `/users/me` | `user_routes()` | ✅ |
| GET | `/users` | `user_routes()` | ✅ |
| GET | `/users/{id}` | `user_routes()` | ✅ |
| PATCH | `/users/{id}` | `user_routes()` | ✅ |
| DELETE | `/users/{id}` | `user_routes()` | ✅ |

### Agents & Customers
| Method | Path | Handler | Status |
|---|---|---|---|
| GET | `/agents` | `agent_routes()` | ✅ |
| GET | `/customers` | `customer_routes()` | ✅ |

### Dashboard
| Method | Path | Handler | Status |
|---|---|---|---|
| GET | `/dashboard/stats` | `dashboard_routes()` | ✅ |

Total: **18 endpoint**, semua tercakup.

---

## Test Manual dengan curl

Setelah server jalan (`cargo run`), coba endpoint-endpoint ini satu per satu.

### 1. Register user baru

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"Budi","email":"budi@test.com","password":"rahasia123","role":"customer"}'
```

Response yang diharapkan: `201 Created` dengan data user (tanpa password).

### 2. Login dan ambil token

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"budi@test.com","password":"rahasia123"}'
```

Simpan `token` dari response. Kita butuh ini untuk request berikutnya.

### 3. Buat ticket (butuh token)

```bash
curl -X POST http://localhost:3000/tickets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN_KAMU>" \
  -d '{"title":"Tidak bisa login","description":"Saya tidak bisa masuk ke akun saya"}'
```

### 4. Lihat semua ticket

```bash
curl http://localhost:3000/tickets \
  -H "Authorization: Bearer <TOKEN_KAMU>"
```

### 5. Cek data diri

```bash
curl http://localhost:3000/users/me \
  -H "Authorization: Bearer <TOKEN_KAMU>"
```

### 6. Cek stats dashboard (butuh role agent/admin)

```bash
curl http://localhost:3000/dashboard/stats \
  -H "Authorization: Bearer <TOKEN_AGENT>"
```

Kalau server merespons dengan benar di semua endpoint di atas, selamat, aplikasimu berjalan sempurna!

---

## Latihan

1. **Coba tanpa token:** Akses `GET /tickets` tanpa header `Authorization`. Pastikan server mengembalikan `401 Unauthorized`, bukan crash.

2. **Ubah PORT:** Jalankan server di port 8080 dengan cara set environment variable sebelum `cargo run`:
   ```bash
   PORT=8080 cargo run
   ```
   Pastikan server berjalan di port yang benar.

3. **Tambah route baru:** Buat endpoint sederhana `GET /health` yang mengembalikan `{"status":"ok"}` tanpa perlu auth. Daftarkan di `main.rs` dengan `.route("/health", get(health_check))`. Ini berguna untuk monitoring dan load balancer.

4. **Tes CORS:** Buka browser, buka DevTools, jalankan fetch dari console:
   ```javascript
   fetch("http://localhost:3000/auth/login", {
     method: "POST",
     headers: { "Content-Type": "application/json" },
     body: JSON.stringify({ email: "budi@test.com", password: "rahasia123" })
   }).then(r => r.json()).then(console.log)
   ```
   Kalau CORS setup benar, tidak ada error `blocked by CORS policy`.

---

Fase 2 selesai. Semua kabel sudah tersambung, semua ruangan sudah terang. Di Fase 3, kita akan fokus pada testing, error handling yang lebih baik, dan persiapan deployment.
