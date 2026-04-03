# Bab 29: Role-Based Access Control

Bayangkan sebuah gedung kantor besar. Semua karyawan punya kartu akses, tapi nggak semua kartu bisa buka semua pintu. Security guard bisa masuk lobi dan ruang kontrol. Kasir bisa masuk kasir dan gudang kecil. Manager bisa masuk hampir semua ruangan. CEO bisa buka semua pintu, termasuk ruang server.

Inilah **Role-Based Access Control** (RBAC), yaitu sistem kontrol akses yang bergantung pada *peran* (role) seseorang, bukan identitas personalnya. Kita nggak nanya "siapa kamu?", tapi "kamu punya hak akses apa?"

Di project support desk kita, ada tiga peran: **Admin** yang bisa lihat semua dan kelola semua, **Agent** yang menangani tiket dan lihat laporan, serta **Customer** yang hanya bisa bikin dan lihat tiket sendiri.

[ILUSTRASI: Gedung kantor dengan tiga jenis kartu akses — merah (Admin, buka semua pintu), kuning (Agent, buka ruang tiket dan laporan), hijau (Customer, hanya buka ruang tiket sendiri). Setiap pintu ada label endpoint API-nya.]

---

## RBAC: Kontrol Akses Berbasis Peran

Di project TypeScript aslinya, ada middleware `requireRole(allowedRoles)` yang jadi factory: kita kasih daftar role yang boleh masuk, dia kembalikan middleware yang ngecek role user.

Di Rust dengan Axum, pendekatannya berbeda. Karena Rust adalah bahasa yang *type-safe*, kita nggak cukup cek role pakai string. Kita pakai **custom extractor**, yaitu struct khusus yang Axum panggil otomatis sebelum handler dijalankan. Kalau extractor gagal (role nggak sesuai), request langsung ditolak sebelum sampai ke logika bisnis. Hasilnya lebih kuat: kalau handler butuh `AdminOnly`, Rust *wajib* memastikan itu sudah dicek, nggak bisa lupa.

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 28:**
- JWT middleware dengan token verification sudah di `src/middleware/` dari "Hasil Akhir Bab 28"
- Claims ekstraction dari token sudah lengkap

**State Sebelumnya:**
Dari Bab 28, middleware JWT sudah bisa extract & verify token. Sekarang Bab 29 tambah custom extractor untuk role checking.

---

## Role Enum di Rust

Pertama, definisikan role sebagai enum. Enum di Rust bukan sekadar konstanta; ini tipe data tersendiri yang compiler bisa validasi.

```rust
// src/models/user.rs

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum Role {
    Admin,
    Agent,
    Customer,
}
```

`#[derive(PartialEq)]` memungkinkan kita bandingkan role pakai `==`. `sqlx::Type` adalah derive macro dari library SQLx yang membuat enum ini bisa langsung dibaca dari kolom PostgreSQL. `#[sqlx(type_name = "user_role", rename_all = "lowercase")]` memberitahu SQLx bahwa tipe di PostgreSQL namanya `user_role` dengan value lowercase (`admin`, `agent`, `customer`).

Di database migration-nya (dari Bab 24), kita sudah punya:

```sql
CREATE TYPE user_role AS ENUM ('admin', 'agent', 'customer');
```

Dengan derive ini, SQLx bisa otomatis konversi antara `Role::Admin` di Rust dan `"admin"` di PostgreSQL.

---

## Update Claims untuk Pakai Role Enum

Di Bab 28 kita sudah bikin JWT dengan `Claims`. Update struct-nya supaya pakai `Role` enum, bukan `String`.

```rust
// src/auth/jwt.rs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user ID
    pub role: Role,        // pakai Role enum, bukan String
    pub exp: usize,
}
```

Bedanya di sini: kalau pakai String, kita bisa salah ketik: `"Admin"` vs `"admin"` vs `"ADMIN"` semuanya berbeda tapi nggak ada yang error saat compile. Dengan enum, kalau salah ketik, Rust langsung error saat compile. Pastikan `Role` implement `Serialize` dan `Deserialize` (sudah kita derive di atas) supaya bisa dimasukkan ke JWT payload.

---

## Custom Extractor: AdminOnly dan AdminOrAgent

Di Axum, **extractor** adalah struct yang implement trait `FromRequestParts`. Setiap kali handler dipanggil, Axum otomatis menjalankan semua extractor yang ada di parameter handler. Kalau salah satu gagal, request ditolak dengan error yang kita definisikan.

```rust
// src/auth/extractors.rs

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use crate::{auth::jwt::Claims, errors::AppError};

// Wrapper untuk Claims setelah auth berhasil (any role)
pub struct AuthUser(pub Claims);

// Hanya admin
pub struct AdminOnly(pub Claims);

// Admin atau agent
pub struct AdminOrAgent(pub Claims);
```

Implementasi `AuthUser` (dari Bab 28, verifikasi JWT saja):

```rust
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Ambil token dari header Authorization
        // Verifikasi dan decode JWT
        // Return AuthUser(claims) kalau valid
        // ...
    }
}
```

`AdminOnly` meng-compose dari `AuthUser`, lalu tambah pengecekan role:

```rust
#[async_trait]
impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Pertama, pastikan user sudah login
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;

        // Lalu cek role-nya
        if claims.role != Role::Admin {
            return Err(AppError::Forbidden(
                "Hanya admin yang boleh akses endpoint ini".to_string(),
            ));
        }

        Ok(AdminOnly(claims))
    }
}
```

Dan `AdminOrAgent`:

```rust
#[async_trait]
impl<S> FromRequestParts<S> for AdminOrAgent
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;

        if claims.role != Role::Admin && claims.role != Role::Agent {
            return Err(AppError::Forbidden(
                "Endpoint ini hanya untuk admin atau agent".to_string(),
            ));
        }

        Ok(AdminOrAgent(claims))
    }
}
```

Pola `let AuthUser(claims) = AuthUser::from_request_parts(...).await?` adalah **destructuring**, yaitu kita langsung ambil `claims` dari dalam struct `AuthUser`. Tanda `?` artinya kalau `AuthUser` gagal (JWT invalid), error langsung di-propagate ke atas.

`AppError::Forbidden` perlu kita tambahkan ke enum error kita (Bab 22), dengan HTTP status 403:

```rust
// src/errors.rs

pub enum AppError {
    // ... error lain
    Forbidden(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            // ...
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response()
            }
        }
    }
}
```

---

## Memakai Role Guard di Handler

Tinggal pakai extractor sebagai parameter handler. Axum otomatis tahu urutan ekstraksinya.

```rust
// src/handlers/user_handler.rs

// Hanya admin yang bisa lihat semua user
pub async fn list_all_users(
    State(state): State<AppState>,
    AdminOnly(claims): AdminOnly,
    Query(filters): Query<UserFilters>,
) -> AppResult<impl IntoResponse> {
    let users = state.user_service.find_all(filters).await?;
    Ok(Json(users))
}

// Admin dan agent bisa lihat semua tiket
pub async fn list_tickets(
    State(state): State<AppState>,
    AdminOrAgent(claims): AdminOrAgent,
) -> AppResult<impl IntoResponse> {
    let tickets = state.ticket_service.find_all().await?;
    Ok(Json(tickets))
}

// Customer buat tiket baru
pub async fn create_ticket(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,  // any authenticated user
    Json(body): Json<CreateTicketDto>,
) -> AppResult<impl IntoResponse> {
    // Validasi role customer di service layer jika perlu
    let ticket = state.ticket_service.create(claims.sub, body).await?;
    Ok((StatusCode::CREATED, Json(ticket)))
}
```

Kalau handler punya `AdminOnly(claims): AdminOnly` di parameter-nya, Rust *tidak akan compile* kalau kamu lupa pasang extractor itu. Ini jaminan dari type system Rust, bukan dari runtime check yang bisa kelewat.

---

## Tabel Mapping Endpoint ke Role

Berikut pemetaan endpoint di project support desk kita:

| Endpoint | Method | Role yang Diizinkan |
|---|---|---|
| `/users` | GET | Admin only |
| `/users/{id}` | GET | Admin only |
| `/agents` | GET | Admin only |
| `/customers` | GET | Admin only |
| `/tickets` | GET | Admin, Agent (filtering per role di service) |
| `/tickets` | POST | Customer only |
| `/tickets/{id}` | PATCH | Admin, Agent |
| `/tickets/{id}` | GET | Admin, Agent, Customer (sesuai ownership) |
| `/dashboard/stats` | GET | Admin, Agent |

Untuk endpoint `/tickets GET`, filtering tambahan dilakukan di service layer. Admin dan Agent lihat semua tiket, sedangkan Customer hanya lihat tiket mereka sendiri. Ini bukan soal akses endpoint, tapi soal filter data.

[ILUSTRASI: Tabel di atas divisualisasikan sebagai pintu-pintu dengan warna berbeda: merah (admin only), kuning (admin+agent), hijau (semua user login), abu-abu (logic di service layer)]

---

## Latihan

**Latihan 1: Tambah extractor `CustomerOnly`**

Buat extractor `CustomerOnly` mengikuti pola `AdminOnly`. Handler `create_ticket` seharusnya hanya bisa dipanggil oleh customer, bukan admin atau agent.

```rust
pub struct CustomerOnly(pub Claims);

// Implement FromRequestParts untuk CustomerOnly
// Hint: cek claims.role != Role::Customer
```

**Latihan 2: Coba akses tanpa token**

Jalankan server, lalu coba akses `GET /users` tanpa header `Authorization`. Apa error yang muncul? Apa HTTP status code-nya?

Sekarang coba dengan token customer yang valid. Apa yang berbeda?

**Latihan 3: Role-based filtering di service**

Untuk endpoint `GET /tickets`, sekarang admin lihat semua tiket. Ubah service layer supaya kalau `claims.role == Role::Customer`, hanya kembalikan tiket milik `claims.sub`, sedangkan kalau `claims.role == Role::Admin` atau `Role::Agent`, kembalikan semua tiket. Handler tetap pakai `AuthUser`, tapi service yang memfilter berdasarkan role.

---

Di bab berikutnya kita akan membahas bagaimana melindungi route di level router, sehingga kita bisa groupkan endpoint berdasarkan role sebelum sampai ke handler.
