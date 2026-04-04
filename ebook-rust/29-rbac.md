# Bab 29: Role-Based Access Control (RBAC)

Bayangkan sebuah gedung kantor besar. Semua karyawan punya kartu akses, tapi nggak semua kartu bisa buka semua pintu. Security guard bisa masuk lobi dan ruang kontrol. Kasir bisa masuk kasir dan gudang kecil. Manager bisa masuk hampir semua ruangan. CEO bisa buka semua pintu, termasuk ruang server.

Inilah **Role-Based Access Control** (RBAC), yaitu sistem kontrol akses yang bergantung pada *peran* (role) seseorang, bukan identitas personalnya. Kita nggak nanya "siapa kamu?", tapi "kamu punya hak akses apa?"

Di project support desk kita, ada tiga peran: **Admin** yang bisa lihat semua dan kelola semua, **Agent** yang menangani tiket dan lihat laporan, serta **Customer** yang hanya bisa bikin dan lihat tiket sendiri.

[ILUSTRASI: Gedung kantor dengan tiga jenis kartu akses — merah (Admin, buka semua pintu), kuning (Agent, buka ruang tiket dan laporan), hijau (Customer, hanya buka ruang tiket sendiri). Setiap pintu ada label endpoint API-nya.]

---

## RBAC: Kontrol Akses Berbasis Peran

Di project TypeScript aslinya, ada middleware `requireRole(allowedRoles)` yang jadi factory: kita kasih daftar role yang boleh masuk, dia kembalikan middleware yang ngecek role user.

Di Rust dengan Axum, pendekatannya berbeda. Karena Rust adalah bahasa yang *type-safe*, kita nggak cukup cek role pakai string. Kita pakai **custom extractor**, yaitu struct khusus yang Axum panggil otomatis sebelum handler dijalankan. Kalau extractor gagal (role nggak sesuai), request langsung ditolak sebelum sampai ke logika bisnis. Hasilnya lebih kuat: kalau handler butuh `AdminOnly`, Rust *wajib* memastikan itu sudah dicek, nggak bisa lupa.

---

## State Awal Bab 29

Dari Bab 28, sudah ada:
- ✅ JWT middleware dengan token verification di `src/middleware/auth.rs`
- ✅ Claims extraction dari token
- ✅ `AuthUser` extractor yang siap untuk dikomposisi dengan role extractors
- ✅ UserRole enum dengan PartialEq (dari Bab 24)
- ✅ parse_role() helper untuk string → enum conversion

Sekarang Bab 29 tambah custom extractors untuk role-based access control.

---

## UserRole Enum: PartialEq Sudah Ada

UserRole enum dari Bab 24 sudah punya `PartialEq` derive:

```rust
// src/models/enums.rs

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}
```

Dengan `#[derive(PartialEq)]`, kita bisa bandingkan dua nilai `UserRole` dengan operator `!=` dan `==`.
Contoh: `role != UserRole::Admin` atau `role == UserRole::Customer`.

---

## Claims Pakai String, Extractor Pakai Enum

Di Bab 28, Claims disimpan dengan `role: String`. Kenapa nggak langsung `role: UserRole`?

JWT adalah standard format untuk serialize data ke JSON dan back. Waktu kita encode Claims ke token, semua field harus bisa di-serialize jadi string. Kalau kita pakai `UserRole` enum langsung, serde akan serialize jadi `"Admin"` (CamelCase), tapi database kita pakai lowercase `"admin"`. Berantakan.

**Solusi yang dipakai di project:**
- Di `generate_token()` (Bab 28): Konversi `UserRole` enum → lowercase string menggunakan `role_to_string()` untuk disimpan di JWT
- Di extractors (Bab 29): Konversi string dari JWT claims → `UserRole` enum menggunakan `parse_claims_role()` untuk type-safe role checking

Tambahkan helper `parse_claims_role()` ke `src/services/auth_service.rs`:

```rust
/// Helper: Konversi string role dari JWT claims ke enum UserRole
pub fn parse_claims_role(role: &str) -> Result<UserRole, AppError> {
    match role {
        "admin" => Ok(UserRole::Admin),
        "agent" => Ok(UserRole::Agent),
        "customer" => Ok(UserRole::Customer),
        _ => Err(AppError::Internal(
            "Invalid role in JWT claims".to_string(),
        )),
    }
}
```

Dengan cara ini, JWT tetap simple (role sebagai string lowercase), tapi di Rust code kita punya type-safe enum untuk perbandingan.

---

## Custom Extractor: Komposisi Role Guard

Di Axum, **extractor** adalah struct yang implement trait `FromRequestParts`. Setiap kali handler dipanggil, Axum otomatis menjalankan semua extractor yang ada di parameter handler. Kalau salah satu gagal, request ditolak dengan error yang kita definisikan.

Trick penting: kita bisa **compose** satu extractor di dalam extractor lain. Caranya: panggil `from_request_parts` dari extractor lain di dalam implementasi kita.

Contoh `AdminOnly`:

```rust
// src/middleware/auth.rs

use crate::models::UserRole;
use crate::services::parse_claims_role;

/// Custom extractor untuk admin users saja
pub struct AdminOnly(pub Claims);

impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Langkah 1: Pastikan user sudah login (compose AuthUser)
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;

        // Langkah 2: Konversi string role ke enum
        let role = parse_claims_role(&claims.role)?;

        // Langkah 3: Cek apakah role adalah Admin
        if role != UserRole::Admin {
            return Err(AppError::Forbidden(
                "Hanya admin yang boleh akses endpoint ini".to_string(),
            ));
        }

        Ok(AdminOnly(claims))
    }
}
```

Pola penting: `let AuthUser(claims) = AuthUser::from_request_parts(...).await?` adalah **destructuring**. Kita langsung ambil `claims` dari dalam struct `AuthUser`. Tanda `?` berarti kalau `AuthUser` gagal (JWT invalid), error langsung di-propagate ke atas — request ditolak sebelum sampai ke handler.

Ini disebut **extractor composition**: satu extractor memanggil extractor lain secara manual. `AdminOnly` tidak perlu mengulang logika JWT verification — cukup panggil `AuthUser::from_request_parts()` yang sudah handle itu. Kalau nanti ada extractor baru (misalnya `VerifiedUser`), tinggal compose dari `AuthUser` juga.

---

## Extractor: AdminOrAgent

Beberapa endpoint boleh diakses oleh admin *atau* agent. Kita pakai `AdminOrAgent`:

```rust
// src/middleware/auth.rs

/// Custom extractor untuk admin atau agent
pub struct AdminOrAgent(pub Claims);

impl<S> FromRequestParts<S> for AdminOrAgent
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        let role = parse_claims_role(&claims.role)?;

        if role != UserRole::Admin && role != UserRole::Agent {
            return Err(AppError::Forbidden(
                "Endpoint ini hanya untuk admin atau agent".to_string(),
            ));
        }

        Ok(AdminOrAgent(claims))
    }
}
```

---

## Extractor: CustomerOnly

Customer register sendiri, jadi ada endpoint yang hanya customer yang bisa akses:

```rust
// src/middleware/auth.rs

/// Custom extractor untuk customer users saja
pub struct CustomerOnly(pub Claims);

impl<S> FromRequestParts<S> for CustomerOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        let role = parse_claims_role(&claims.role)?;

        if role != UserRole::Customer {
            return Err(AppError::Forbidden(
                "Hanya customer yang boleh akses endpoint ini".to_string(),
            ));
        }

        Ok(CustomerOnly(claims))
    }
}
```

---

## Memakai Role Guard di Handler

Dari Bab 27-28, kita tahu handler di project ini mengikuti pattern:
- Extract `State(state): State<AppState>` untuk akses database
- Extract `Json(dto): Json<DTO>` untuk validasi input
- Return `Result<Json<Value>, AppError>` — Axum otomatis convert error ke HTTP response

Sekarang tambahkan extractor role (misalnya `AdminOnly(claims): AdminOnly`) sebagai parameter handler juga. Axum panggil secara otomatis sebelum handler body dijalankan. Kalau role nggak match, extractor langsung throw error dan handler nggak pernah dijalankan.

Contoh paling simple: endpoint `/me` yang return info user yang login:

```rust
// src/main.rs

async fn get_current_user(
    crate::middleware::AuthUser(claims): crate::middleware::AuthUser,
) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": {
            "id": claims.sub,
            "email": claims.email,
            "role": claims.role
        }
    }))
}

// Di router setup:
let auth_routes = Router::new()
    .route("/auth/register", post(handlers::auth_handler::register))
    .route("/auth/login", post(handlers::auth_handler::login))
    .route("/me", get(get_current_user))
    .with_state(state);
```

Kalau handler punya `AdminOnly(claims): AdminOnly` di parameter-nya, Rust *tidak akan compile* kalau kamu lupa pasang extractor itu. Ini jaminan dari type system Rust, bukan dari runtime check yang bisa kelewat.

---

## Tabel Mapping Endpoint ke Role

Berikut pemetaan endpoint di project support desk kita (untuk reference chapter berikutnya):

| Endpoint | Method | Role yang Diizinkan |
|---|---|---|
| `/auth/register` | POST | Public (any) |
| `/auth/login` | POST | Public (any) |
| `/me` | GET | AuthUser (any authenticated) |
| `/users` | GET | AdminOnly |
| `/users/{id}` | GET | AdminOnly |
| `/tickets` | GET | AdminOrAgent (dengan filtering di service) |
| `/tickets` | POST | CustomerOnly |
| `/tickets/{id}` | GET | AuthUser (dengan filtering di service) |
| `/tickets/{id}` | PATCH | AdminOrAgent |
| `/dashboard/stats` | GET | AdminOrAgent |

Untuk endpoint yang punya tanda **(dengan filtering di service)**, extractor hanya jadi gate pertama. Logic filter data lebih lanjut dilakukan di service layer — misalnya, customer hanya lihat tiket mereka sendiri, tapi endpoint tetap dibuka untuk mereka.

[ILUSTRASI: Tabel divisualisasikan sebagai pintu-pintu dengan warna berbeda: merah (AdminOnly), kuning (AdminOrAgent), hijau (AuthUser), abu-abu (public). Setiap pintu menunjukkan HTTP method.]

---

## Latihan

**Latihan 1: Test endpoint `/me`**

Jalankan server, trus coba akses:

```bash
# Daftar customer baru
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name": "Budi", "email": "budi@test.com", "password": "password123", "role": "customer"}'

# Ambil token dari response
TOKEN="..."

# Akses /me dengan token
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/me

# Coba tanpa token — should 401
curl http://localhost:3000/me
```

Apa yang kamu lihat?

**Latihan 2: Coba akses dengan token customer ke endpoint AdminOnly**

Buat handler sederhana yang pakai `AdminOnly` extractor (misalnya `GET /admin-test`). Trus:

```bash
# Dengan token customer
curl -H "Authorization: Bearer $CUSTOMER_TOKEN" http://localhost:3000/admin-test
# Harusnya 403 Forbidden

# Dengan token admin (coba create user dengan role admin di database langsung)
# atau lihat di testing: admin user dibuat waktu migration
```

Apa error yang muncul? Apa HTTP status code-nya?

**Latihan 3: Tambah `AgentOnly` extractor**

Ikuti pola `AdminOnly`, bikin extractor baru `AgentOnly` yang hanya accept `UserRole::Agent`. Kalau role bukan agent, return Forbidden error.

Hint:
```rust
pub struct AgentOnly(pub Claims);
// implement FromRequestParts sama seperti AdminOnly, tapi cek role != UserRole::Agent
```

---

## Hasil Akhir

Berikut adalah kode lengkap untuk Bab 29. Bandingkan dengan project mu untuk memastikan step-by-step di atas tercermin dengan benar.

### Step 1: `src/models/enums.rs` — Tambah PartialEq

```rust
use sqlx::Type;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}

// ... enum lain tetap sama
```

**Apa yang berubah:**
- Tambah `PartialEq` ke derive list supaya bisa perbandingan `==` dan `!=`

---

### Step 2: `src/services/auth_service.rs` — Tambah parse_claims_role()

Tambahkan function baru di akhir file (setelah `verify_token()`):

```rust
/// Helper: Konversi string role dari JWT claims ke enum UserRole
pub fn parse_claims_role(role: &str) -> Result<UserRole, AppError> {
    match role {
        "admin" => Ok(UserRole::Admin),
        "agent" => Ok(UserRole::Agent),
        "customer" => Ok(UserRole::Customer),
        _ => Err(AppError::Internal(
            "Invalid role in JWT claims".to_string(),
        )),
    }
}
```

---

### Step 3: `src/services/mod.rs` — Re-export parse_claims_role

Update exports:

```rust
pub mod auth_service;

pub use auth_service::{AuthService, Claims, verify_token, parse_claims_role};
```

---

### Step 4: `src/middleware/auth.rs` — Tambah Imports dan 3 Extractor Baru

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::common::AppError;
use crate::models::UserRole;
use crate::services::{Claims, verify_token, parse_claims_role};

/// Custom extractor untuk authenticated users (any role)
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

/// Custom extractor untuk admin users saja
pub struct AdminOnly(pub Claims);

impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        let role = parse_claims_role(&claims.role)?;

        if role != UserRole::Admin {
            return Err(AppError::Forbidden(
                "Hanya admin yang boleh akses endpoint ini".to_string(),
            ));
        }

        Ok(AdminOnly(claims))
    }
}

/// Custom extractor untuk admin atau agent
pub struct AdminOrAgent(pub Claims);

impl<S> FromRequestParts<S> for AdminOrAgent
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        let role = parse_claims_role(&claims.role)?;

        if role != UserRole::Admin && role != UserRole::Agent {
            return Err(AppError::Forbidden(
                "Endpoint ini hanya untuk admin atau agent".to_string(),
            ));
        }

        Ok(AdminOrAgent(claims))
    }
}

/// Custom extractor untuk customer users saja
pub struct CustomerOnly(pub Claims);

impl<S> FromRequestParts<S> for CustomerOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        let role = parse_claims_role(&claims.role)?;

        if role != UserRole::Customer {
            return Err(AppError::Forbidden(
                "Hanya customer yang boleh akses endpoint ini".to_string(),
            ));
        }

        Ok(CustomerOnly(claims))
    }
}
```

---

### Step 5: `src/middleware/mod.rs` — Update Re-exports

```rust
pub mod auth;

pub use auth::{AuthUser, AdminOnly, AdminOrAgent, CustomerOnly};
```

---

### Step 6: `src/main.rs` — Tambah test endpoint `/me`

Tambahkan function handler:

```rust
async fn get_current_user(
    crate::middleware::AuthUser(claims): crate::middleware::AuthUser,
) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": {
            "id": claims.sub,
            "email": claims.email,
            "role": claims.role
        }
    }))
}
```

Dan tambahkan route ke `auth_routes`:

```rust
// Setup auth routes dengan state
let auth_routes = Router::new()
    .route("/auth/register", post(handlers::auth_handler::register))
    .route("/auth/login", post(handlers::auth_handler::login))
    .route("/me", get(get_current_user))
    .with_state(state);
```

---

## Verifikasi

```bash
# Build harus 0 error
cargo build

# Jalankan server
cargo run

# Test di terminal lain
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name": "Test", "email": "test@example.com", "password": "password123", "role": "customer"}'

# Ambil token dari response, trus:
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:3000/me
```

Harusnya kelihat info user (id, email, role) dengan status 200.

---

## Kesimpulan Bab 29

**Implementasi RBAC (Role-Based Access Control):**
- ✅ UserRole enum dengan PartialEq untuk type-safe role comparison
- ✅ `parse_claims_role()` helper - konversi string dari JWT → UserRole enum
- ✅ `AdminOnly` extractor - restrict access ke admin saja
- ✅ `AdminOrAgent` extractor - restrict access ke admin atau agent
- ✅ `CustomerOnly` extractor - restrict access ke customer saja

**Key Pattern:**
- Extractors compose `AuthUser` terlebih dahulu (verify token)
- Kemudian parse role string → enum
- Kemudian check role dengan `!=` operator (type-safe)
- Jika role tidak match, return `AppError::Forbidden` (HTTP 403)

**Keamanan Type-Safe:**
Karena extractor adalah Rust trait yang compile-time checked, handler yang membutuhkan `AdminOnly` WAJIB declare itu di parameter. Rust compiler tidak akan compile kalau lupa. Ini lebih kuat dari runtime checks yang bisa kelewat.

**Status Build:** ✅ **0 errors, 22 warnings** (expected - unused extractors sampai dipakai di handlers)
