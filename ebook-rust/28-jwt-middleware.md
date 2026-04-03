# Bab 28: Autentikasi — JWT dan Middleware

Bayangkan kamu masuk ke sebuah gedung kantor. Satpam minta KTP, verifikasi data kamu, lalu kasih kartu akses (badge) yang berlaku satu minggu. Setiap kali mau masuk ruangan, kamu tinggal tempel badge itu; satpam nggak perlu telepon HRD lagi buat cek siapa kamu. Semua info sudah ada di badge.

JWT (JSON Web Token) persis kayak badge itu. Sekali login berhasil, server kasih token. Setiap request berikutnya, kamu bawa token itu di header, dan server langsung tahu kamu siapa tanpa perlu query database lagi.

[ILUSTRASI: diagram alur login → server generate token → client simpan token → client kirim token di setiap request → server verifikasi token]

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 27:**
- Register handler dengan password hashing (argon2) dan validasi sudah lengkap
- Login handler mengembalikan token (placeholder dari Bab 27)

**State Sebelumnya:**
Dari Bab 27, folder `src/services/` punya `auth_service.rs` dengan `AuthService` yang handle register dan login. Handler auth di `src/handlers/` sudah siap.

---

## Apa Itu JWT?

JWT punya tiga bagian, dipisah dengan titik:

```
Header.Payload.Signature
```

- **Header:** algoritma enkripsi yang dipakai (biasanya `HS256`)
- **Payload:** data yang kamu simpan, misalnya user id, email, role
- **Signature:** tanda tangan digital, dibuat dari header + payload + secret key

Karena ada signature, kalau ada yang coba ubah payload-nya, signaturenya langsung invalid. Token nggak bisa dipalsuin tanpa tahu secret key.

Beberapa istilah penting: *claims* adalah data yang disimpan di payload JWT, `sub` adalah konvensi standar untuk menyimpan id user ("subject"), dan `exp` adalah timestamp kapan token expired.

**jsonwebtoken** sudah ada di `Cargo.toml` (versi 9). Dua operasi utama yang kita pakai:

```rust
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};

// Generate token (saat login)
let claims = Claims { /* ... */ };
let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?;

// Verify token (saat request auth diperlukan)
let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())?;
let claims = token_data.claims;
```

---

## Claims Struct

Pertama, definisikan struct untuk data yang akan disimpan di dalam JWT:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user id sebagai string
    pub email: String,
    pub role: String,     // "admin", "agent", atau "customer"
    pub exp: usize,       // expiry timestamp (unix seconds)
}
```

Struct ini disimpan di `src/services/auth_service.rs`, bersama dengan AuthService.

---

## Generate Token

Update `AuthService` di `src/services/auth_service.rs` dengan dua perubahan:

**1. Tambah `jwt_secret` field dan update constructor:**

```rust
#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, jwt_secret: String) -> Self {
        Self { user_repo, jwt_secret }
    }

    // ... rest of methods
}
```

**2. Tambah method `generate_token`:**

```rust
pub fn generate_token(&self, user: &User) -> Result<String, AppError> {
    let expiry = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 7 * 24 * 3600; // 7 hari

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: format_role(&user.role),
        exp: expiry,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}
```

Yang terjadi di sini: hitung waktu expired (sekarang + 7 hari dalam Unix timestamp), buat struct `Claims` dengan data user, lalu `encode` menghasilkan string JWT. `Header::default()` pakai algoritma `HS256` secara default.

**3. Update `login()` method** untuk menggunakan `generate_token`:

Ganti baris placeholder:
```rust
// Lama:
Ok("token_placeholder".to_string())

// Baru:
let token = self.generate_token(&user)?;
Ok(token)
```

---

## Verify Token

Tambah sebagai free function (public, bukan method) agar bisa digunakan di middleware:

```rust
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Token tidak valid atau expired".to_string()))
}
```

`Validation::default()` sudah otomatis cek signature dan memastikan token belum expired (field `exp`). Kalau salah satu gagal, langsung return `Unauthorized`.

---

## Helper: format_role

Karena `Claims.role` adalah `String`, tapi `User.role` adalah `UserRole` enum, kita butuh konversi:

```rust
fn format_role(role: &UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Agent => "agent".to_string(),
        UserRole::Customer => "customer".to_string(),
    }
}
```

---

## Custom Extractor: AuthUser

Ini bagian paling elegan dari Axum. Daripada parse header Authorization secara manual di setiap handler, kita buat **custom extractor**, di mana Axum akan otomatis jalankan logika auth sebelum handler dieksekusi.

*Extractor* adalah tipe yang implement trait `FromRequestParts`. Axum memanggil fungsi ini otomatis ketika tipe tersebut ada di parameter handler.

[ILUSTRASI: diagram satpam (FromRequestParts) berdiri di depan pintu ruangan (handler) — semua tamu harus lewat satpam dulu sebelum masuk]

Buat folder dan file baru: `src/middleware/auth.rs`

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::common::AppError;
use crate::services::{Claims, verify_token};

pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Token diperlukan".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                AppError::Unauthorized("Format token salah, gunakan: Bearer <token>".to_string())
            })?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET tidak dikonfigurasi".to_string()))?;

        let claims = verify_token(token, &jwt_secret)?;
        Ok(AuthUser(claims))
    }
}
```

Alur kerjanya: ambil header `Authorization` (kalau tidak ada, return 401), strip prefix `"Bearer "` (kalau format salah, return 401), verifikasi token (kalau invalid/expired, return 401), lalu return `AuthUser(claims)` kalau semua OK.

Catat: **NO `#[async_trait]` attribute** di Axum 0.8 — trait method sudah async secara native.

---

## Pakai AuthUser di Handler

Tinggal tambahkan `AuthUser` sebagai parameter di handler manapun yang butuh autentikasi:

```rust
use crate::middleware::AuthUser;
use axum::Json;
use serde_json::{json, Value};

async fn get_my_profile(
    AuthUser(claims): AuthUser,
) -> Result<Json<Value>, crate::common::AppError> {
    Ok(Json(json!({
        "success": true,
        "data": {
            "user_id": claims.sub,
            "email": claims.email,
            "role": claims.role,
        }
    })))
}
```

Axum otomatis panggil `AuthUser::from_request_parts` sebelum handler jalan. Kalau auth gagal, handler bahkan tidak dieksekusi; langsung balik error response. `AuthUser(claims)` adalah pattern destructuring, kita langsung ambil `Claims` dari dalam wrapper.

---

## Update AppState dan main.rs

### Update AppState

Di `src/main.rs`, tambahkan field `jwt_secret`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
    pub auth_service: AuthService,
    pub jwt_secret: String,  // ← TAMBAH
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {  // ← UBAH SIGNATURE
        let user_repo = UserRepository::new(pool.clone());

        Self {
            user_repo: user_repo.clone(),
            ticket_repo: TicketRepository::new(pool.clone()),
            response_repo: ResponseRepository::new(pool.clone()),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            auth_service: AuthService::new(user_repo, jwt_secret.clone()),  // ← PASS jwt_secret
            jwt_secret,  // ← TAMBAH
            db: pool,
        }
    }
}
```

### Update main()

Di `main()` function, baca JWT_SECRET dari env dan pass ke AppState:

```rust
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
```

Pastikan `.env` kamu sudah punya `JWT_SECRET`:

```env
JWT_SECRET=rahasia-yang-panjang-dan-aman-minimal-32-karakter
```

---

## Ringkasan Alur

| Aksi | Endpoint | Langkah Kunci |
|------|----------|---------------|
| Register | POST /auth/register | Validasi → cek email → hash password → generate JWT |
| Login | POST /auth/login | Cari user → verifikasi hash → **generate JWT token** |
| Akses resource protected | GET /api/resource (dengan token di header) | Extract header → verify token → extract claims → handler execute |

---

## Latihan

1. **Buat endpoint `/me`** yang return profil user yang sedang login menggunakan `AuthUser` extractor. Test dengan curl: `curl -H "Authorization: Bearer <token>" http://localhost:3000/me`.

2. **Role-based guard:** buat extractor `AdminUser` yang sama seperti `AuthUser` tapi tambahkan pengecekan `claims.role == "admin"`. Kalau bukan admin, return `AppError::Forbidden`.

3. **Eksplorasi:** Coba decode JWT kamu di [jwt.io](https://jwt.io), paste token yang di-generate dan lihat isi payload-nya. Perhatikan bahwa payload *bisa dibaca* tanpa secret, jadi jangan simpan password atau data sensitif di JWT.

4. **Refresh token (opsional):** Riset perbedaan access token (expire pendek, misal 15 menit) vs refresh token (expire panjang, misal 30 hari). Kapan pola ini lebih baik dari token 7 hari yang kita pakai sekarang?

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan latihan Bab 28, struktur folder harus seperti ini:

```
src/
├── middleware/                 ← NEW FOLDER
│   ├── mod.rs                  ← NEW
│   └── auth.rs                 ← NEW: AuthUser extractor
├── services/
│   ├── mod.rs                  ← UPDATED: re-export Claims, verify_token
│   └── auth_service.rs         ← UPDATED: add Claims, generate_token, verify_token
├── handlers/
├── repositories/
├── models/
├── dto/
├── common/
├── db.rs
└── main.rs                      ← UPDATED: jwt_secret in AppState, mod middleware
```

### File: `src/services/auth_service.rs` — UPDATED

Tambah di atas `AuthService` struct:

```rust
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}
```

Update `AuthService` struct:
```rust
#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, jwt_secret: String) -> Self {
        Self { user_repo, jwt_secret }
    }

    pub fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 7 * 24 * 3600;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: format_role(&user.role),
            exp: expiry,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    // register() tetap sama

    pub async fn login(&self, dto: LoginDto) -> Result<String, AppError> {
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Email atau password salah".to_string()))?;

        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Argon2::default()
            .verify_password(dto.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Email atau password salah".to_string()))?;

        // Generate token JWT
        let token = self.generate_token(&user)?;
        Ok(token)
    }
}
```

Tambah helper functions di akhir:

```rust
fn format_role(role: &UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Agent => "agent".to_string(),
        UserRole::Customer => "customer".to_string(),
    }
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Token tidak valid atau expired".to_string()))
}
```

### File: `src/services/mod.rs` — UPDATED

```rust
pub mod auth_service;

pub use auth_service::{AuthService, Claims, verify_token};
```

### File: `src/middleware/mod.rs` — BARU

```rust
pub mod auth;

pub use auth::AuthUser;
```

### File: `src/middleware/auth.rs` — BARU

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::common::AppError;
use crate::services::{Claims, verify_token};

pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Token diperlukan".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                AppError::Unauthorized("Format token salah, gunakan: Bearer <token>".to_string())
            })?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET tidak dikonfigurasi".to_string()))?;

        let claims = verify_token(token, &jwt_secret)?;
        Ok(AuthUser(claims))
    }
}
```

### File: `src/main.rs` — UPDATED

Tambah di module declarations:
```rust
mod middleware;
```

Update `AppState`:
```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
    pub auth_service: AuthService,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = UserRepository::new(pool.clone());

        Self {
            user_repo: user_repo.clone(),
            ticket_repo: TicketRepository::new(pool.clone()),
            response_repo: ResponseRepository::new(pool.clone()),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            auth_service: AuthService::new(user_repo, jwt_secret.clone()),
            jwt_secret,
            db: pool,
        }
    }
}
```

Update `main()`:
```rust
let jwt_secret = std::env::var("JWT_SECRET")
    .expect("JWT_SECRET harus di-set di .env");

let state = AppState::new(pool, jwt_secret);
```

**Verifikasi:**
```bash
cargo build
# Harus 0 errors
```

---

## Kesimpulan Bab 28

Bab ini mengupgrade dari placeholder token menjadi JWT yang sesungguhnya. Key insights:

1. **JWT Structure:** Header.Payload.Signature dengan claims di payload
2. **Token Generation:** Encrypt dengan secret key saat login
3. **Token Verification:** Decrypt dan validate signature + expiry saat handler
4. **Axum Extractors:** Custom extractor `AuthUser` otomatis validate sebelum handler jalan
5. **No async_trait needed:** Axum 0.8 sudah support async traits native

**Status Build:** ✅ 0 errors

Bab berikutnya: **Integration Handler ke Repository** — Menggunakan repositories di dalam handlers untuk operasi CRUD dengan autentikasi
