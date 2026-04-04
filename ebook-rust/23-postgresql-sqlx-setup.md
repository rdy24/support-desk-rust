# Bab 23: PostgreSQL dan SQLx Setup

Sampai di sini, aplikasi sudah bisa menerima request, memvalidasi input, dan mengembalikan response JSON. Tapi ada satu masalah besar: semua data hilang setiap kali server di-restart. Waktunya sambungkan ke database sungguhan.

Bab ini membahas cara setup PostgreSQL via Docker, mengamankan konfigurasi dengan file `.env`, lalu menghubungkan semuanya ke Axum lewat SQLx.

[ILUSTRASI: Diagram alur dari request HTTP → Axum Handler → SQLx → PostgreSQL, dengan panah dua arah menunjukkan data mengalir bolak-balik]

---

## Kunci Jawaban Latihan Bab 22

Berikut jawaban untuk latihan Bab 22:

### Latihan #1: Tambah variant `BadRequest`

Update `src/common/response.rs`:

```rust
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    BadRequest(String),          // ← TAMBAH INI
    ValidationError(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),  // ← TAMBAH INI
            AppError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        // ... rest tetap sama
    }
}
```

### Latihan #2: Buat handler dengan AppError

Tambahkan di `src/main.rs` (contoh handler):

```rust
use crate::models::Ticket;
use crate::common::{AppResult, ApiResponse};
use uuid::Uuid;

// Handler sederhana yang mendemonstrasikan AppError
async fn get_ticket(Path(id): Path<i32>) -> AppResult<ApiResponse<Ticket>> {
    if id % 2 != 0 {  // Kalau ganjil
        return Err(AppError::NotFound(format!("Ticket {} tidak ditemukan", id)));
    }
    
    // Kalau genap, return dummy data
    let ticket = Ticket {
        id: Uuid::new_v4(),
        customer_id: Uuid::new_v4(),
        agent_id: None,
        category: "technical".to_string(),
        priority: "high".to_string(),
        status: "open".to_string(),
        subject: "Server Issue".to_string(),
        description: "Server tidak bisa diakses".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    Ok(ApiResponse::ok(ticket, "Ticket ditemukan"))
}
```

### Latihan #3: Unit test pagination

Sudah ada di `src/common/response.rs` test module. Jalankan dengan:
```bash
cargo test test_pagination
```

### Latihan #4: Explore `skip_serializing_if` (OPTIONAL)

Hapus `#[serde(skip_serializing_if = "Option::is_none")]` dari `ApiResponse` di `src/common/response.rs`, maka response error akan muncul `"data": null` di JSON. Dengan attribute itu, field data tidak muncul sama sekali jika None.

---

## State Sebelumnya

Sebelum mulai Bab 23, pastikan folder struktur dari Bab 22 sudah ada lengkap:

```
src/
├── main.rs
├── models/
│   ├── mod.rs
│   ├── user.rs
│   ├── ticket.rs
│   └── api_response.rs
├── dto/
│   ├── mod.rs
│   ├── ticket_dto.rs
│   └── user_dto.rs
└── common/
    ├── mod.rs
    └── response.rs
```

Dan `src/main.rs` harus punya mod declarations:
```rust
mod common;
mod dto;
mod models;
```

Verifikasi dengan: `cargo build` harus berhasil tanpa error.

Jika belum, selesaikan Bab 22 terlebih dahulu.

---

## ⚡ Quick Start (TL;DR)

Kalau kamu hanya ingin cepat-cepat jalankan aplikasi tanpa membaca detail:

```bash
# Terminal 1: Start database
docker compose up -d

# Verifikasi database running
docker ps | grep support-desk-db

# Terminal 2: Run server
cargo build
cargo run

# Terminal 3: Test
curl http://localhost:3000/health
```

Kalau semua berjalan: database sudah connected. Baca lanjutan bab ini untuk mengerti cara kerjanya.

---

## SQLx vs ORM

SQLx **bukan ORM**. Perbedaannya penting untuk dipahami sebelum mulai.

ORM seperti Diesel atau SeaORM mengabstraksikan SQL, kamu mendefinisikan query lewat method chain atau macro khusus, dan library yang generate SQL-nya. Praktis, tapi kamu kehilangan kontrol atas query yang dijalankan. SQLx mengambil pendekatan berbeda: kamu tetap menulis SQL mentah, tapi SQLx membantu validasi dan mapping hasilnya ke Rust struct.

Keuntungan konkretnya: SQLx dibangun async-native dari awal, mendukung `async/await` tanpa wrapper tambahan. Fitur paling berharganya adalah *compile-time query checking*: SQLx memverifikasi query SQL saat compile (butuh koneksi DB aktif), bukan saat runtime. Hasil query juga langsung di-map ke Rust struct secara type-safe.

Diesel lebih opinionated dan banyak boilerplate. SeaORM lebih mirip ORM penuh. SQLx cocok kalau kontrol penuh atas SQL lebih diprioritaskan.

---

## Setup PostgreSQL dengan Docker Compose

Docker dipakai supaya tidak perlu install PostgreSQL langsung di mesin. Database berjalan di dalam container yang terisolasi, gampang dibawa kemana-mana.

**Docker Compose** adalah cara terbaik untuk mengelola database di project. Konfigurasi container ditulis di file `docker-compose.yml` di root project. Jalankan satu perintah, semua container langsung siap. Tim lain tinggal clone repo dan jalankan hal yang sama.

### Langkah 1: Buat file `docker-compose.yml`

**Lokasi:** Buat file baru bernama `docker-compose.yml` di **root folder project** (sama level dengan `Cargo.toml`)

**Isi file:**

```yaml
services:
  postgres:
    image: postgres:16-alpine
    container_name: support-desk-db
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: support_desk
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

Penjelasan bagian yang penting:

| Key | Artinya |
|-----|---------|
| `image: postgres:16-alpine` | Pakai PostgreSQL 16 versi alpine (lebih ringan) |
| `restart: unless-stopped` | Container otomatis restart kalau crash, kecuali di-stop manual |
| `volumes: postgres_data` | Data tersimpan di volume Docker, tidak hilang kalau container di-restart |

### Jalankan Container PostgreSQL

Setelah `docker-compose.yml` sudah ada, jalankan perintah ini di terminal:

```bash
docker compose up -d
```

Perintah ini akan:
1. **Download PostgreSQL image** dari Docker Hub (kalau belum ada)
2. **Membuat dan menjalankan container** `support-desk-db` di background

Verifikasi container sudah running:

```bash
docker ps
```

Harus muncul container `support-desk-db` dengan status `Up`.

### Database Otomatis Ter-create

Karena kita sudah set `POSTGRES_DB: support_desk` di `docker-compose.yml`, image resmi PostgreSQL akan **otomatis membuat database `support_desk`** saat container pertama kali dijalankan (inisialisasi awal).

Verifikasi database sudah ada:

```bash
docker exec -it support-desk-db psql -U postgres -c "\l"
```

Harus ada database `support_desk` dalam list.

> **Catatan:** `POSTGRES_DB` hanya berefek saat **inisialisasi pertama** (saat volume masih kosong). Kalau kamu sudah pernah menjalankan container sebelumnya tanpa `POSTGRES_DB`, kamu perlu hapus volume dulu (`docker compose down -v`) lalu jalankan ulang, atau buat database manual:
>
> ```bash
> docker exec -it support-desk-db createdb -U postgres support_desk
> ```

Selesai! Database `support_desk` sudah siap. Tables akan dibuat di Bab 24 via migrations.

### Perintah Lainnya

```bash
docker compose down              # Stop container (data tetap aman)
docker compose down -v           # Stop container + hapus semua data
docker compose logs postgres     # Lihat logs database
```

---

## File .env dan Keamanan

File `.env` menyimpan konfigurasi sensitif: URL database, secret JWT, dan lain-lain. File ini disimpan di direktori project tapi tidak boleh masuk ke Git.

### Langkah 2: Buat file `.env`

**Lokasi:** Buat file baru bernama `.env` di **root folder project** (sama level dengan `Cargo.toml` dan `docker-compose.yml`)

**Isi file:**

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/support_desk
JWT_SECRET=rahasia_jwt_super_panjang_dan_aman
PORT=3000
```

Format `DATABASE_URL`:

```
postgres://[user]:[password]@[host]:[port]/[nama_database]
```

### Jangan Commit .env!

Ini **sangat penting**. Kalau file `.env` ter-push ke GitHub, siapapun bisa lihat password database.

Pastikan `.gitignore` sudah berisi:

```
# Environment variables
.env
.env.local
.env.*.local
```

### Langkah 3: Buat file `.env.example`

**Lokasi:** Buat file baru bernama `.env.example` di **root folder project**

**Isi file:**

File `.env.example` adalah template yang aman untuk di-commit. Tidak berisi nilai asli, hanya placeholder:

```
DATABASE_URL=postgres://user:password@localhost:5432/dbname
JWT_SECRET=isi_dengan_secret_panjang_dan_random
PORT=3000
```

File `.env.example` aman untuk di-commit karena tidak ada nilai sensitif.

---

## Membaca Environment Variable

Untuk membaca file `.env` di Rust, pakai library `dotenvy`. 

**Good news:** Dependency `dotenvy` sudah di-include di `Cargo.toml` dari Bab 18, jadi tidak perlu tambah lagi.

Cara pakainya di `main.rs`:

```rust
dotenvy::dotenv().ok();
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL harus di-set di .env");
```

`dotenvy::dotenv().ok()` memuat file `.env` ke environment. Pakai `.ok()` supaya tidak panic kalau file tidak ada. Di production, env var biasanya sudah di-set langsung oleh sistem tanpa perlu file `.env`. `std::env::var("DATABASE_URL")` membaca nilainya dan mengembalikan `Result`. `.expect(...)` memastikan program berhenti dengan pesan error yang jelas kalau env var tidak ada.

**Environment variable** adalah variabel yang di-set di luar program, bukan di dalam kode. Cocok untuk konfigurasi yang beda antara development dan production.

---

## Connection Pool

Membuka koneksi baru ke database setiap request itu mahal, karena butuh waktu untuk handshake dan autentikasi. Kalau setiap request membuka koneksi baru, aplikasi akan lambat.

**Connection pool** menyelesaikan ini dengan cara menyiapkan sejumlah koneksi di awal, lalu meminjamkannya ke request yang datang, dan mengembalikannya setelah selesai. Koneksi tidak dibuat ulang dari nol setiap saat.

[ILUSTRASI: Pool berisi 10 "koneksi" yang tersedia. Request 1, 2, 3 masing-masing mengambil satu koneksi dari pool, memakainya, lalu mengembalikannya]

### Langkah 4: Buat file `src/db.rs`

**Good news:** Dependency SQLx sudah di-include di `Cargo.toml` dari Bab 18, jadi tidak perlu tambah lagi.

**Lokasi:** Buat file baru bernama `db.rs` di folder `src/` (sama level dengan `main.rs`)

**Isi file:**

```rust
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Gagal koneksi ke database")
}
```

`max_connections(10)` menetapkan maksimal 10 koneksi simultan, sesuaikan dengan kebutuhan dan kapasitas server database. `connect(database_url)` adalah operasi async, jadi perlu `.await`. Tanpa `.await`, yang kamu dapat bukan pool yang siap pakai, tapi sebuah Future yang belum dijalankan — dan compiler akan menolak dengan error tipe.

---

## AppState — Berbagi Pool ke Semua Handler

Axum punya mekanisme bernama **State** untuk berbagi data ke semua handler. Buat struct `AppState` yang menyimpan pool:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}
```

`#[derive(Clone)]` diperlukan karena Axum membutuhkan state yang bisa di-clone. Setiap request berjalan di task yang berbeda. `PgPool` sendiri sudah `Clone`-safe: clone-nya tidak membuat pool baru, hanya menambah reference ke pool yang sama.

---

## Kode main.rs yang Diperbarui

### Langkah 5: Update `src/main.rs`

**File yang diupdate:** `src/main.rs` (file dari Bab 22)

**Yang ditambah:**

1. Tambahkan module import di **paling atas** file:

```rust
mod db;
```

2. Tambahkan imports baru:

```rust
use sqlx::PgPool;
use db::create_pool;
```

3. Tambahkan struct `AppState` sebelum `TicketFilters` struct:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}
```

4. Update `src/main.rs` — Ganti bagian `async fn main()` dengan kode di bawah:

**Lokasi:** Update fungsi `main()` di `src/main.rs` (ganti seluruh fungsi `main`)

**Kode baru:**

```rust
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

    // Setup router dengan semua routes
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/tickets", ticket_routes())
        .nest("/users", user_routes());

    // Baca PORT dari environment, default 3000 jika tidak ada
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server berjalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
```

**Penjelasan:**

| Kode | Artinya |
|------|---------|
| `dotenvy::dotenv().ok()` | Load file `.env` ke environment |
| `std::env::var("DATABASE_URL")` | Baca DATABASE_URL dari environment |
| `create_pool(&database_url).await` | Buat connection pool ke database (operasi async) |
| `sqlx::query("SELECT 1").execute(&pool)` | Test koneksi dengan query sederhana |
| `std::env::var("PORT").unwrap_or_else(...)` | Baca PORT dari env, default "3000" jika tidak ada |

**Catatan:** Struct `AppState` didefinisikan di awal file tapi belum dipakai. Ini akan dipakai di Bab 24+ ketika handler perlu mengakses pool secara langsung melalui `State` extractor Axum.

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan latihan Bab 23, folder struktur dan file baru harus seperti ini:

```
support-desk/
├── Cargo.toml                  ← sqlx dependency sudah ada (dari Bab 18)
├── docker-compose.yml          ← NEW (dari latihan #6)
├── .env                        ← NEW (dari latihan #2, jangan commit!)
├── .env.example                ← NEW (dari latihan #5, safe to commit)
├── .gitignore                  ← UPDATE (tambah .env)
├── src/
│   ├── main.rs                 ← UPDATE (tambah mod db, AppState, dotenvy)
│   ├── db.rs                   ← NEW (dari latihan #3)
│   ├── models/
│   ├── dto/
│   └── common/
```

**File: `.env`** (JANGAN COMMIT!)
```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/support_desk
JWT_SECRET=rahasia_jwt_super_panjang_dan_aman_minimal_32_karakter
PORT=3000
```

**File: `.env.example`** (Safe to commit)
```
DATABASE_URL=postgres://user:password@localhost:5432/support_desk
JWT_SECRET=isi_dengan_secret_panjang_dan_random_minimal_32_karakter
PORT=3000
```

**File: `docker-compose.yml`**
```yaml
services:
  postgres:
    image: postgres:16-alpine
    container_name: support-desk-db
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: support_desk
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

**File: `src/db.rs`**
```rust
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Gagal koneksi ke database")
}
```

**Perubahan di `src/main.rs`:**

1. **Tambah import di paling atas (setelah mod declarations):**
   ```rust
   mod db;
   use sqlx::PgPool;
   use db::create_pool;
   ```

2. **Tambah struct AppState (sebelum TicketFilters struct):**
   ```rust
   #[derive(Clone)]
   pub struct AppState {
       pub db: PgPool,
   }
   ```

3. **Ganti seluruh fungsi `async fn main()`** dengan kode di Langkah 5 di atas — yang penting:
   - Load `.env` dengan `dotenvy::dotenv().ok()`
   - Baca `DATABASE_URL` dari environment
   - Buat pool dengan `create_pool(&database_url).await`
   - Test koneksi dengan `sqlx::query("SELECT 1")`
   - Setup router dan jalankan server

**Update `.gitignore`** — Tambahkan:
```
.env
.env.local
.env.*.local
```

---

## ⚠️ PENTING: Database Harus Dijalankan Dulu

Satu kesalahan umum: mencoba menjalankan `cargo run` tanpa database aktif. Hasilnya error `connection refused` atau `database not found`.

**Urutan yang BENAR:**

### Step 1: Start Database Container

```bash
docker compose up -d
```

Ini menjalankan PostgreSQL di background. Tunggu 2-3 detik agar container siap menerima koneksi.

Verifikasi container sudah running:

```bash
docker ps
```

Output harus menampilkan container `support-desk-db` dengan status `Up`:

```
CONTAINER ID   IMAGE              NAMES              STATUS
abc123def456   postgres:16-alpine support-desk-db   Up 2 seconds
```

### Step 2: Buat atau Verifikasi File `.env`

```bash
cat .env
```

Harus berisi (sesuai dengan `docker-compose.yml`):

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/support_desk
JWT_SECRET=rahasia_jwt_super_panjang_dan_aman
PORT=3000
```

### Step 3: Build dan Run Server

Baru setelah database running, jalankan:

```bash
cargo build
cargo run
```

Output harus:

```
Server berjalan di http://0.0.0.0:3000
```

### Step 4: Test Koneksi

Di terminal lain, test endpoint:

```bash
curl http://localhost:3000/health
```

Harus return: `OK`

---

## Troubleshooting

| Error | Penyebab | Solusi |
|-------|---------|--------|
| `connection refused` | Database tidak running | Jalankan `docker compose up -d` |
| `database "support_desk" does not exist` | Database name tidak match `.env` | Pastikan `.env` pakai `DATABASE_URL=postgres://...@localhost:5432/support_desk` |
| `password authentication failed` | Username/password tidak match | Pastikan `.env` pakai user `postgres` dan password `postgres` (sesuai `docker-compose.yml`) |
| `server berjalan` tapi `curl` error | Firewall blokir port 3000 | Cek dengan `lsof -i :3000` atau coba port lain di `.env` |

**Database sudah connected, siap untuk membuat tables via migrations di Bab 24.**

---

## Ringkasan File yang Dibuat/Diupdate di Bab 23

| File | Status | Deskripsi |
|------|--------|-----------|
| `.env` | 🆕 BARU | Environment variables untuk database URL, JWT secret, PORT |
| `.env.example` | 🆕 BARU | Template .env yang aman untuk di-commit |
| `docker-compose.yml` | 🆕 BARU | Konfigurasi PostgreSQL container |
| `src/db.rs` | 🆕 BARU | Connection pool initialization dengan SQLx |
| `src/main.rs` | ✏️ DIUPDATE | Tambah `mod db;`, AppState struct, database initialization |
| `.gitignore` | ✏️ DIUPDATE | Tambah `.env` ke ignore list |

**Total: 6 file baru/updated**

**Penting:** 
- ✅ `.env` **JANGAN di-commit** ke Git (sudah di-`.gitignore`)
- ✅ `.env.example` aman untuk di-commit
- ✅ `docker-compose.yml` aman untuk di-commit

---

## Common Mistakes ❌

### ❌ Mistake 1: Jalankan `cargo run` tanpa database

```bash
cargo run
# ERROR: connection refused
```

**Solusi:** Pastikan `docker compose up -d` sudah dijalankan dulu.

### ❌ Mistake 2: `.env` tidak ada atau config salah

```bash
cargo run
# ERROR: DATABASE_URL harus di-set di .env
```

**Solusi:** Buat file `.env` dengan isi:

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/support_desk
JWT_SECRET=rahasia_jwt_super_panjang_dan_aman
PORT=3000
```

### ❌ Mistake 3: Container tidak running tapi kira-kira running

```bash
docker compose up -d
cargo run
# Terjadi konek beberapa detik kemudian...
# ERROR: database "support_desk" does not exist
```

**Solusi:** Container butuh beberapa detik untuk fully ready. Tunggu 3-5 detik sebelum run server, atau cek status:

```bash
docker compose logs postgres | tail -20
# Cari tulisan "database system is ready to accept connections"
```

### ❌ Mistake 4: Lupa `.env` tidak boleh di-commit

```bash
git add .
git commit -m "add database config"
git push
# ⚠️ Password database sudah tersimpan di GitHub public!
```

**Solusi:** `.gitignore` harus punya:

```
.env
.env.local
```

Cek dengan `git status`, file `.env` tidak boleh muncul.

---

## Latihan

1. **Jalankan container Docker** dengan perintah di atas, lalu verifikasi dengan `docker ps`.

2. **Buat file `.env`** dan pastikan `.gitignore` sudah mengecualikannya. Cek dengan `git status`, file `.env` tidak boleh muncul sebagai untracked.

3. **Buat file `src/db.rs`** dengan fungsi `create_pool`, lalu panggil dari `main.rs`. Coba jalankan server. Kalau tidak ada error, koneksi berhasil.

4. **Eksplorasi**: coba ubah `max_connections` ke angka berbeda (misal 5 atau 20). Kapan menurutmu nilai ini perlu dinaikkan?

5. **Tantangan**: buat file `.env.example` yang bisa di-commit ke Git, berisi semua key yang dibutuhkan tapi tanpa nilai sensitif.

6. **Docker Compose**: buat file `docker-compose.yml` di root project menggunakan template di atas, lalu jalankan database dengan `docker compose up -d`. Verifikasi dengan `docker ps`, pastikan container `support-desk-db` berstatus `Up`.
