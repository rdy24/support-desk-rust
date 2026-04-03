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

Struct `User` dan `Ticket` di `src/models/` harus field-nya **persis sama** dengan kolom di database, termasuk tipe data. `sqlx` akan map kolom ke field berdasarkan nama.

### ⚠️ PENTING: PostgreSQL Enum Mapping

Kolom enum di PostgreSQL (`user_role`, `ticket_status`, dll) harus di-map ke **Rust enum dengan trait `sqlx::Type`**, bukan ke `String`. Tanpa ini, saat query pakai type casting (`$3::ticket_status`), sqlx akan error:

```
error: no built in mapping found for type ticket_status
```

**Solusi: Buat Rust enum dengan `#[derive(sqlx::Type)]`**

```rust
// src/models/enums.rs - FILE BARU
use sqlx::Type;

#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}

#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "ticket_status", rename_all = "lowercase")]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "ticket_priority", rename_all = "lowercase")]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Type)]
#[sqlx(type_name = "ticket_category", rename_all = "lowercase")]
pub enum TicketCategory {
    General,
    Billing,
    Technical,
    Other,
}
```

**Attribute penting:**
- `#[sqlx(type_name = "...")]`: nama enum type di PostgreSQL
- `#[sqlx(rename_all = "lowercase")]`: nama variant di Rust akan diubah ke lowercase saat match ke database

Contoh: Rust `TicketStatus::InProgress` akan di-map ke PostgreSQL `in_progress`.

### Update Structs

```rust
// src/models/user.rs
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::enums::UserRole;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,        // ← UBAH dari String ke UserRole
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

```rust
// src/models/ticket.rs
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::enums::{TicketStatus, TicketPriority, TicketCategory};

#[derive(Debug, sqlx::FromRow)]
pub struct Ticket {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub category: TicketCategory,      // ← UBAH dari String
    pub priority: TicketPriority,      // ← UBAH dari String
    pub status: TicketStatus,          // ← UBAH dari String
    pub subject: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Update `src/models/mod.rs`

```rust
pub mod enums;
pub mod user;
pub mod ticket;

pub use enums::*;
pub use user::User;
pub use ticket::Ticket;
```

---

## UserRepository

Buat file `src/repositories/user_repository.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;
use crate::common::AppError;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### find_by_id

```rust
pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, name, email, password, role, created_at, updated_at FROM users WHERE id = $1",
        id
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(user)
}
```

### find_by_email

Dipakai saat login untuk mencari user berdasarkan email yang diinput:

```rust
pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(user)
}
```

### create

Insert user baru dan langsung kembalikan data yang tersimpan via `RETURNING *`:

```rust
pub async fn create(
    &self,
    name: &str,
    email: &str,
    password: &str,
    role: &str,
) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password, role)
         VALUES ($1, $2, $3, $4::user_role)
         RETURNING *",
        name, email, password, role
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(user)
}
```

> `$4::user_role`: kita cast string ke enum PostgreSQL secara eksplisit. Tanpa cast ini, PostgreSQL bisa complain soal tipe data.

### find_all dengan Pagination

Pagination membagi hasil query jadi halaman-halaman kecil supaya tidak semua data dikembalikan sekaligus.

```rust
pub async fn find_all(
    &self,
    role: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<(Vec<User>, i64), AppError> {
    let offset = (page - 1) * limit;

    let users = sqlx::query_as!(
        User,
        "SELECT * FROM users
         WHERE ($1::text IS NULL OR role::text = $1)
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3",
        role, limit, offset
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users
         WHERE ($1::text IS NULL OR role::text = $1)",
        role
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .unwrap_or(0);

    Ok((users, total))
}
```

Trik SQL `($1::text IS NULL OR role::text = $1)`: kalau `role` adalah `None`, kondisi pertama (`IS NULL`) true, jadi semua user dikembalikan. Kalau `role` ada nilainya, filter berdasarkan role itu.

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
│   ├── ticket_repository.rs   ← NEW
│   └── response_repository.rs ← NEW (untuk TicketResponse)
├── models/
│   ├── mod.rs                 ← UPDATE: add `pub mod enums;`
│   ├── enums.rs               ← NEW: UserRole, TicketStatus, TicketPriority, TicketCategory
│   ├── user.rs                ← UPDATE: use UserRole enum
│   └── ticket.rs              ← UPDATE: use TicketStatus, TicketPriority, TicketCategory enums
├── main.rs
├── dto/
├── common/
├── db.rs
└── ... (lainnya)
```

**File: `src/models/enums.rs`** ← PENTING! Harus dibuat terlebih dahulu
```rust
use sqlx::Type;

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "ticket_status", rename_all = "lowercase")]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "ticket_priority", rename_all = "lowercase")]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "ticket_category", rename_all = "lowercase")]
pub enum TicketCategory {
    General,
    Billing,
    Technical,
    Other,
}
```

**File: `src/models/user.rs`** ← UPDATE
```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use super::UserRole;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**File: `src/models/ticket.rs`** ← UPDATE
```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use super::{TicketStatus, TicketPriority, TicketCategory};

#[derive(Debug, FromRow)]
pub struct Ticket {
    pub id: Uuid,
    pub customer_id: Uuid,
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
pub mod user;
pub mod ticket;

pub use enums::{UserRole, TicketStatus, TicketPriority, TicketCategory};
pub use user::User;
pub use ticket::Ticket;
```

**File: `src/repositories/user_repository.rs`** (include latihan #1 & #3)
```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    models::{User, UserRole},
    common::AppError
};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, name, email, password, role, created_at, updated_at FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, name, email, password, role, created_at, updated_at FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(user)
    }

    pub async fn find_many(&self, limit: i64, offset: i64) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as!(
            User,
            "SELECT id, name, email, password, role, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(users)
    }

    pub async fn create(
        &self,
        name: &str,
        email: &str,
        password: &str,
        role: UserRole,   // ← UBAH dari &str ke UserRole
    ) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (name, email, password, role) VALUES ($1, $2, $3, $4) RETURNING id, name, email, password, role, created_at, updated_at",
            name,
            email,
            password,
            role                // ← sqlx otomatis handle konversi enum ke PostgreSQL
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(user)
    }

    // Latihan #3: delete method
    pub async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1 RETURNING id",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(result.is_some())
    }
}
```

**File: `src/repositories/ticket_repository.rs`** (include latihan #2)
```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    models::{Ticket, TicketCategory, TicketPriority, TicketStatus},
    common::AppError
};

pub struct TicketRepository {
    pool: PgPool,
}

impl TicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Ticket>, AppError> {
        let ticket = sqlx::query_as!(
            Ticket,
            "SELECT id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at FROM tickets WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ticket)
    }

    // Latihan #2: find_many dengan category filter
    pub async fn find_many(
        &self,
        limit: i64,
        offset: i64,
        category: Option<TicketCategory>,  // ← UBAH dari Option<&str>
    ) -> Result<Vec<Ticket>, AppError> {
        let tickets = sqlx::query_as!(
            Ticket,
            "SELECT id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at FROM tickets 
             WHERE ($1::ticket_category IS NULL OR category = $1)
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            category,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(tickets)
    }

    pub async fn create(
        &self,
        customer_id: Uuid,
        category: TicketCategory,      // ← UBAH dari &str
        priority: TicketPriority,      // ← UBAH dari &str
        subject: &str,
        description: &str,
    ) -> Result<Ticket, AppError> {
        let ticket = sqlx::query_as!(
            Ticket,
            "INSERT INTO tickets (customer_id, category, priority, subject, description) VALUES ($1, $2, $3, $4, $5) RETURNING id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at",
            customer_id,
            category,      // ← sqlx otomatis handle konversi enum
            priority,      // ← sqlx otomatis handle konversi enum
            subject,
            description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ticket)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: TicketStatus,  // ← UBAH dari &str
    ) -> Result<Ticket, AppError> {
        let ticket = sqlx::query_as!(
            Ticket,
            "UPDATE tickets SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at",
            status,    // ← sqlx otomatis handle konversi enum
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ticket)
    }
}
```

**File: `src/repositories/response_repository.rs`** (untuk TicketResponse)
```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::{models::TicketResponse, common::AppError};

pub struct ResponseRepository {
    pool: PgPool,
}

impl ResponseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_ticket(&self, ticket_id: Uuid) -> Result<Vec<TicketResponse>, AppError> {
        let responses = sqlx::query_as!(
            TicketResponse,
            "SELECT id, ticket_id, user_id, message, created_at FROM ticket_responses WHERE ticket_id = $1 ORDER BY created_at DESC",
            ticket_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(responses)
    }

    pub async fn create(
        &self,
        ticket_id: Uuid,
        user_id: Uuid,
        message: &str,
    ) -> Result<TicketResponse, AppError> {
        let response = sqlx::query_as!(
            TicketResponse,
            "INSERT INTO ticket_responses (ticket_id, user_id, message) VALUES ($1, $2, $3) RETURNING id, ticket_id, user_id, message, created_at",
            ticket_id,
            user_id,
            message
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(response)
    }
}
```

**File: `src/repositories/mod.rs`**
```rust
pub mod response_repository;
pub mod ticket_repository;
pub mod user_repository;

pub use response_repository::ResponseRepository;
pub use ticket_repository::TicketRepository;
pub use user_repository::UserRepository;
```

**Update File: `src/main.rs`** — Tambahkan mod repositories dan update AppState:
```rust
mod repositories;

use repositories::{ResponseRepository, TicketRepository, UserRepository};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
}

// Di main():
let pool = create_pool(&database_url).await;
sqlx::migrate!("./migrations").run(&pool).await.expect("Failed migrations");

let state = AppState {
    db: pool.clone(),
    user_repo: UserRepository::new(pool.clone()),
    ticket_repo: TicketRepository::new(pool.clone()),
    response_repo: ResponseRepository::new(pool.clone()),
};
```

**Verifikasi:**
```bash
cargo build  # Harus compile tanpa error (sqlx compile-time checking)
cargo run    # Server start, repositories ready untuk dipakai di Bab 26
```

Repository layer sekarang siap untuk dipakai di service layer dan handlers di bab-bab berikutnya.
