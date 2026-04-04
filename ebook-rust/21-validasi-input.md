# Bab 21: Validasi Input

Bayangkan kamu lagi di bandara, mau naik pesawat. Sebelum masuk gate, ada security check, petugas memeriksa tiket, passport, bawaan kamu. Kalau ada yang mencurigakan, langsung ditolak di situ. Kamu nggak bisa bilang "percaya aja deh sama saya" dan langsung masuk.

Validasi input di aplikasi kita bekerja persis seperti itu. Data yang dikirim client, baik form registrasi, form buat tiket, maupun apapun lainnya, harus kita periksa dulu sebelum diproses. Jangan pernah percaya begitu saja.

[ILUSTRASI: Diagram alur request masuk → validasi (security check) → kalau gagal langsung tolak dengan error, kalau lolos lanjut ke business logic]

---

## Kunci Jawaban Latihan Bab 20

Berikut jawaban untuk latihan Bab 20:

### Latihan #1: Buat struct `TicketResponse`

File: `src/models/ticket.rs` (tambahkan di akhir file)

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TicketResponse {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub message: String,
    pub created_at: DateTime<Utc>,
}
```

Lalu update `src/models/mod.rs` untuk export:
```rust
pub use ticket::{Ticket, TicketResponse};
```

### Latihan #2: Uji serialisasi manual

Di `src/main.rs`, sebelum `#[tokio::main]`, tambahkan test function:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_user_serialization() {
        let user = models::User {
            id: Uuid::new_v4(),
            name: "Budi".to_string(),
            email: "budi@example.com".to_string(),
            password: "secret_password".to_string(),
            role: "customer".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string_pretty(&user).unwrap();
        println!("User JSON:\n{}", json);
        
        // Verifikasi password tidak ada di JSON
        assert!(!json.contains("secret_password"));
        assert!(!json.contains("password"));
        assert!(json.contains("Budi"));
        assert!(json.contains("budi@example.com"));
    }
}
```

Jalankan dengan: `cargo test test_user_serialization -- --nocapture`

### Latihan #2: Ubah `role` menjadi enum (OPTIONAL)

Jika ingin dikerjakan, buat enum di `src/models/user.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Agent,
    Customer,
}

// Kemudian update User struct:
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: Role,  // ← ganti dari String
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## State Sebelumnya

Setelah menyelesaikan Bab 20 (tanpa mengerjakan latihan opsional), folder struktur dan file kamu harus seperti ini:

```
src/
├── main.rs
└── models/
    ├── mod.rs
    ├── user.rs
    ├── ticket.rs
    └── api_response.rs
```

**File: `src/models/user.rs`**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**File: `src/models/ticket.rs`**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: Uuid,
    pub customer_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Uuid>,
    pub category: String,
    pub priority: String,
    pub status: String,
    pub subject: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**File: `src/models/api_response.rs`**
```rust
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
        ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}
```

**File: `src/models/mod.rs`**
```rust
pub mod api_response;
pub mod ticket;
pub mod user;

pub use api_response::ApiResponse;
pub use ticket::Ticket;
pub use user::User;
```

**File: `src/main.rs`** — Tetap sama seperti Bab 19, hanya tambah di atas:
```rust
mod models;

// ... rest of code tetap sama
```

Verifikasi dengan: `cargo build` harus berhasil tanpa error

⚠️ **Catatan:** Jika kamu mengerjakan latihan #1-2 Bab 20 (opsional), DTOs dan TicketResponse mungkin ada di models. Untuk Bab 21, kita akan membuat folder `src/dto/` terpisah dan pindahkan DTOs ke sana.

---

## Kenapa Validasi Penting?

Di Bab 20 kita sudah punya struct DTO seperti `CreateTicketDto`. Struct itu memastikan *struktur* data, yaitu field yang ada harus sesuai tipe. Tapi serde tidak peduli soal *isi* data.

Misalnya, user bisa kirim `subject: ""` (string kosong, tapi valid secara tipe), `email: "bukan-email"` (string valid, tapi bukan format email), atau `priority: "super-ultra-urgent"` (value sembarangan). Kalau kita simpan data sampah itu ke database, masalahnya akan muncul belakangan dan susah dilacak. Lebih baik tolak di pintu masuk.

**Prinsipnya sederhana: validasi di level paling awal, sebelum data menyentuh business logic atau database.**

---

## `validator` Crate

Rust punya crate bernama `validator` yang bikin validasi jadi elegan, kita tinggal tambah attribute ke struct, dan crate ini yang mengurus semuanya.

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
validator = { version = "0.18", features = ["derive"] }
# (saat ebook ini ditulis, Maret 2026)
```

Feature `derive` wajib aktif karena kita akan pakai macro `#[derive(Validate)]`.

---

## Attribute Validasi

Setelah derive `Validate`, kita bisa annotate setiap field dengan aturan validasinya.

### Length — Panjang String

```rust
#[validate(length(min = 5, max = 200, message = "Subject harus 5-200 karakter"))]
pub subject: String,
```

`min` dan `max` mengacu pada jumlah karakter. `message` adalah pesan error yang muncul kalau validasi gagal, opsional tapi bagus untuk UX.

### Email

```rust
#[validate(email(message = "Format email tidak valid"))]
pub email: String,
```

Attribute `email` otomatis mengecek format email yang valid: ada `@`, ada domain, dsb. Kita nggak perlu nulis regex sendiri.

### Custom

Untuk validasi yang lebih spesifik, seperti mengecek nilai harus salah satu dari enum tertentu, kita pakai `custom`:

```rust
#[validate(custom(function = "validate_category"))]
pub category: String,
```

Cara nulis function-nya dibahas di bagian berikut.

---

## Custom Validation Function

Kadang aturan validasi kita terlalu spesifik untuk di-cover oleh attribute bawaan. Di sinilah custom function berguna.

Signature-nya selalu sama: terima reference ke nilai, kembalikan `Result<(), validator::ValidationError>`.

```rust
fn validate_category(category: &str) -> Result<(), validator::ValidationError> {
    match category {
        "general" | "billing" | "technical" | "other" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_category")),
    }
}

fn validate_priority(priority: &str) -> Result<(), validator::ValidationError> {
    match priority {
        "low" | "medium" | "high" | "urgent" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_priority")),
    }
}
```

Kalau nilai cocok dengan salah satu opsi yang valid, kembalikan `Ok(())`. Kalau tidak, kembalikan `Err` dengan kode error (string identifier, bukan pesan untuk user).

---

## DTO Lengkap dengan Validasi

Sekarang kita bikin DTOs dengan validasi. **DTO (Data Transfer Object)** adalah struct khusus untuk menerima input dari client HTTP — terpisah dari Model yang merepresentasikan data di database. Kenapa dipisah? Karena model harus match schema database, sedangkan DTO bisa punya validation rules, field opsional, atau struktur yang berbeda. Contoh: `password` ada di `RegisterDto` (input), tapi di-skip saat serialize `User` model (output).

DTOs akan disimpan di folder `src/dto/` (pisah dari `src/models/` yang berisi models untuk database).

### Struktur Folder DTO

**Langkah 1:** Buat folder baru bernama `src/dto/` (jika belum ada)

```
src/
├── main.rs           ← sudah ada dari Bab 19
├── models/           ← sudah ada dari Bab 20
└── dto/              ← NEW FOLDER — akan dibuat di bagian ini
    ├── mod.rs        ← NEW
    ├── ticket_dto.rs ← NEW
    └── user_dto.rs   ← NEW
```

---

### File 1 (BARU): `src/dto/ticket_dto.rs`

**Lokasi:** Buat file baru bernama `ticket_dto.rs` di dalam folder `src/dto/`

**Isi file:**

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

fn validate_category(category: &str) -> Result<(), validator::ValidationError> {
    match category {
        "general" | "billing" | "technical" | "other" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_category")),
    }
}

fn validate_priority(priority: &str) -> Result<(), validator::ValidationError> {
    match priority {
        "low" | "medium" | "high" | "urgent" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_priority")),
    }
}
```

---

### File 2 (BARU): `src/dto/user_dto.rs`

**Lokasi:** Buat file baru bernama `user_dto.rs` di dalam folder `src/dto/`

**Isi file:**

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

fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    match role {
        "customer" | "agent" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email(message = "Format email tidak valid"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password tidak boleh kosong"))]
    pub password: String,
}
```

**Catatan penting di `RegisterDto`:** role hanya boleh `customer` atau `agent`, bukan `admin`. User tidak boleh mendaftarkan diri sebagai admin lewat API publik.

---

### File 3 (BARU): `src/dto/mod.rs`

**Lokasi:** Buat file baru bernama `mod.rs` di dalam folder `src/dto/`

**Isi file:**

File ini tugasnya "mengumpulkan" semua sub-module (ticket_dto.rs dan user_dto.rs) dan mengeksport yang penting:

```rust
pub mod ticket_dto;
pub mod user_dto;

pub use ticket_dto::CreateTicketDto;
pub use user_dto::{LoginDto, RegisterDto};
```

### Update `src/main.rs`

**File yang diupdate:** `src/main.rs` (file dari Bab 19)

**Yang ditambah:** Tambahkan satu baris di **paling atas** file, bersama dengan `mod models;`:

```rust
mod models;
mod dto;  // ← TAMBAH INI

use axum::{
    // ... rest of imports
};
```

Sekarang di handler manapun, kamu bisa import dengan:

```rust
use crate::dto::{CreateTicketDto, LoginDto, RegisterDto};
```

[ILUSTRASI: Tabel perbandingan aturan validasi Zod (TypeScript) vs validator crate (Rust) — menunjukkan bahwa logika bisnisnya sama, hanya sintaksnya berbeda]

---

## Menjalankan Validasi di Handler

Setelah struct ter-derive dengan `Validate`, menjalankan validasinya mudah. Panggil `.validate()` dan tangani hasilnya.

**Update handler `create_ticket` di `src/main.rs` (file dari Bab 19):**

**Langkah 1:** Tambahkan 2 import di awal file `src/main.rs`, bersama import lainnya:

```rust
use crate::dto::CreateTicketDto;  // ← NEW
use validator::Validate;          // ← NEW
```

**Langkah 2:** Cari fungsi `create_ticket` di `src/main.rs` (yang sudah ada) dan **ganti seluruh fungsi tersebut** dengan:

```rust
async fn create_ticket(
    Json(body): Json<CreateTicketDto>,  // ← CHANGE: Json<Value> → Json<CreateTicketDto>
) -> (StatusCode, Json<Value>) {
    // ← TAMBAH: Validasi input sebelum proses
    if let Err(errors) = body.validate() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "success": false,
                "message": "Validasi gagal",
                "errors": errors.to_string()
            })),
        );
    }

    // Data sudah bersih, lanjut proses bisnis
    println!("Ticket baru: subject={}, category={}", body.subject, body.category);
    
    (StatusCode::CREATED, Json(json!({ 
        "success": true,
        "message": "Ticket berhasil dibuat"
    })))
}
```

**Yang berubah dari Bab 19:**
1. Parameter: `Json(body): Json<Value>` → `Json(body): Json<CreateTicketDto>` (sekarang typed, bukan JSON sembarangan!)
2. Validasi: Tambah `.validate()` check sebelum proses data
3. Error handling: Return `422 Unprocessable Entity` jika validasi gagal
4. Response error: Menyertakan pesan error yang detail dari validator

**Hasil:** Sekarang endpoint `/tickets` POST akan:
- ✓ Terima data dengan type-safe (CreateTicketDto)
- ✓ Validasi sesuai rules yang didefinisikan di DTO
- ✓ Return error 422 + detail error jika tidak valid
- ✓ Return error 201 + success jika valid

**Penjelasan `.validate()`:**
- Mengembalikan `Result<(), ValidationErrors>`
- Kalau Ada error, semua error-nya terkumpul di `ValidationErrors`, bukan berhenti di error pertama
- Jadi client dapat info lengkap sekaligus: "subject terlalu pendek, category invalid, dll"
- Status code `422 Unprocessable Entity` adalah standar HTTP untuk "request diterima tapi isinya bermasalah"

### Tanda `?` sebagai Alternatif

Kalau handler kamu sudah mengembalikan tipe `Result`, kamu bisa pakai `?` langsung:

```rust
body.validate().map_err(|e| {
    (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({
        "success": false,
        "errors": e.to_string()
    })))
})?;
```

Pilih mana yang lebih cocok dengan struktur handler kamu.

---

## Latihan

1. **Tambah DTO baru:** Buat `UpdateTicketDto` dengan field `subject` (opsional, tapi kalau diisi minimal 5 karakter) dan `status` (enum: `open`, `in_progress`, `resolved`, `closed`). Hint: untuk field opsional, gunakan `Option<String>` dan attribute `#[validate(nested)]` atau kondisional.

2. **Uji coba manual:** Jalankan server, lalu kirim request ke endpoint create ticket dengan `subject: "ok"` (kurang dari 5 karakter). Pastikan kamu mendapat response `422` dengan pesan error yang jelas.

3. **Custom error message:** Perbarui `validate_category` dan `validate_priority` supaya menyertakan pesan error yang lebih informatif, bukan hanya kode seperti `"invalid_category"`. Gunakan `ValidationError::new` lalu set field `message`. ⚠️ **MANDATORY untuk Bab 22**

---

## Hasil Akhir Bab Ini

Setelah bab ini, struktur folder harus seperti ini:

```
src/
├── main.rs           ← masih sama, belum pakai DTO
├── models/
│   ├── mod.rs
│   ├── user.rs
│   ├── ticket.rs
│   └── api_response.rs
└── dto/              ← NEW FOLDER
    ├── mod.rs        ← NEW
    ├── ticket_dto.rs ← NEW
    └── user_dto.rs   ← NEW
```

**File 1: `src/dto/ticket_dto.rs`** (sesuaikan dengan latihan)
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

**File 2: `src/dto/user_dto.rs`**
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

**File 3: `src/dto/mod.rs`**
```rust
pub mod ticket_dto;
pub mod user_dto;

pub use ticket_dto::{CreateTicketDto, UpdateTicketDto};
pub use user_dto::{LoginDto, RegisterDto};
```

**Update File 4: `src/main.rs`** — Tambahkan di paling atas:
```rust
mod dto;

// ... rest of code tetap sama
```

**Verifikasi:**
```bash
cargo build
# Harus compile tanpa error

# Test dengan request yang tidak valid:
cargo run

# Di terminal lain:
curl -X POST http://localhost:3000/tickets \
  -H "Content-Type: application/json" \
  -d '{"subject":"ok","description":"test","category":"general","priority":"high"}'
# Harus response 422 dengan pesan error

# Test dengan request yang valid:
curl -X POST http://localhost:3000/tickets \
  -H "Content-Type: application/json" \
  -d '{"subject":"Server Down Issue","description":"Server tidak bisa diakses","category":"technical","priority":"urgent"}'
# Harus response 201
```

---

## Ringkasan File yang Dibuat/Diupdate di Bab 21

Setelah Bab 21 selesai, berikut status setiap file:

| File | Status | Deskripsi |
|------|--------|-----------|
| `src/main.rs` | ✏️ DIUPDATE | Tambah `mod dto;` + update handler `create_ticket` |
| `src/models/` | ✅ SUDAH ADA | Dari Bab 20 (4 file) |
| `src/dto/mod.rs` | 🆕 BARU | File ini mengexport semua DTO |
| `src/dto/ticket_dto.rs` | 🆕 BARU | CreateTicketDto + UpdateTicketDto dengan validasi |
| `src/dto/user_dto.rs` | 🆕 BARU | RegisterDto + LoginDto dengan validasi |

**Total: 8 file dalam folder src/**

---

Di bab berikutnya (Bab 22), kita akan membuat standar response handling dengan `src/common/response.rs` agar semua endpoint return format yang konsisten dan professional.
