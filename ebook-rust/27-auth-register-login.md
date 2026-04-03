# Bab 27: Autentikasi - Register dan Login

Bayangkan kamu punya loker di gym. Loker itu dikunci dengan kode kombinasi. Kalau ada orang lain minta kode kamu, kamu nggak mau kasih, kan? Tapi gimana kalau petugas gym butuh verifikasi bahwa kamu pemilik loker itu tanpa kamu harus kasih kode aslinya?

Itulah inti dari autentikasi modern. Sistem kita perlu tahu "ini beneran kamu", tapi tanpa menyimpan password aslimu. Di bab ini, kita bangun dua fitur fundamental: **register** (daftar akun baru) dan **login** (masuk ke sistem).

[ILUSTRASI: Dua jalur paralel — "Register" (pengguna baru → hash password → simpan ke DB → selesai) dan "Login" (pengguna lama → cari email → verifikasi hash → beri token)]

JWT (token yang kamu dapatkan setelah login) akan dibahas detail di bab 28. Di bab ini, login akan kembalikan placeholder dulu, dan kita fokus ke bagian yang lebih kritis: **password hashing** dan **alur autentikasi**.

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 26:**
- Dashboard queries dan response queries sudah tercakup di "Hasil Akhir Bab 26"
- `ResponseRepository`, `DashboardRepository`, dan `DashboardStats` sudah siap

**State Sebelumnya:**
- Folder `src/repositories/` lengkap dengan 4 repositories (user, ticket, response, dashboard)
- `AppState` include semua repositories
- `src/dto/user_dto.rs` sudah punya `RegisterDto` dan `LoginDto` dengan validation attributes

Verifikasi:
```bash
cargo build
# Harus 0 errors
```

---

## Kenapa Password Tidak Boleh Disimpan Plaintext

**Plaintext** artinya teks apa adanya, tanpa diubah. Kalau kamu simpan password `rahasia123` langsung ke database, itulah plaintext.

Masalahnya: kalau database bocor (dan ini terjadi terus di dunia nyata), semua password langsung terbaca. Hacker bisa login sebagai siapapun.

Analoginya begini. Menyimpan password plaintext itu seperti memfotokopi KTP asli pengguna, lalu menyimpannya di laci kantor. Kalau laci dibobol, semua data identitas langsung ketahuan. Menyimpan password dalam bentuk hash lebih seperti menyimpan sidik jari pengguna. Kalau data bocor, hacker dapat sidik jari, tapi sidik jari nggak bisa dikembalikan jadi identitas asli. Kamu tetap bisa verifikasi "apa sidik jari ini cocok?" tanpa perlu tahu orangnya siapa.

**Hash** adalah fungsi satu arah: `password → hash` mudah, tapi `hash → password` hampir mustahil. Setiap kali login, kita hash password yang diketik, lalu bandingkan dengan hash yang tersimpan.

---

## argon2: Password Hashing yang Aman

Ada banyak algoritma hashing. **Argon2** adalah yang direkomendasikan saat ini karena dirancang lambat secara sengaja, sehingga brute force (coba ribuan kombinasi) jadi sangat mahal secara komputasi.

`argon2` sudah ada di `Cargo.toml` (versi 0.5). Dua operasi utama yang kita pakai:

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

// Hash password (saat register)
let salt = SaltString::generate(&mut OsRng);
let hashed = Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .map_err(|e| AppError::Internal(e.to_string()))?
    .to_string();

// Verifikasi (saat login)
let parsed_hash = PasswordHash::new(&stored_hash)
    .map_err(|e| AppError::Internal(e.to_string()))?;
Argon2::default().verify_password(input.as_bytes(), &parsed_hash)
    .map_err(|_| AppError::Unauthorized("Email atau password salah".to_string()))?;
```

**Salt** adalah string acak yang ditambahkan sebelum hashing. Ini mencegah dua pengguna dengan password sama menghasilkan hash yang sama, sehingga hacker tidak bisa pakai "rainbow table" (daftar hash yang sudah dihitung sebelumnya).

---

## AuthService

`AuthService` bertanggung jawab atas logika register dan login. Dia butuh akses ke database lewat `UserRepository`, sesuai pola yang sudah kita bangun di bab 25.

```
src/
├── services/
│   ├── mod.rs
│   └── auth_service.rs
```

### Struktur Dasar

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::{
    common::AppError,
    dto::{LoginDto, RegisterDto},
    models::User,
    models::UserRole,
    repositories::UserRepository,
};

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }
}
```

`AuthService::new` menerima `UserRepository` sebagai argumen, yaitu **dependency injection**. Service tidak membuat repo sendiri; dia menerima dari luar. Itulah kenapa mudah diganti saat testing.

Catat: `#[derive(Clone)]` di struct. Karena `AuthService` akan disimpan di `AppState` (yang juga derive `Clone`), service ini harus Clone-able juga.

---

## Register: Daftar Akun Baru

Alur register ada empat langkah: validasi input, cek apakah email sudah dipakai, hash password, lalu simpan user baru ke database.

```rust
pub async fn register(&self, dto: RegisterDto) -> Result<User, AppError> {
    // Langkah 1: Validasi input
    dto.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Langkah 2: Cek email sudah ada
    if self.user_repo.find_by_email(&dto.email).await?.is_some() {
        return Err(AppError::Conflict("Email sudah digunakan".to_string()));
    }

    // Langkah 3: Hash password
    let salt = SaltString::generate(&mut OsRng);
    let hashed = Argon2::default()
        .hash_password(dto.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_string();

    // Langkah 4: Parse role dari string (dari DTO) ke enum
    let role = parse_role(&dto.role)?;

    // Langkah 5: Simpan ke DB
    let user = self
        .user_repo
        .create(&dto.name, &dto.email, &hashed, role)
        .await?;

    Ok(user)
}
```

**Penting:** `dto.role` adalah `String` (dari input user), tapi `UserRepository::create()` butuh `UserRole` enum. Kita konversi via `parse_role()` helper.

`AppError::Conflict` menghasilkan HTTP 409, yang memberi tahu client bahwa data sudah ada. `map_err(|e| AppError::Internal(e.to_string()))` mengonversi error dari library argon2 ke `AppError` kita (simple string error, bukan `anyhow`).

---

## Login: Masuk ke Sistem

Alur login ada tiga langkah: cari user berdasarkan email, verifikasi password yang dimasukkan dengan hash yang tersimpan, lalu generate token.

```rust
pub async fn login(&self, dto: LoginDto) -> Result<String, AppError> {
    // Langkah 1: Validasi input
    dto.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Langkah 2: Cari user
    let user = self
        .user_repo
        .find_by_email(&dto.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Email atau password salah".to_string()))?;

    // Langkah 3: Verifikasi password
    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Argon2::default()
        .verify_password(dto.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Email atau password salah".to_string()))?;

    // Langkah 4: Return token (sementara, JWT di Bab 28)
    Ok("token_placeholder".to_string())
}
```

Perhatikan satu hal penting: pesan error untuk "email tidak ditemukan" dan "password salah" **sengaja dibuat sama**: `"Email atau password salah"`. Ini praktik keamanan standar. Kalau pesannya berbeda, hacker bisa tahu email mana yang terdaftar, lalu fokus mencoba password-nya saja.

`.ok_or_else()` mengubah `Option<User>` menjadi `Result<User, AppError>`. Kalau `find_by_email` kembalikan `None` (user tidak ada), langsung return `Unauthorized`.

[ILUSTRASI: Flowchart login — mulai dari "Cari user by email" → percabangan "Ada?" → Tidak: return error "Email atau password salah" → Ya: "Verifikasi password" → percabangan "Cocok?" → Tidak: return error yang sama → Ya: "Return token"]

---

## Helper: Konversi Role String ke Enum

```rust
fn parse_role(role: &str) -> Result<UserRole, AppError> {
    match role {
        "customer" => Ok(UserRole::Customer),
        "agent" => Ok(UserRole::Agent),
        _ => Err(AppError::BadRequest(
            "Role harus 'customer' atau 'agent'".to_string(),
        )),
    }
}
```

Helper ini digunakan di `register()` untuk konversi `dto.role: String` ke `UserRole` enum yang butuh `UserRepository::create()`.

---

## AuthHandler — Route HTTP

Handler bertugas menerima request HTTP, memanggil service, dan memformat response. Buat file baru:

```
src/
├── handlers/
│   ├── mod.rs
│   └── auth_handler.rs
```

### File: `src/handlers/auth_handler.rs`

```rust
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use crate::{dto::{LoginDto, RegisterDto}, AppState};

/// Handler untuk register (POST /auth/register)
pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), crate::common::AppError> {
    let user = state.auth_service.register(dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": user
        })),
    ))
}

/// Handler untuk login (POST /auth/login)
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let token = state.auth_service.login(dto).await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "token": token
        }
    })))
}
```

**Penjelasan:**
- `State(state): State<AppState>` mengekstrak AppState dari router. Router harus punya state ini (dilihat nanti di setup router).
- `Json(dto): Json<RegisterDto>` automatically deserialize JSON body ke DTO. Jika gagal, Axum return 400 Bad Request otomatis.
- Register mengembalikan `StatusCode::CREATED` (HTTP 201) karena memang membuat resource baru.
- Login mengembalikan 200 OK (default Axum).
- `state.auth_service.register(dto)` dan `state.auth_service.login(dto)` memanggil service methods.
- `?` operator mempropagate error ke Axum's error handler, yang otomatis konversi ke HTTP response via `AppError`'s `IntoResponse` impl.

---

## Update AppState dan Router di main.rs

### Update AppState

Di `src/main.rs`, tambahkan `auth_service` field ke `AppState`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
    pub auth_service: AuthService,  // ← TAMBAH INI
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let user_repo = UserRepository::new(pool.clone());

        Self {
            user_repo: user_repo.clone(),
            ticket_repo: TicketRepository::new(pool.clone()),
            response_repo: ResponseRepository::new(pool.clone()),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            auth_service: AuthService::new(user_repo),  // ← TAMBAH INI
            db: pool,
        }
    }
}
```

### Setup Router dengan State

Di `src/main.rs`, ganti router setup (setelah migrations):

```rust
// Buat AppState dengan semua repositories dan services
let state = AppState::new(pool);

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
```

**Penjelasan router state:**
- `Router::new().route(...).with_state(state)` membuat `Router<AppState>` (router yang mempunyai state)
- Handler yang memakai `State<AppState>` hanya bisa di-add ke router dengan state ini
- Kita membangun auth routes sebagai sub-router dengan state, lalu `.merge()` ke main router
- `.merge()` menggabungkan dua routers; semua route dari auth_routes ditambahkan ke main router
- Existing ticket_routes() dan user_routes() tidak memakai state (mereka `Router<()>`), tapi `.nest()` masih bisa merge mereka

---

## Ringkasan Alur

| Aksi | Endpoint | Langkah Kunci | Error Response |
|------|----------|---------------|---|
| Register | POST /auth/register | Validasi input → cek email → hash password → parse role → simpan | 400 (validation), 409 (email ada), 500 (server error) |
| Login | POST /auth/login | Validasi input → cari user → verifikasi hash → return token | 400 (validation), 401 (email/password), 500 (server error) |

---

## Latihan

1. **Validasi role di RegisterDto:** Sudah ada `#[validate(custom(function = "validate_role"))]` di `src/dto/user_dto.rs`. Periksa bahwa hanya "customer" dan "agent" yang diterima. Apa yang happen kalau user kirim role yang tidak valid?

2. **Auto-login setelah register:** Di handler `register`, setelah berhasil simpan user, coba langsung panggil `login` dengan credential yang sama dan return token sekaligus, sehingga user tidak perlu login lagi setelah register. Apa pro-cons dari pendekatan ini?

3. **Error logging:** Saat ini error dari argon2 atau database hanya dikonversi jadi string generic. Implementasikan logging (simple: gunakan `eprintln!`) yang mencatat error sesungguhnya saat terjadi, untuk debugging, tanpa expose detail ke client.

4. **Unit test untuk AuthService:** Buat test untuk register (skenario sukses, email duplikat) dan login (skenario sukses, email tidak ada, password salah). Gunakan in-memory atau test database SQLite. *(Petunjuk: lihat bab sebelumnya tentang testing.)*

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan latihan Bab 27, struktur folder dan file harus seperti ini:

```
src/
├── services/               ← NEW FOLDER
│   ├── mod.rs              ← NEW
│   └── auth_service.rs     ← NEW
├── handlers/               ← NEW FOLDER
│   ├── mod.rs              ← NEW
│   └── auth_handler.rs     ← NEW
├── repositories/
├── models/
├── dto/
├── common/
├── db.rs
└── main.rs                 ← UPDATE: add modules, AppState, router
```

### File: `src/services/mod.rs` — BARU

```rust
pub mod auth_service;

pub use auth_service::AuthService;
```

### File: `src/services/auth_service.rs` — BARU

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::{
    common::AppError,
    dto::{LoginDto, RegisterDto},
    models::User,
    models::UserRole,
    repositories::UserRepository,
};
use validator::Validate;

/// Service untuk menangani autentikasi (register, login)
#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    /// Register: Daftar akun pengguna baru
    pub async fn register(&self, dto: RegisterDto) -> Result<User, AppError> {
        // Validasi input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Langkah 1: Cek apakah email sudah digunakan
        if self.user_repo.find_by_email(&dto.email).await?.is_some() {
            return Err(AppError::Conflict("Email sudah digunakan".to_string()));
        }

        // Langkah 2: Hash password
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Argon2::default()
            .hash_password(dto.password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        // Langkah 3: Parse role dari string ke enum
        let role = parse_role(&dto.role)?;

        // Langkah 4: Simpan user baru ke database
        let user = self
            .user_repo
            .create(&dto.name, &dto.email, &hashed, role)
            .await?;

        Ok(user)
    }

    /// Login: Masuk ke sistem
    pub async fn login(&self, dto: LoginDto) -> Result<String, AppError> {
        // Validasi input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Langkah 1: Cari user berdasarkan email
        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Email atau password salah".to_string()))?;

        // Langkah 2: Verifikasi password yang dimasukkan dengan hash yang tersimpan
        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Argon2::default()
            .verify_password(dto.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Email atau password salah".to_string()))?;

        // Langkah 3: Return token (placeholder untuk sekarang, JWT di Bab 28)
        Ok("token_placeholder".to_string())
    }
}

/// Helper: Konversi string role ke enum UserRole
fn parse_role(role: &str) -> Result<UserRole, AppError> {
    match role {
        "customer" => Ok(UserRole::Customer),
        "agent" => Ok(UserRole::Agent),
        _ => Err(AppError::BadRequest(
            "Role harus 'customer' atau 'agent'".to_string(),
        )),
    }
}
```

### File: `src/handlers/mod.rs` — BARU

```rust
pub mod auth_handler;
```

### File: `src/handlers/auth_handler.rs` — BARU

```rust
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use crate::{dto::{LoginDto, RegisterDto}, AppState};

/// Handler untuk register (POST /auth/register)
pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), crate::common::AppError> {
    let user = state.auth_service.register(dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": user
        })),
    ))
}

/// Handler untuk login (POST /auth/login)
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<serde_json::Value>, crate::common::AppError> {
    let token = state.auth_service.login(dto).await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "token": token
        }
    })))
}
```

### File: `src/main.rs` — UPDATE

Di bagian atas (lines 1-6), tambahkan modules:

```rust
mod models;
mod dto;
mod common;
mod db;
mod repositories;
mod services;          // ← TAMBAH
mod handlers;          // ← TAMBAH
```

Di imports (line 7-25 area), tambahkan:

```rust
use crate::services::AuthService;
use axum::routing::post;  // ← TAMBAH post untuk auth routes
```

Di `AppState` struct (lines 27-46), tambahkan field:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
    pub auth_service: AuthService,  // ← TAMBAH
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let user_repo = UserRepository::new(pool.clone());

        Self {
            user_repo: user_repo.clone(),
            ticket_repo: TicketRepository::new(pool.clone()),
            response_repo: ResponseRepository::new(pool.clone()),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            auth_service: AuthService::new(user_repo),  // ← TAMBAH
            db: pool,
        }
    }
}
```

Di `main()`, setelah migrations block, setup auth routes:

```rust
// Buat AppState dengan semua repositories dan services
let state = AppState::new(pool);

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
```

**Verifikasi:**
```bash
cargo build
# Harus 0 errors (warnings tentang unused code OK)

# Test register:
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name": "John", "email": "john@example.com", "password": "password123", "role": "customer"}'

# Test login:
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "john@example.com", "password": "password123"}'
```

---

## Kesimpulan Bab 27

Bab ini memperkenalkan **authentication core**: password hashing dengan argon2, register flow, dan login flow. Key insights:

1. **Password Hashing:** Tidak pernah simpan plaintext; gunakan salt + hash algorithm yang lambat (argon2)
2. **AuthService:** Dependency injection pattern — menerima UserRepository, bukan membuat sendiri
3. **Security practice:** Error message untuk login sengaja dibuat ambigu (email atau password salah, bukan specific)
4. **JWT placeholder:** Token di ch28 akan replace "token_placeholder"

**Status Build:** ✅ 0 errors

Bab berikutnya: **JWT Token Generation & Validation** - Upgrade placeholder token ke JWT yang aman dan dapat diverifikasi
