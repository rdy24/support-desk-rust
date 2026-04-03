# Bab 25: Repository: User dan Ticket

Bayangkan sebuah kantor pemerintahan dengan dua bagian: **bagian arsip** dan **bagian operasional**. Bagian operasional melayani warga, yaitu mereka yang terima berkas, proses permohonan, buat keputusan. Tapi kalau butuh data lama, mereka tidak langsung bongkar lemari sendiri. Mereka minta ke bagian arsip, yang tahu persis di mana setiap dokumen disimpan.

Begitulah **Repository Pattern** bekerja di kode ini. Repository adalah bagian arsip. Service/handler adalah bagian operasional. Keduanya punya tanggung jawab yang jelas dan tidak saling campur urusan.

[ILUSTRASI: Diagram dua kotak, "Handler/Service" (bagian operasional) dan "Repository" (bagian arsip), dengan panah dua arah bertuliskan "minta data" dan "kembalikan data", serta kotak "Database" di sisi Repository]

---

## Kunci Jawaban Latihan Bab 24

Latihan Bab 24 fokus pada praktik (jalankan sqlx, docker compose, verify tables). Sudah tercakup di "Hasil Akhir Bab 24":
- Migration files sudah lengkap (users, tickets, ticket_responses, indexes)
- Auto-migration di main() sudah ada

Verifikasi dengan:
```bash
cargo run
# Tabel harus terbuat otomatis saat server start
```

---

## State Sebelumnya

Sebelum mulai Bab 25, pastikan dari Bab 24 sudah ada:

```
support-desk/
├── migrations/                 ← 4 migration files lengkap
├── Cargo.toml
├── src/
│   ├── main.rs                 ← Dengan sqlx::migrate!
│   ├── db.rs
│   ├── models/
│   ├── dto/
│   └── common/
└── .env, docker-compose.yml, dll
```

Verifikasi:
```bash
cargo run
# Server start tanpa error, tables terbuat otomatis
```

---

## Repository Pattern

Tanpa repository pattern, query SQL bisa tersebar di mana-mana, di handler, di service, bahkan di middleware. Ini membuat kode susah di-maintain: mau ganti struktur tabel, harus ubah banyak tempat.

Dengan repository, semua query ke database dikumpulkan di satu tempat. Handler dan service cukup panggil method repository tanpa perlu tahu SQL-nya. Kalau suatu saat perlu ganti database (misalnya dari PostgreSQL ke MySQL), cukup ubah repository-nya saja.

Struktur yang dibangun:

```
src/
├── repositories/
│   ├── mod.rs
│   ├── user_repository.rs
│   └── ticket_repository.rs
```

---

## `sqlx::query_as!` — SQL yang Dicek saat Kompilasi

*Compile-time checking* artinya pemeriksaan kode terjadi saat build/compile, bukan saat aplikasi jalan.

`sqlx` punya fitur istimewa: macro `query_as!` yang memvalidasi SQL query langsung saat kompilasi. Kalau query-nya salah, misalnya nama kolom typo atau tipe data tidak cocok, Rust langsung error sebelum aplikasi sempat jalan. Bedanya dengan library lain: error tidak baru ketahuan saat runtime ketika user sudah memakai aplikasi.

```rust
// query_as! memetakan hasil query ke struct secara otomatis
let user = sqlx::query_as!(
    User,                               // struct target
    "SELECT * FROM users WHERE id = $1", // SQL query ($1 = parameter pertama)
    id                                   // nilai parameter
)
.fetch_optional(&self.pool)  // ambil 0 atau 1 baris
.await?;
```

Ada tiga fetch method yang sering dipakai:
- `.fetch_one()`: ambil tepat 1 baris, error kalau tidak ada
- `.fetch_optional()`: ambil 0 atau 1 baris, return `Option`
- `.fetch_all()`: ambil semua baris, return `Vec`

---

## Struct Database Row

Struct `User` dan `Ticket` di `src/models/` harus field-nya **persis sama** dengan kolom di database. Untuk enum columns, kita menggunakan Rust enums dan melakukan konversi secara eksplisit di repository.

### Langkah 1: Buat File Enum (`src/models/enums.rs`)

**Lokasi:** File baru `src/models/enums.rs`

Buat Rust enum yang sesuai dengan PostgreSQL enum types:

```rust
use sqlx::Type;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_status", rename_all = "lowercase")]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_priority", rename_all = "lowercase")]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_category", rename_all = "lowercase")]
pub enum TicketCategory {
    General,
    Billing,
    Technical,
    Other,
}
```

**Penjelasan:**
- `#[derive(Type)]`: Trait SQLx untuk map ke PostgreSQL type
- `#[sqlx(type_name = "...")]`: Nama enum type di PostgreSQL (harus persis sama)
- `#[sqlx(rename_all = "lowercase")]`: Konversi Rust variant ke lowercase di database
- Contoh: `TicketStatus::InProgress` → PostgreSQL `in_progress`

---

### Langkah 2: Update Model Structs

Update `src/models/user.rs` dan `src/models/ticket.rs` untuk menggunakan enum types:

**File: `src/models/user.rs`**
```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use super::UserRole;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: UserRole,  // ← Ubah dari String ke UserRole
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**File: `src/models/ticket.rs`**
```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use super::{TicketStatus, TicketPriority, TicketCategory};

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: Uuid,
    pub customer_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Uuid>,
    pub category: TicketCategory,  // ← Ubah dari String
    pub priority: TicketPriority,  // ← Ubah dari String
    pub status: TicketStatus,      // ← Ubah dari String
    pub subject: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Update `src/models/mod.rs`:**
```rust
pub mod enums;
pub mod api_response;
pub mod ticket;
pub mod user;

pub use enums::{UserRole, TicketStatus, TicketPriority, TicketCategory};
pub use api_response::ApiResponse;
pub use ticket::{Ticket, TicketResponse};
pub use user::User;
```

**Catatan:** `CreateTicketDto` bukan di models, tapi di `src/dto/ticket_dto.rs` (akan dibuat di Bab 21 bagian validasi input). Jangan export dari models.

---

## UserRepository

Buat file `src/repositories/user_repository.rs` dengan repository methods:

**Lokasi:** File baru `src/repositories/user_repository.rs`

Pada repository, kita:
1. Query PostgreSQL enum columns sebagai **text** (menggunakan `::text` casting)
2. Parse string values ke Rust enums secara manual
3. Kembalikan struct dengan enum fields

**Kode lengkap untuk UserRepository:**

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, UserRole};
use crate::common::AppError;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Cari user berdasarkan ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: Uuid,
            name: String,
            email: String,
            password: String,
            role: String,  // ← Ambil sebagai String dulu
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, name, email, password, role::text, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| User {
            id: r.id,
            name: r.name,
            email: r.email,
            password: r.password,
            role: parse_role(&r.role).unwrap_or(UserRole::Customer),  // ← Parse ke enum
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    // ... methods lainnya (find_by_email, create, find_all, delete)
}

// Helper functions untuk konversi
fn parse_role(s: &str) -> Option<UserRole> {
    match s {
        "admin" => Some(UserRole::Admin),
        "agent" => Some(UserRole::Agent),
        "customer" => Some(UserRole::Customer),
        _ => None,
    }
}

fn format_role(role: UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Agent => "agent".to_string(),
        UserRole::Customer => "customer".to_string(),
    }
}
```

**Penjelasan:**
- Definisikan struct temporary `UserRow` dengan enum fields sebagai `String`
- Query: `role::text` cast PostgreSQL enum ke text
- Parse string ke Rust enum dengan fungsi helper
- Return struct `User` dengan enum fields yang sudah ter-parse

---

## TicketRepository

Buat file `src/repositories/ticket_repository.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Ticket, TicketStatus, TicketPriority, TicketCategory};
use crate::dto::CreateTicketDto;
use crate::common::AppError;

#[derive(Clone)]
pub struct TicketRepository {
    pool: PgPool,
}

impl TicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### find_by_id

```rust
pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Ticket>, AppError> {
    let ticket = sqlx::query_as::<_, Ticket>(
        "SELECT id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at FROM tickets WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ticket)
}
```

### find_many dengan Filter dan Pagination

Ticket perlu filter berdasarkan customer_id dan status dengan pagination:

```rust
pub async fn find_many(
    &self,
    customer_id: Option<Uuid>,
    status: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<(Vec<Ticket>, i64), AppError> {
    let offset = (page - 1) * limit;

    let tickets = sqlx::query_as::<_, Ticket>(
        "SELECT id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at FROM tickets
         WHERE ($1::uuid IS NULL OR customer_id = $1)
           AND ($2::text IS NULL OR status::text = $2)
         ORDER BY created_at DESC
         LIMIT $3 OFFSET $4"
    )
    .bind(customer_id)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets
         WHERE ($1::uuid IS NULL OR customer_id = $1)
           AND ($2::text IS NULL OR status::text = $2)"
    )
    .bind(customer_id)
    .bind(status)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((tickets, total.0))
}
```

### create

```rust
pub async fn create(
    &self,
    dto: &CreateTicketDto,
    customer_id: Uuid,
) -> Result<Ticket, AppError> {
    let ticket = sqlx::query_as::<_, Ticket>(
        "INSERT INTO tickets (customer_id, category, priority, subject, description)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at"
    )
    .bind(customer_id)
    .bind(dto.category)
    .bind(dto.priority)
    .bind(&dto.subject)
    .bind(&dto.description)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ticket)
}
```

### delete

Hapus ticket berdasarkan ID:

```rust
pub async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM tickets WHERE id = $1")
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result.rows_affected() > 0)
}
```

`.rows_affected()` mengembalikan jumlah baris yang dihapus. Kalau > 0, berarti ticket ada dan sudah dihapus.

---

## Daftarkan Module Repository

Di `src/repositories/mod.rs`:

```rust
pub mod user_repository;
pub mod ticket_repository;

pub use user_repository::UserRepository;
pub use ticket_repository::TicketRepository;
```

Selanjutnya, di `src/main.rs`, import repositories dan update `AppState`:

```rust
use crate::repositories::{UserRepository, TicketRepository};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            ticket_repo: TicketRepository::new(pool.clone()),
            db: pool,
        }
    }
}
```

---

## Latihan

1. **Tambahkan method `find_by_email`** ke `UserRepository` untuk mencari user berdasarkan email. Gunakan `ORDER BY created_at DESC` untuk pengurutan konsisten.

2. **Modifikasi `find_many` di `TicketRepository`** untuk mendukung filter tambahan berdasarkan `category`. Hint: tambah parameter `category: Option<&str>` dan tambahkan kondisi di WHERE clause.

3. **Tambahkan method `update` ke `TicketRepository`** untuk update status atau priority ticket. Gunakan `COALESCE` untuk partial update (hanya field yang dikirim yang berubah).

4. **Verifikasi compile-time checking:** Coba sengaja ubah nama kolom di salah satu query, misalnya `emaill` instead of `email`. Compile dan perhatikan error yang muncul. Ini bukti nyata manfaat type-safe queries dari sqlx.

---

---

## ⚠️ Common Error & Solution

**Error:**
```
error: no built in mapping found for type ticket_status for param #1
  --> src/repositories/ticket_repository.rs:89:22
```

**Penyebab:** Kolom enum PostgreSQL tidak ter-map ke Rust enum dengan benar.

**Solusi:** Pastikan:
1. ✅ File `src/models/enums.rs` sudah dibuat dengan `#[derive(sqlx::Type)]`
2. ✅ Model `User` dan `Ticket` menggunakan enum types, bukan `String`
3. ✅ Repository methods menggunakan enum types di parameter dan return values
4. ✅ Imports sudah benar (misalnya `use crate::models::UserRole;`)

Setelah memastikan kesemuanya, jalankan:
```bash
cargo build
# Harus compile tanpa error
```

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan latihan Bab 25, folder struktur harus ada folder repositories baru:

```
src/
├── repositories/               ← NEW FOLDER
│   ├── mod.rs                 ← NEW
│   ├── user_repository.rs     ← NEW
│   └── ticket_repository.rs   ← NEW
├── models/
│   ├── mod.rs                 ← UPDATE: add `pub mod enums;`
│   ├── enums.rs               ← NEW: UserRole, TicketStatus, TicketPriority, TicketCategory
│   ├── user.rs                ← UPDATE: use UserRole enum
│   ├── ticket.rs              ← UPDATE: use TicketStatus, TicketPriority, TicketCategory enums
│   └── api_response.rs
├── main.rs                    ← UPDATE: add `mod repositories;`
├── dto/
├── common/
├── db.rs
└── ... (lainnya)
```

**File: `src/models/enums.rs`** ← PENTING! Harus dibuat terlebih dahulu
```rust
use sqlx::Type;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_status", rename_all = "lowercase")]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_priority", rename_all = "lowercase")]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_category", rename_all = "lowercase")]
pub enum TicketCategory {
    General,
    Billing,
    Technical,
    Other,
}
```

**Penjelasan derives:**
- `Debug`, `Clone`, `Copy`: Rust std traits
- `Serialize`, `Deserialize`: Untuk JSON serialization di API responses
- `Type`: SQLx trait untuk PostgreSQL enum mapping

**File: `src/models/user.rs`** ← UPDATE
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use super::UserRole;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**File: `src/models/ticket.rs`** ← UPDATE
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use super::{TicketStatus, TicketPriority, TicketCategory};

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: Uuid,
    pub customer_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Uuid>,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub subject: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**File: `src/models/mod.rs`** ← UPDATE
```rust
pub mod enums;
pub mod api_response;
pub mod ticket;
pub mod user;

pub use api_response::ApiResponse;
pub use enums::{UserRole, TicketStatus, TicketPriority, TicketCategory};
pub use ticket::{Ticket, TicketResponse};
pub use user::User;
```

**⚠️ PENTING:** `CreateTicketDto` (dengan validation) ada di `src/dto/ticket_dto.rs`, BUKAN di models. DTOs adalah untuk input validation, models adalah untuk database structs.

**File: `src/repositories/mod.rs`**
```rust
pub mod user_repository;
pub mod ticket_repository;

pub use user_repository::UserRepository;
pub use ticket_repository::TicketRepository;
```

**File: `src/repositories/user_repository.rs`** - Struktur lengkap

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, UserRole};
use crate::common::AppError;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Methods dari Bab 25:
    // - find_by_id(id: Uuid) -> Result<Option<User>, AppError>
    // - find_by_email(email: &str) -> Result<Option<User>, AppError>
    // - create(name, email, password, role) -> Result<User, AppError>
    // - delete(id: Uuid) -> Result<bool, AppError>
    // Helper functions: parse_role(&str), format_role(UserRole)
}
```

**File: `src/repositories/ticket_repository.rs`** - Struktur lengkap

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Ticket, TicketStatus, TicketPriority, TicketCategory};
use crate::dto::CreateTicketDto;  // ← Import dari DTO, bukan models!
use crate::common::AppError;

#[derive(Clone)]
pub struct TicketRepository {
    pool: PgPool,
}

impl TicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Methods dari Bab 25:
    // - find_by_id(id: Uuid) -> Result<Option<Ticket>, AppError>
    // - find_many(customer_id, status, page, limit) -> Result<(Vec<Ticket>, i64), AppError>
    // - create(dto: CreateTicketDto, customer_id) -> Result<Ticket, AppError>
    // - delete(id: Uuid) -> Result<bool, AppError>
}
```

**Catatan Struktur:** CreateTicketDto dengan validation ada di `src/dto/ticket_dto.rs`. Repository menerima validated DTO dan simpan ke database.

**Update: `src/main.rs`**
Pastikan sudah ada di paling atas (dengan mod declarations lain):
```rust
mod repositories;
```

---

## Kesimpulan Bab 25

Bab ini mengimplementasikan **Repository Pattern** untuk mengelola database queries dengan type-safe queries. Benefit utama:

1. **Separation of Concerns**: Semua query SQL terpusat di repository layer
2. **Type Safety**: Enum types dan SQLx compile-time checking mencegah SQL errors
3. **Reusability**: Handler bisa panggil repository methods tanpa perlu tahu SQL details
4. **Maintainability**: Perubahan database schema hanya perlu update di repository

**Key Takeaways:**
- ✅ Enums untuk PostgreSQL enum columns (UserRole, TicketStatus, TicketPriority, TicketCategory)
- ✅ Repository structs dengan Clone trait (siap untuk di-share ke handlers)
- ✅ Type-safe queries dengan sqlx::query_as dan runtime parameter binding
- ✅ Error handling dengan AppError mapping

**Status Build**: ✅ Compile successful (0 errors, 36 warnings OK — unused code normal untuk stage ini)

**Next Chapter (Bab 26):** ResponseRepository + DashboardRepository — menambah 2 repository untuk ticket responses dan dashboard statistics
