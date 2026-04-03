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
pub use ticket::{CreateTicketDto, CreateTicketResponseDto, Ticket, TicketResponse};
pub use user::User;
```

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
use crate::models::Ticket;
use crate::dto::{CreateTicketDto, UpdateTicketDto};
use crate::common::AppError;

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
    let ticket = sqlx::query_as!(
        Ticket,
        "SELECT * FROM tickets WHERE id = $1",
        id
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(ticket)
}
```

### find_many dengan Filter

Ticket perlu filter yang lebih kompleks, berdasarkan customer, agent, atau status:

```rust
pub async fn find_many(
    &self,
    customer_id: Option<Uuid>,
    agent_id: Option<Uuid>,
    status: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<(Vec<Ticket>, i64), AppError> {
    let offset = (page - 1) * limit;

    let tickets = sqlx::query_as!(
        Ticket,
        "SELECT * FROM tickets
         WHERE ($1::uuid IS NULL OR customer_id = $1)
           AND ($2::uuid IS NULL OR agent_id = $2)
           AND ($3::text IS NULL OR status::text = $3)
         ORDER BY created_at DESC
         LIMIT $4 OFFSET $5",
        customer_id, agent_id, status, limit, offset
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tickets
         WHERE ($1::uuid IS NULL OR customer_id = $1)
           AND ($2::uuid IS NULL OR agent_id = $2)
           AND ($3::text IS NULL OR status::text = $3)",
        customer_id, agent_id, status
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .unwrap_or(0);

    Ok((tickets, total))
}
```

### create

```rust
pub async fn create(
    &self,
    dto: &CreateTicketDto,
    customer_id: Uuid,
) -> Result<Ticket, AppError> {
    let ticket = sqlx::query_as!(
        Ticket,
        "INSERT INTO tickets (customer_id, category, priority, subject, description, tags)
         VALUES ($1, $2::ticket_category, $3::ticket_priority, $4, $5, $6)
         RETURNING *",
        customer_id,
        dto.category,
        dto.priority,
        dto.subject,
        dto.description,
        &dto.tags as &[String]
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(ticket)
}
```

> `&dto.tags as &[String]`: cara memberi tahu sqlx bahwa kita passing array PostgreSQL (`text[]`).

### update

Update hanya field yang dikirim (partial update). Kita pakai `COALESCE`, yaitu fungsi SQL yang mengambil nilai pertama yang bukan NULL:

```rust
pub async fn update(
    &self,
    id: Uuid,
    dto: &UpdateTicketDto,
) -> Result<Option<Ticket>, AppError> {
    let ticket = sqlx::query_as!(
        Ticket,
        "UPDATE tickets SET
            agent_id = COALESCE($2, agent_id),
            status = COALESCE($3::ticket_status, status),
            priority = COALESCE($4::ticket_priority, priority)
         WHERE id = $1
         RETURNING *",
        id,
        dto.agent_id,
        dto.status.as_deref(),
        dto.priority.as_deref(),
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(ticket)
}
```

`COALESCE($2, agent_id)` artinya: kalau `$2` (nilai baru) ada isinya, pakai itu. Kalau `NULL`, pertahankan nilai `agent_id` yang lama.

[ILUSTRASI: Tabel sebelum/sesudah update, hanya kolom yang dikirim berubah, kolom lain tetap sama. Seperti mengisi formulir perubahan data, cukup tulis field yang mau diganti]

---

## Integrasi ke AppState

Repository perlu dimasukkan ke `AppState` supaya bisa diakses dari handler:

```rust
// src/state.rs
use sqlx::PgPool;
use crate::repositories::{UserRepository, TicketRepository};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            ticket_repo: TicketRepository::new(pool.clone()),
            pool,
        }
    }
}
```

Jangan lupa daftarkan module di `src/repositories/mod.rs`:

```rust
pub mod user_repository;
pub mod ticket_repository;

pub use user_repository::UserRepository;
pub use ticket_repository::TicketRepository;
```

Dan di `src/main.rs`, saat membuat AppState:

```rust
let state = AppState::new(pool);
```

---

## Latihan

1. Tambahkan method `find_by_id` ke `UserRepository` dan test dengan user yang ada di database.

2. Modifikasi `find_many` di `TicketRepository` untuk mendukung filter tambahan berdasarkan `category`. Hint: tambah parameter `category: Option<&str>` dan tambahkan kondisi di WHERE clause.

3. Buat method baru `delete` di `UserRepository`:
   ```rust
   pub async fn delete(&self, id: Uuid) -> Result<bool, AppError>
   ```
   Method ini return `true` kalau user ditemukan dan dihapus, `false` kalau user tidak ditemukan. Hint: pakai `DELETE FROM users WHERE id = $1 RETURNING id` dan cek apakah ada baris yang dikembalikan.

4. Coba sengaja typo nama kolom di salah satu query, misalnya `emaill` instead of `email`. Compile dan perhatikan error yang muncul. Ini bukti nyata manfaat compile-time checking dari sqlx.

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
pub use ticket::{CreateTicketDto, CreateTicketResponseDto, Ticket, TicketResponse};
pub use user::User;
```

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

    // Methods dari latihan Bab 25:
    // - find_by_id(id: Uuid)
    // - find_by_email(email: &str)
    // - create(name, email, password, role)
    // - find_all(page, limit)
    // - delete(id)
    // Helper functions: parse_role(&str), format_role(UserRole)
}
```

**File: `src/repositories/ticket_repository.rs`** - Struktur lengkap

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Ticket, TicketStatus, TicketPriority, TicketCategory};
use crate::dto::{CreateTicketDto, UpdateTicketDto};
use crate::common::AppError;

#[derive(Clone)]
pub struct TicketRepository {
    pool: PgPool,
}

impl TicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Methods dari latihan Bab 25:
    // - find_by_id(id: Uuid)
    // - find_many(customer_id, agent_id, status, page, limit)
    // - create(dto: CreateTicketDto, customer_id)
    // - update(id, dto: UpdateTicketDto)
    // - delete(id)
    // Helper functions: parse_category(&str), parse_priority(&str), parse_status(&str)
}
```

**Update: `src/main.rs`**
Tambahkan di paling atas (dengan mod declarations lain):
```rust
mod repositories;
```

---

## Kesimpulan Bab 25

Bab ini mengenalkan **Repository Pattern** untuk mengelola database queries. Benefit:

1. **Separation of Concerns**: Queries terpusat di satu tempat
2. **Type Safety**: Enum types mencegah invalid values masuk ke database
3. **Reusability**: Handler bisa panggil repository methods tanpa tahu detail SQL
4. **Testability**: Repository bisa di-mock untuk testing

**Status Build**: ✅ Berhasil compile (0 errors)

Bab berikutnya: **Integrasi Repository ke Handler** - Menggunakan repositories di HTTP endpoint handlers
