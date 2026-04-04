# Bab 22: Respons API Standar

Bayangkan kantor besar dengan 5 departemen yang masing-masing kirim laporan ke manajemen. Departemen A pakai Excel, B pakai PDF, C kirim lewat WhatsApp, D tulis tangan di kertas, E kirim email panjang tanpa struktur. Manajemen pusing karena setiap laporan harus dibaca dengan cara berbeda.

Solusinya: satu template seragam. Ada kolom status, pesan, dan data. Berhasil? Tulis "sukses". Gagal? Tulis "gagal" beserta alasannya.

Itulah yang dibangun di bab ini: satu format response standar untuk semua endpoint API.

[ILUSTRASI: Perbandingan dua respons API — kiri: berbagai format tidak konsisten dari tiap endpoint, kanan: semua endpoint mengembalikan format JSON yang seragam dengan field success, message, dan data]

---

---

## Kunci Jawaban Latihan Bab 21

Berikut jawaban untuk semua latihan Bab 21:

### Latihan #1: Buat `UpdateTicketDto`

File: `src/dto/ticket_dto.rs` (tambahkan di akhir file, sebelum custom function)

```rust
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTicketDto {
    #[validate(length(min = 5, max = 200, message = "Subject harus 5-200 karakter"))]
    pub subject: Option<String>,

    #[validate(custom(function = "validate_status"))]
    pub status: Option<String>,
}
```

Lalu tambah validate function:
```rust
fn validate_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "open" | "in_progress" | "resolved" | "closed" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_status")),
    }
}
```

Lalu update `src/dto/mod.rs`:
```rust
pub use ticket_dto::{CreateTicketDto, UpdateTicketDto};
```

### Latihan #2: Test manual dengan invalid input

Jalankan server: `cargo run`

Kemudian test di terminal lain:
```bash
# Test dengan subject terlalu pendek (invalid)
curl -X POST http://localhost:3000/tickets \
  -H "Content-Type: application/json" \
  -d '{"subject":"ok","description":"test test","category":"general","priority":"high"}'

# Response harus 422 Unprocessable Entity dengan error message
```

### Latihan #3: Custom error message dengan `.message` field

Update validate functions di `src/dto/ticket_dto.rs`:

```rust
fn validate_category(category: &str) -> Result<(), validator::ValidationError> {
    match category {
        "general" | "billing" | "technical" | "other" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_category");
            err.message = Some("Category harus: general, billing, technical, atau other".into());
            Err(err)
        }
    }
}

fn validate_priority(priority: &str) -> Result<(), validator::ValidationError> {
    match priority {
        "low" | "medium" | "high" | "urgent" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_priority");
            err.message = Some("Priority harus: low, medium, high, atau urgent".into());
            Err(err)
        }
    }
}

fn validate_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "open" | "in_progress" | "resolved" | "closed" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_status");
            err.message = Some("Status harus: open, in_progress, resolved, atau closed".into());
            Err(err)
        }
    }
}
```

Juga update `src/dto/user_dto.rs`:
```rust
fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    match role {
        "customer" | "agent" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_role");
            err.message = Some("Role harus: customer atau agent (tidak boleh admin)".into());
            Err(err)
        }
    }
}
```

---

## State Sebelumnya

Setelah menyelesaikan semua latihan Bab 21, folder struktur harus seperti ini:

```
src/
├── main.rs
├── models/
│   ├── mod.rs
│   ├── user.rs
│   ├── ticket.rs
│   └── api_response.rs
└── dto/
    ├── mod.rs
    ├── ticket_dto.rs
    └── user_dto.rs
```

**File: `src/dto/ticket_dto.rs`**
```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketDto {
    #[validate(length(min = 5, max = 200, message = "Subject harus 5-200 karakter"))]
    pub subject: String,

    #[validate(length(min = 10, message = "Deskripsi minimal 10 karakter"))]
    pub description: String,

    #[validate(custom(function = "validate_category"))]
    pub category: String,

    #[validate(custom(function = "validate_priority"))]
    pub priority: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTicketDto {
    #[validate(length(min = 5, max = 200, message = "Subject harus 5-200 karakter"))]
    pub subject: Option<String>,

    #[validate(custom(function = "validate_status"))]
    pub status: Option<String>,
}

fn validate_category(category: &str) -> Result<(), validator::ValidationError> {
    match category {
        "general" | "billing" | "technical" | "other" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_category");
            err.message = Some("Category harus: general, billing, technical, atau other".into());
            Err(err)
        }
    }
}

fn validate_priority(priority: &str) -> Result<(), validator::ValidationError> {
    match priority {
        "low" | "medium" | "high" | "urgent" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_priority");
            err.message = Some("Priority harus: low, medium, high, atau urgent".into());
            Err(err)
        }
    }
}

fn validate_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "open" | "in_progress" | "resolved" | "closed" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_status");
            err.message = Some("Status harus: open, in_progress, resolved, atau closed".into());
            Err(err)
        }
    }
}
```

**File: `src/dto/user_dto.rs`**
```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterDto {
    #[validate(length(min = 2, max = 100, message = "Nama harus 2-100 karakter"))]
    pub name: String,

    #[validate(email(message = "Format email tidak valid"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password minimal 8 karakter"))]
    pub password: String,

    #[validate(custom(function = "validate_role"))]
    pub role: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email(message = "Format email tidak valid"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password tidak boleh kosong"))]
    pub password: String,
}

fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    match role {
        "customer" | "agent" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_role");
            err.message = Some("Role harus: customer atau agent (tidak boleh admin)".into());
            Err(err)
        }
    }
}
```

**File: `src/dto/mod.rs`**
```rust
pub mod ticket_dto;
pub mod user_dto;

pub use ticket_dto::{CreateTicketDto, UpdateTicketDto};
pub use user_dto::{LoginDto, RegisterDto};
```

**File: `src/main.rs`** — Update untuk import dto:
```rust
mod dto;
mod models;

// ... rest of code tetap sama
```

Verifikasi: `cargo build` harus berhasil tanpa error

---

## Kenapa Perlu Format Standar?

Di bab sebelumnya sudah ada model struct, DTO dengan validasi, dan routing dasar. `ApiResponse<T>` juga sudah didefinisikan. Masalahnya: belum terintegrasi dengan sistem response Axum.

Tanpa format standar, tiap handler bisa return JSON yang bentuknya beda-beda:

```json
// Endpoint A
{ "user": { "id": 1, "name": "Budi" } }

// Endpoint B
{ "data": [...], "count": 10 }

// Endpoint C
{ "error": "User not found" }

// Endpoint D
"Internal Server Error"
```

Frontend yang konsumsi API ini harus nulis logika parsing yang berbeda untuk tiap endpoint. Menyiksa.

Format standar yang dituju:

```json
// Sukses
{ "success": true, "message": "Data berhasil diambil", "data": { ... } }

// Gagal
{ "success": false, "message": "User tidak ditemukan", "data": null }
```

Konsisten, predictable, mudah di-handle di frontend.

---

## IntoResponse — Trait untuk HTTP Response

**Trait** adalah kontrak di Rust, semacam perjanjian bahwa tipe ini bisa melakukan sesuatu. `IntoResponse` adalah kontrak dari Axum yang bilang: *"tipe ini bisa diubah jadi HTTP response"*.

Tiap handler Axum harus return sesuatu yang implement `IntoResponse`. Axum sudah bawaan support beberapa tipe: `String`, `StatusCode`, tuple `(StatusCode, Json<T>)`, dan lainnya.

Supaya `ApiResponse<T>` bisa langsung di-return dari handler tanpa wrap manual, caranya sederhana: **implement `IntoResponse` untuk `ApiResponse<T>`**. Tanpa ini, compiler akan menolak handler kamu dengan error karena Axum tidak tahu cara mengubah `ApiResponse<T>` jadi HTTP response.

Analogi: Kalau mau kirim paket ke luar negeri, paket harus di-pack dengan format standar (bungkus dengan plastik, label, dll). Begitu juga response harus di-"pack" jadi format HTTP response yang Axum mengerti. `IntoResponse` adalah packer-nya.

---

## ApiResponse\<T\> dengan IntoResponse

Kita akan membuat file baru di folder `src/common/` untuk mengelola semua response-related code. Struktur yang akan kita buat:

```
src/
├── common/              ← NEW FOLDER
│   ├── mod.rs          ← pub use semua response types
│   └── response.rs     ← ApiResponse, PaginatedResponse, AppError, AppResult
├── models/
├── dto/
└── main.rs
```

**Langkah 1:** Buat folder `src/common/`

**Langkah 2:** Buat file `src/common/response.rs` dengan kode berikut:

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

// Struct ApiResponse yang bisa langsung di-return dari handler
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

// Method helper untuk membuat response sukses
impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }
}

// Implement IntoResponse — ini yang bikin ApiResponse bisa langsung di-return
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        // Bungkus dengan StatusCode::OK dan Json, maka otomatis jadi HTTP response
        (StatusCode::OK, Json(self)).into_response()
    }
}
```

**Penjelasan:**

- **`#[serde(skip_serializing_if = "Option::is_none")]`**: Field `data` tidak akan muncul di JSON kalau nilainya `None`. Jadi output lebih rapi, bukan `"data": null`.
- **`T: Serialize`**: Generic constraint. Tipe `T` apapun yang dipakai harus bisa di-serialize ke JSON.
- **`into_response` method**: Ini yang "mengubah" `ApiResponse<T>` menjadi HTTP response yang Axum mengerti. Kita bungkus dengan `(StatusCode::OK, Json(self))` yang sudah punya `IntoResponse` bawaan Axum.

**Hasil:** Handler sekarang bisa ditulis lebih bersih:

```rust
use axum::extract::Path;
use uuid::Uuid;
use crate::common::ApiResponse;

pub async fn get_user(Path(id): Path<Uuid>) -> ApiResponse<UserDto> {
    let user = // ... ambil dari database
    ApiResponse::ok(user, "User berhasil diambil")
}
```

Tidak perlu wrap manual dengan `Json(...)` atau atur status code! Jauh lebih rapi.

---

## PaginatedResponse untuk List Data

Data list biasanya butuh pagination, tidak mungkin return 10.000 data sekaligus. Frontend perlu tahu: total data berapa, sekarang di halaman berapa, satu halaman isinya berapa.

```rust
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, limit: i64) -> Self {
        let total_pages = (total as f64 / limit as f64).ceil() as i64;
        Self { success: true, data, total, page, limit, total_pages }
    }
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
```

`total_pages` dihitung otomatis dari `total` dibagi `limit`, dibulatkan ke atas pakai `.ceil()`. Jadi kalau ada 25 data dengan limit 10, hasilnya 3 halaman (bukan 2.5).

Contoh output JSON:

```json
{
  "success": true,
  "data": [...],
  "total": 25,
  "page": 1,
  "limit": 10,
  "total_pages": 3
}
```

---

## AppError yang Otomatis Jadi HTTP Response

Tanpa penanganan terpusat, setiap handler harus manual atur status code dan format pesan error. Ribuan baris kode akan penuh dengan logika yang sama berulang-ulang:

```rust
// Cara lama — berulang di setiap handler 😞
async fn some_handler() -> (StatusCode, Json<Value>) {
    if user_not_found {
        return (StatusCode::NOT_FOUND, Json(json!({
            "success": false,
            "message": "User tidak ditemukan"
        })));
    }
}
```

Solusi: **buat enum `AppError` yang langsung implement `IntoResponse`**. Setiap kali error terjadi, Axum tahu otomatis harus kirim response apa.

Tambahkan kode di `src/common/response.rs` (lanjutkan dari `ApiResponse`):

```rust
// ============================================
// AppError — Enum untuk semua jenis error
// ============================================

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    BadRequest(String),
    ValidationError(String),
    Internal(String),
}

// Implement IntoResponse untuk AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Match error type, tentukan status code dan message
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        // Buat response dengan format standar ApiResponse
        let body = ApiResponse::<()> {
            success: false,
            message,
            data: None,
        };

        (status, Json(body)).into_response()
    }
}
```

**Penting:** Lihat `AppError::Internal` — pesannya **tidak diekspos** ke client. Internal error seringkali berisi detail teknis berbahaya (koneksi database error, stack trace). Kita log di server untuk debugging, tapi kirim pesan generik "Internal server error" ke luar.

**Contoh pemakaian di handler:**

```rust
use crate::common::{ApiResponse, AppError, AppResult};

pub async fn get_user(Path(id): Path<Uuid>) -> AppResult<ApiResponse<UserDto>> {
    // Cek user di database
    let user = db.find_user(id)
        .await
        .ok_or(AppError::NotFound(format!("User {} tidak ditemukan", id)))?;
    
    // Kalau ada error di tengah jalan, Axum otomatis handle dengan into_response
    Ok(ApiResponse::ok(user, "User berhasil diambil"))
}
```

Kalau `find_user` return `None`, kita konversi ke `AppError::NotFound` dengan `ok_or(...)`. Tanda `?` artinya: kalau error, langsung return error itu (Axum akan panggil `into_response` otomatis).

[ILUSTRASI: Diagram alur error — dari database error atau validasi error, melalui AppError enum, kemudian otomatis dikonversi menjadi HTTP response dengan status code yang tepat dan format JSON yang konsisten]

---

## AppResult — Type Alias yang Praktis

Daripada nulis `Result<T, AppError>` berulang-ulang di setiap handler, buat type alias:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

**Type alias** hanya nama lain untuk tipe yang sudah ada, tidak bikin tipe baru, hanya shortcut untuk pengetikan lebih cepat.

Tambahkan di `src/common/response.rs`:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

**Manfaat:**

- `Result<T, AppError>` → `AppResult<T>` (lebih pendek)
- Handler jadi lebih bersih dan mudah dibaca
- Kalau nanti ganti error type, hanya ubah di satu tempat

**Contoh pemakaian:**

```rust
use crate::common::{ApiResponse, AppError, AppResult};

// Return type: AppResult<ApiResponse<UserDto>>
// Artinya: Result<ApiResponse<UserDto>, AppError>
pub async fn get_user(Path(id): Path<Uuid>) -> AppResult<ApiResponse<UserDto>> {
    let user = find_user_by_id(id).await
        .ok_or(AppError::NotFound(format!("User {} tidak ditemukan", id)))?;

    Ok(ApiResponse::ok(user, "User berhasil diambil"))
}
```

Tanda `?` artinya:
- Kalau ada error, langsung return error itu (Axum akan otomatis panggil `into_response`)
- Kalau sukses, lanjut ke baris berikutnya
- Lebih ringkas daripada `.map_err()` manual

---

## Module Declaration: `src/common/mod.rs`

**Langkah 3:** Buat file `src/common/mod.rs`:

```rust
pub mod response;

pub use response::{ApiResponse, AppError, AppResult, PaginatedResponse};
```

**Langkah 4:** Daftarkan di `src/main.rs` di paling atas:

```rust
mod models;
mod dto;
mod common;  // ← TAMBAH INI
```

**Setelah itu**, di handler manapun, import dengan:

```rust
use crate::common::{ApiResponse, AppError, AppResult, PaginatedResponse};
```

Dan bisa langsung pakai di handler tanpa wrap-wrap ribet. ✓

---

## Latihan

1. **Tambah variant baru di AppError:** Buat variant `BadRequest(String)` yang return status code `400 BAD REQUEST`. Tambahkan juga di `IntoResponse` impl-nya.

2. **Buat handler sederhana:** Buat handler `get_ticket` yang menerima `Path<i32>`. Kalau id-nya genap, return sukses dengan data dummy. Kalau ganjil, return `AppError::NotFound`. Pastikan return type-nya `AppResult<ApiResponse<TicketDto>>`.

3. **Test pagination math:** Tulis unit test untuk `PaginatedResponse::new` dengan beberapa kasus: total=0, total=10 limit=10, total=11 limit=10. Pastikan `total_pages` selalu benar.

4. **Eksplor `skip_serializing_if`:** Coba hapus atribut `#[serde(skip_serializing_if = "Option::is_none")]` dari `ApiResponse`, lalu jalankan dan lihat perbedaan output JSON-nya. Kapan `data: null` di JSON itu oke, kapan tidak? ⚠️ **OPTIONAL**

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan latihan Bab 22, folder struktur harus seperti ini:

```
src/
├── main.rs                  ← update: tambah mod common;
├── models/
│   ├── mod.rs
│   ├── user.rs
│   ├── ticket.rs
│   └── api_response.rs
├── dto/
│   ├── mod.rs
│   ├── ticket_dto.rs
│   └── user_dto.rs
└── common/                  ← NEW FOLDER
    ├── mod.rs              ← NEW
    └── response.rs         ← NEW
```

**File baru yang dibuat:** `src/common/response.rs` dan `src/common/mod.rs`

**File: `src/common/response.rs`**
```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: u32, limit: u32) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        Self {
            success: true,
            data,
            total,
            page,
            limit,
            total_pages,
        }
    }
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    BadRequest(String),          // dari latihan #1
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
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = ApiResponse::<()> {
            success: false,
            message,
            data: None,
        };

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_total_zero() {
        let response = PaginatedResponse::new(Vec::<i32>::new(), 0, 1, 10);
        assert_eq!(response.total_pages, 0);
    }

    #[test]
    fn test_pagination_exact_fit() {
        let response = PaginatedResponse::new(vec![1, 2, 3], 10, 1, 10);
        assert_eq!(response.total_pages, 1);
    }

    #[test]
    fn test_pagination_multiple_pages() {
        let response = PaginatedResponse::new(vec![1], 11, 1, 10);
        assert_eq!(response.total_pages, 2);
    }
}
```

**File: `src/common/mod.rs`**
```rust
pub mod response;

pub use response::{ApiResponse, AppError, AppResult, PaginatedResponse};
```

**Update File: `src/main.rs`** — Tambahkan `mod common;` di paling atas sebelum imports:
```rust
mod models;
mod dto;
mod common;  // ← TAMBAH INI

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use crate::dto::CreateTicketDto;
use validator::Validate;

// ... rest of code tetap sama dengan Bab 21
```

**Verifikasi:**
```bash
cargo build
# Harus compile tanpa error

# Test unit test:
cargo test test_pagination
# Semua test harus pass

# Run server:
cargo run
```

Di bab berikutnya, kita akan setup database PostgreSQL menggunakan Docker dan SQLx. Kalau kamu belum pernah pakai Docker, jangan khawatir — kita akan setup dari nol, step-by-step. Pastikan Docker sudah ter-install di komputermu sebelum lanjut.
