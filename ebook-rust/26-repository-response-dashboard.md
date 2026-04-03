# Bab 26: Repository: Response dan Dashboard

Bayangkan kamu kerja di sebuah call center. Ada dua buku catatan berbeda di meja resepsionis:

1. **Buku tamu percakapan:** setiap kali ada balasan untuk tiket tertentu, dicatat di sini. Siapa yang balas, isinya apa, kapan.
2. **Papan statistik kantor:** di dinding, ada papan besar yang menampilkan ringkasan: berapa tiket yang masuk hari ini, berapa yang sudah selesai, berapa agent yang aktif.

Kedua buku ini punya sifat yang sangat berbeda. Buku tamu = insert dan lookup per tiket. Papan statistik = hitung-hitungan besar dari seluruh data.

---

## State Awal Bab 26

Dari Bab 25, sudah ada:
- ✅ `UserRepository` dan `TicketRepository` dengan semua methods
- ✅ Enum types di `src/models/enums.rs`
- ✅ Models `User` dan `Ticket` menggunakan enum types
- ✅ Folder `src/repositories/` dengan `mod.rs`
- ✅ `AppState` di `src/main.rs` (akan di-update)

Verifikasi:
```bash
cargo build
# Harus 0 errors
```

---

## Persiapan: Tambah Model TicketResponse

Sebelum buat ResponseRepository, kita perlu model `TicketResponse` di `src/models/ticket.rs`:

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

Update `src/models/mod.rs`:
```rust
pub use ticket::{Ticket, TicketResponse};
```

---

## Persiapan: Repositories Harus Clone

`UserRepository` dan `TicketRepository` dari Bab 25 HARUS punya `#[derive(Clone)]`:

```rust
#[derive(Clone)]
pub struct UserRepository { ... }

#[derive(Clone)]
pub struct TicketRepository { ... }
```

Kenapa? Karena bab ini akan wrap keempat repositories di `AppState`, yang sendiri derive `Clone`. PgPool adalah Arc internally, jadi aman di-clone.

---

## ⚠️ Important: Struktur DTO vs Models

Sebelum buat ResponseRepository, perhatikan struktur yang benar:

- **Models** (`src/models/`) = Database-aligned structs
  - `User`, `Ticket`, `TicketResponse` — fields persis sesuai database columns
- **DTO** (`src/dto/`) = Request/Response validation structs
  - `CreateTicketDto` (dengan validation), `LoginDto`, dll — untuk input validation

**Repository menggunakan keduanya:**
```rust
// Menerima DTO (sudah validated dari handler)
pub async fn create(&self, dto: &CreateTicketDto, customer_id: Uuid) -> Result<Ticket, AppError>
// Return Model yang di-query dari database
```

---

## ResponseRepository

Buat file baru: `src/repositories/response_repository.rs`

### Struct dan Constructor

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::TicketResponse;
use crate::common::AppError;

#[derive(Clone)]
pub struct ResponseRepository {
    pool: PgPool,
}

impl ResponseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

**Penjelasan:** `ResponseRepository` mengikuti pola yang sama seperti `UserRepository` dan `TicketRepository` dari Bab 25. Struct hanya wrap `PgPool`, dan konstruktor membuat instance baru.

### Method `create` — Simpan Balasan Baru

```rust
pub async fn create(
    &self,
    ticket_id: Uuid,
    user_id: Uuid,
    message: String,
) -> Result<TicketResponse, AppError> {
    #[derive(sqlx::FromRow)]
    struct ResponseRow {
        id: Uuid,
        ticket_id: Uuid,
        user_id: Uuid,
        message: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let row = sqlx::query_as::<_, ResponseRow>(
        r#"INSERT INTO ticket_responses (ticket_id, user_id, message)
           VALUES ($1, $2, $3)
           RETURNING id, ticket_id, user_id, message, created_at"#
    )
    .bind(ticket_id)
    .bind(user_id)
    .bind(&message)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(TicketResponse {
        id: row.id,
        ticket_id: row.ticket_id,
        user_id: row.user_id,
        message: row.message,
        created_at: row.created_at,
    })
}
```

**Penjelasan:**
- Gunakan temporary struct `ResponseRow` dengan `#[derive(sqlx::FromRow)]` untuk mapping database row ke Rust struct
- `RETURNING *` atau (lebih eksplisit) `RETURNING id, ticket_id, ...` mengembalikan baris baru yang di-insert
- Struct `TicketResponse` di `src/models/ticket.rs` tidak derive `sqlx::FromRow` (hanya `Serialize, Deserialize`), jadi kita gunakan `ResponseRow` sebagai perantara
- Error mapping: `.map_err(|e| AppError::Internal(e.to_string()))?`

### Method `find_by_ticket_id` — Ambil Semua Balasan

```rust
pub async fn find_by_ticket_id(
    &self,
    ticket_id: Uuid,
) -> Result<Vec<TicketResponse>, AppError> {
    #[derive(sqlx::FromRow)]
    struct ResponseRow {
        id: Uuid,
        ticket_id: Uuid,
        user_id: Uuid,
        message: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows = sqlx::query_as::<_, ResponseRow>(
        r#"SELECT id, ticket_id, user_id, message, created_at
           FROM ticket_responses
           WHERE ticket_id = $1
           ORDER BY created_at ASC"#
    )
    .bind(ticket_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(rows.into_iter().map(|r| TicketResponse {
        id: r.id,
        ticket_id: r.ticket_id,
        user_id: r.user_id,
        message: r.message,
        created_at: r.created_at,
    }).collect())
}
```

**Penjelasan:**
- `.fetch_all()` mengembalikan `Vec<ResponseRow>`
- Urutan `ORDER BY created_at ASC` = paling lama duluan (urut kronologis)
- `.into_iter().map(...)` mengonversi vector `ResponseRow` ke vector `TicketResponse`

---

## DashboardRepository

Buat file baru: `src/repositories/dashboard_repository.rs`

Dashboard merupakan aggregate queries — hitung jumlah tiket per status, hitung user per role, dll. Bukan operasi sederhana per-record.

### Struct `DashboardStats`

```rust
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_tickets: i64,
    pub open_tickets: i64,
    pub in_progress_tickets: i64,
    pub resolved_tickets: i64,
    pub closed_tickets: i64,
    pub total_users: i64,
    pub total_agents: i64,
    pub total_customers: i64,
    pub avg_responses_per_ticket: f64,
}
```

**Penjelasan:**
- `COUNT(*)` di PostgreSQL mengembalikan `BIGINT`, yang sqlx map ke `i64` di Rust
- Gunakan `i64`, bukan `i32` (akan compile error kalau tipe tidak sesuai)
- Derive `Serialize` agar bisa di-convert ke JSON untuk HTTP response
- `#[serde(rename_all = "camelCase")]` agar di JSON muncul sebagai `totalTickets`, `openTickets`, dll (sesuai API convention)

### Struct `DashboardRepository`

```rust
use sqlx::PgPool;
use crate::common::AppError;

#[derive(Clone)]
pub struct DashboardRepository {
    pool: PgPool,
}

impl DashboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### Method `get_stats` — Query Aggregate

Ini adalah method paling penting. Dashboard biasanya query data agregat — hitung jumlah, rata-rata, dsb. PostgreSQL punya teknik efisien untuk ini: `COUNT(*) FILTER (WHERE ...)`.

```rust
pub async fn get_stats(&self) -> Result<DashboardStats, AppError> {
    // --- Statistik Tiket ---
    #[derive(sqlx::FromRow)]
    struct TicketStatsRow {
        total_tickets: i64,
        open_tickets: i64,
        in_progress_tickets: i64,
        resolved_tickets: i64,
        closed_tickets: i64,
    }

    let ticket_stats = sqlx::query_as::<_, TicketStatsRow>(
        r#"SELECT
            COUNT(*) AS total_tickets,
            COUNT(*) FILTER (WHERE status = 'open') AS open_tickets,
            COUNT(*) FILTER (WHERE status = 'in_progress') AS in_progress_tickets,
            COUNT(*) FILTER (WHERE status = 'resolved') AS resolved_tickets,
            COUNT(*) FILTER (WHERE status = 'closed') AS closed_tickets
        FROM tickets"#
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // --- Statistik User ---
    #[derive(sqlx::FromRow)]
    struct UserStatsRow {
        total_users: i64,
        total_agents: i64,
        total_customers: i64,
    }

    let user_stats = sqlx::query_as::<_, UserStatsRow>(
        r#"SELECT
            COUNT(*) AS total_users,
            COUNT(*) FILTER (WHERE role = 'agent') AS total_agents,
            COUNT(*) FILTER (WHERE role = 'customer') AS total_customers
        FROM users"#
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // --- Rata-rata Responses Per Tiket ---
    let total_responses: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ticket_responses"
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let avg_responses_per_ticket = if ticket_stats.total_tickets > 0 {
        total_responses as f64 / ticket_stats.total_tickets as f64
    } else {
        0.0
    };

    Ok(DashboardStats {
        total_tickets: ticket_stats.total_tickets,
        open_tickets: ticket_stats.open_tickets,
        in_progress_tickets: ticket_stats.in_progress_tickets,
        resolved_tickets: ticket_stats.resolved_tickets,
        closed_tickets: ticket_stats.closed_tickets,
        total_users: user_stats.total_users,
        total_agents: user_stats.total_agents,
        total_customers: user_stats.total_customers,
        avg_responses_per_ticket,
    })
}
```

**Penjelasan:**

**`COUNT(*) FILTER (WHERE ...)`:**
- Teknik PostgreSQL untuk conditional aggregation
- Satu scan tabel, menghitung beberapa counter sekaligus
- Lebih efisien dari 5 query terpisah
- `COUNT(*)` tanpa FILTER = total baris, dengan FILTER = hitung baris yang memenuhi kondisi

**`sqlx::query_scalar`:**
- Ketika query cuma return satu nilai (scalar), bukan struct
- `.fetch_one()` mengembalikan `i64` langsung, bukan wrapped dalam struct

**Division by Zero:**
- Cegah dengan check `if ticket_stats.total_tickets > 0`
- Kalau zero, return `0.0` daripada panic

---

## Daftarkan Repository Baru

### Update `src/repositories/mod.rs`

```rust
pub mod user_repository;
pub mod ticket_repository;
pub mod response_repository;
pub mod dashboard_repository;

pub use user_repository::UserRepository;
pub use ticket_repository::TicketRepository;
pub use response_repository::ResponseRepository;
pub use dashboard_repository::{DashboardRepository, DashboardStats};
```

### Update `src/main.rs` — AppState dan Constructor

Di bagian atas file (setelah imports), update `AppState`:

```rust
use crate::repositories::{
    UserRepository, TicketRepository, ResponseRepository, DashboardRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            ticket_repo: TicketRepository::new(pool.clone()),
            response_repo: ResponseRepository::new(pool.clone()),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            db: pool,
        }
    }
}
```

**Penjelasan:**
- Semua repository di-wrap di `AppState` untuk easy injection ke handlers nanti
- `#[derive(Clone)]` on `UserRepository`, `TicketRepository`, `ResponseRepository`, `DashboardRepository` (semua struct hanya wrap `PgPool`, dan `PgPool` is Clone)
- Constructor `AppState::new(pool)` membuat semua repository sekaligus
- `pool.clone()` aman karena `PgPool` adalah Arc internally, clone-nya cuma reference increment

---

## Latihan Opsional

**Sudah diimplementasikan:** 
- ✅ Method `find_latest_by_ticket_id` di ResponseRepository
- ✅ Field `avg_responses_per_ticket: f64` di DashboardStats dengan division by zero handling

**Challenge Tambahan:**

1. **Optimasi query dashboard:** Coba gabungkan `ticket_stats` dan `user_stats` menjadi satu query dengan CROSS JOIN (atau WITH clause). Bandingkan keterbacaan kode dengan versi dua query terpisah. Mana yang lebih maintainable?

2. **Add logging untuk errors:** Di setiap method, tambahkan `eprintln!` sebelum return error untuk debugging. Contoh:
   ```rust
   .map_err(|e| {
       eprintln!("Database error in ResponseRepository::create: {}", e);
       AppError::Internal(e.to_string())
   })?
   ```

3. **Tambah method `count_responses_for_ticket`** di ResponseRepository untuk menghitung jumlah balasan per tiket tanpa load semua data.

4. **Eksperimen dengan DashboardRepository:** Coba split `get_stats` menjadi beberapa method terpisah (`get_ticket_stats()`, `get_user_stats()`, `get_response_stats()`) untuk lebih modular. Pro dan kontra dari split ini?

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan latihan Bab 26, struktur folder dan file harus seperti ini:

```
src/
├── repositories/           ← UPDATED & EXPANDED
│   ├── mod.rs              ← UPDATE: tambah response & dashboard modules
│   ├── user_repository.rs  ← UPDATE: tambah #[derive(Clone)]
│   ├── ticket_repository.rs ← UPDATE: tambah #[derive(Clone)]
│   ├── response_repository.rs ← NEW
│   └── dashboard_repository.rs ← NEW
├── models/
│   ├── mod.rs
│   ├── enums.rs
│   ├── user.rs
│   ├── ticket.rs
│   └── api_response.rs
├── main.rs                 ← UPDATE: AppState + constructor
├── dto/
├── common/
└── db.rs
```

### File: `src/repositories/user_repository.rs` & `ticket_repository.rs` — Sudah Updated dari Bab 25

Kedua file sudah punya `#[derive(Clone)]` dari Bab 25. Tidak perlu diubah lagi.

### File: `src/repositories/response_repository.rs` — BARU

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::TicketResponse;
use crate::common::AppError;

/// Repository untuk mengelola ticket responses (balasan tiket)
#[derive(Clone)]
pub struct ResponseRepository {
    pool: PgPool,
}

impl ResponseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Simpan balasan baru untuk tiket
    pub async fn create(
        &self,
        ticket_id: Uuid,
        user_id: Uuid,
        message: String,
    ) -> Result<TicketResponse, AppError> {
        #[derive(sqlx::FromRow)]
        struct ResponseRow {
            id: Uuid,
            ticket_id: Uuid,
            user_id: Uuid,
            message: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, ResponseRow>(
            r#"INSERT INTO ticket_responses (ticket_id, user_id, message)
               VALUES ($1, $2, $3)
               RETURNING id, ticket_id, user_id, message, created_at"#
        )
        .bind(ticket_id)
        .bind(user_id)
        .bind(&message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(TicketResponse {
            id: row.id,
            ticket_id: row.ticket_id,
            user_id: row.user_id,
            message: row.message,
            created_at: row.created_at,
        })
    }

    /// Ambil semua balasan untuk satu tiket, urut dari paling lama
    pub async fn find_by_ticket_id(
        &self,
        ticket_id: Uuid,
    ) -> Result<Vec<TicketResponse>, AppError> {
        #[derive(sqlx::FromRow)]
        struct ResponseRow {
            id: Uuid,
            ticket_id: Uuid,
            user_id: Uuid,
            message: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let rows = sqlx::query_as::<_, ResponseRow>(
            r#"SELECT id, ticket_id, user_id, message, created_at
               FROM ticket_responses
               WHERE ticket_id = $1
               ORDER BY created_at ASC"#
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| TicketResponse {
            id: r.id,
            ticket_id: r.ticket_id,
            user_id: r.user_id,
            message: r.message,
            created_at: r.created_at,
        }).collect())
    }

    /// (Latihan #1) Ambil balasan terbaru untuk satu tiket
    pub async fn find_latest_by_ticket_id(
        &self,
        ticket_id: Uuid,
    ) -> Result<Option<TicketResponse>, AppError> {
        #[derive(sqlx::FromRow)]
        struct ResponseRow {
            id: Uuid,
            ticket_id: Uuid,
            user_id: Uuid,
            message: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, ResponseRow>(
            r#"SELECT id, ticket_id, user_id, message, created_at
               FROM ticket_responses
               WHERE ticket_id = $1
               ORDER BY created_at DESC
               LIMIT 1"#
        )
        .bind(ticket_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| TicketResponse {
            id: r.id,
            ticket_id: r.ticket_id,
            user_id: r.user_id,
            message: r.message,
            created_at: r.created_at,
        }))
    }
}
```

### File: `src/repositories/dashboard_repository.rs` — BARU

```rust
use sqlx::PgPool;
use serde::Serialize;
use crate::common::AppError;

/// Statistik dashboard untuk aplikasi
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_tickets: i64,
    pub open_tickets: i64,
    pub in_progress_tickets: i64,
    pub resolved_tickets: i64,
    pub closed_tickets: i64,
    pub total_users: i64,
    pub total_agents: i64,
    pub total_customers: i64,
    pub avg_responses_per_ticket: f64,
}

/// Repository untuk mengambil statistik dashboard
#[derive(Clone)]
pub struct DashboardRepository {
    pool: PgPool,
}

impl DashboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Ambil statistik lengkap untuk dashboard
    pub async fn get_stats(&self) -> Result<DashboardStats, AppError> {
        // --- Statistik Tiket ---
        #[derive(sqlx::FromRow)]
        struct TicketStatsRow {
            total_tickets: i64,
            open_tickets: i64,
            in_progress_tickets: i64,
            resolved_tickets: i64,
            closed_tickets: i64,
        }

        let ticket_stats = sqlx::query_as::<_, TicketStatsRow>(
            r#"SELECT
                COUNT(*) AS total_tickets,
                COUNT(*) FILTER (WHERE status = 'open') AS open_tickets,
                COUNT(*) FILTER (WHERE status = 'in_progress') AS in_progress_tickets,
                COUNT(*) FILTER (WHERE status = 'resolved') AS resolved_tickets,
                COUNT(*) FILTER (WHERE status = 'closed') AS closed_tickets
            FROM tickets"#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // --- Statistik User ---
        #[derive(sqlx::FromRow)]
        struct UserStatsRow {
            total_users: i64,
            total_agents: i64,
            total_customers: i64,
        }

        let user_stats = sqlx::query_as::<_, UserStatsRow>(
            r#"SELECT
                COUNT(*) AS total_users,
                COUNT(*) FILTER (WHERE role = 'agent') AS total_agents,
                COUNT(*) FILTER (WHERE role = 'customer') AS total_customers
            FROM users"#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // --- Rata-rata Responses Per Tiket ---
        let total_responses: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ticket_responses"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let avg_responses_per_ticket = if ticket_stats.total_tickets > 0 {
            total_responses as f64 / ticket_stats.total_tickets as f64
        } else {
            0.0
        };

        Ok(DashboardStats {
            total_tickets: ticket_stats.total_tickets,
            open_tickets: ticket_stats.open_tickets,
            in_progress_tickets: ticket_stats.in_progress_tickets,
            resolved_tickets: ticket_stats.resolved_tickets,
            closed_tickets: ticket_stats.closed_tickets,
            total_users: user_stats.total_users,
            total_agents: user_stats.total_agents,
            total_customers: user_stats.total_customers,
            avg_responses_per_ticket,
        })
    }
}
```

### File: `src/repositories/mod.rs` — UPDATE

```rust
pub mod user_repository;
pub mod ticket_repository;
pub mod response_repository;
pub mod dashboard_repository;

pub use user_repository::UserRepository;
pub use ticket_repository::TicketRepository;
pub use response_repository::ResponseRepository;
pub use dashboard_repository::{DashboardRepository, DashboardStats};
```

### File: `src/main.rs` — UPDATE AppState (Lines ~18-30)

```rust
use sqlx::PgPool;
use db::create_pool;
use crate::repositories::{
    UserRepository, TicketRepository, ResponseRepository, DashboardRepository,
};

// ============================================
// AppState — berbagi repositories dan pool ke semua handler
// ============================================
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            ticket_repo: TicketRepository::new(pool.clone()),
            response_repo: ResponseRepository::new(pool.clone()),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            db: pool,
        }
    }
}
```

**Verifikasi:**
```bash
cargo build
# Harus 0 errors (warnings OK)
```

---

## Kesimpulan Bab 26

Bab ini mengimplementasikan dua repository khusus:

1. **ResponseRepository** — untuk CRUD ticket_responses dengan 3 methods:
   - `create(ticket_id, user_id, message)` — simpan balasan
   - `find_by_ticket_id(ticket_id)` — ambil semua balasan (urut kronologis)
   - `find_latest_by_ticket_id(ticket_id)` — ambil balasan terbaru

2. **DashboardRepository** — untuk aggregate statistics:
   - `DashboardStats` struct dengan 9 fields statistik
   - `get_stats()` — query aggregate dengan `COUNT(*) FILTER (WHERE ...)`
   - Division by zero protection untuk `avg_responses_per_ticket`

3. **AppState Expansion** — gabung 4 repositories:
   - UserRepository + TicketRepository (dari Bab 25)
   - ResponseRepository + DashboardRepository (baru)
   - Constructor `AppState::new(pool)` siap untuk di-inject ke handlers

**Key Patterns:**
- **Separation of Concerns:** Setiap repository fokus pada domain data-nya
- **Aggregate Query Pattern:** Dashboard menggunakan `COUNT(*) FILTER` untuk efficiency
- **Type Safety:** Temporary structs (`ResponseRow`, `TicketStatsRow`, dll) untuk mapping database → Rust
- **Dependency Injection Ready:** AppState siap untuk passed ke handler via Axum state extraction

**Status Build:** ✅ 0 errors, 17 warnings (expected — unused code OK)

**Bab Berikutnya:** Bab 27 **Authentication (Register & Login)** — menggunakan repositories untuk user management dan JWT token generation
