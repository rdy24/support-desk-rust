# Bab 26: Repository: Response dan Dashboard

Bayangkan kamu kerja di sebuah call center. Ada dua buku catatan berbeda di meja resepsionis:

1. **Buku tamu percakapan:** setiap kali ada balasan untuk tiket tertentu, dicatat di sini. Siapa yang balas, isinya apa, kapan.
2. **Papan statistik kantor:** di dinding, ada papan besar yang menampilkan ringkasan: berapa tiket yang masuk hari ini, berapa yang sudah selesai, berapa agent yang aktif.

Kedua buku ini punya sifat yang sangat berbeda. Buku tamu = insert dan lookup per tiket. Papan statistik = hitung-hitungan besar dari seluruh data.

Dua "buku catatan" tadi tercermin dalam dua repository yang dibahas di bab ini: `ResponseRepository` dan `DashboardRepository`.

[ILUSTRASI: Dua meja kerja, meja kiri ada "Buku Tamu Percakapan" (ResponseRepository) untuk insert/lookup per tiket, meja kanan ada "Papan Statistik" (DashboardRepository) dengan angka-angka aggregate di papan tulis]

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 25:**
- Latihan #1-4 sudah tercakup di "Hasil Akhir Bab 25" (UserRepository & TicketRepository lengkap dengan delete method & category filter)
- **⚠️ PENTING:** Pastikan enum types sudah dibuat di Bab 25:
  - `src/models/enums.rs` dengan `UserRole`, `TicketStatus`, `TicketPriority`, `TicketCategory` (masing-masing dengan `#[derive(sqlx::Type)]`)
  - Model `User` dan `Ticket` sudah di-update untuk menggunakan enum types, bukan `String`

**State Sebelumnya:**
Folder `src/repositories/` harus punya 3 file (user, ticket, response) dari Bab 25. AppState sudah include ketiga repositories.

Verifikasi dengan menjalankan:
```bash
cargo build
# Harus compile tanpa error tentang "no built in mapping found"
```

---

## ResponseRepository

Tambahkan file baru di `src/repositories/response_repository.rs`.

### Struct dan Constructor

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::{errors::AppError, models::ticket_response::TicketResponse};

pub struct ResponseRepository {
    pool: PgPool,
}

impl ResponseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### `create`

Method ini menyimpan satu balasan tiket ke tabel `ticket_responses`.

```rust
pub async fn create(
    &self,
    ticket_id: Uuid,
    user_id: Uuid,
    message: String,
) -> Result<TicketResponse, AppError> {
    let response = sqlx::query_as!(
        TicketResponse,
        r#"
        INSERT INTO ticket_responses (ticket_id, user_id, message)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        ticket_id,
        user_id,
        message
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(response)
}
```

`RETURNING *` adalah fitur PostgreSQL yang mengembalikan baris yang baru saja di-insert tanpa perlu query tambahan.

### `find_by_ticket_id`

Ambil semua balasan untuk satu tiket tertentu, diurutkan dari yang paling lama.

```rust
pub async fn find_by_ticket_id(
    &self,
    ticket_id: Uuid,
) -> Result<Vec<TicketResponse>, AppError> {
    let responses = sqlx::query_as!(
        TicketResponse,
        r#"
        SELECT * FROM ticket_responses
        WHERE ticket_id = $1
        ORDER BY created_at ASC
        "#,
        ticket_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(responses)
}
```

`.fetch_all()` dipakai karena satu tiket bisa punya banyak balasan. Hasilnya `Vec<TicketResponse>`, bisa kosong kalau belum ada balasan, bisa berisi banyak.

---

## DashboardRepository

Dashboard bukan sekadar "ambil data", dia harus **menghitung dan meringkas** data dari seluruh tabel.

Buat file `src/repositories/dashboard_repository.rs`.

### Struct `DashboardStats`

Pertama, definisikan struct untuk menampung hasil statistik:

```rust
pub struct DashboardStats {
    pub total_tickets: i64,
    pub open_tickets: i64,
    pub in_progress_tickets: i64,
    pub resolved_tickets: i64,
    pub closed_tickets: i64,
    pub total_users: i64,
    pub total_agents: i64,
    pub total_customers: i64,
}
```

`COUNT(*)` di PostgreSQL mengembalikan tipe `BIGINT`, dan `sqlx` memetakannya ke `i64` di Rust. Kalau kamu pakai `i32`, kompilasi akan error karena tipe tidak cocok.

### Struct Repository

```rust
use sqlx::PgPool;
use crate::common::AppError;

pub struct DashboardRepository {
    pool: PgPool,
}

impl DashboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### Query Aggregate dengan `FILTER`

Bagian paling krusial adalah method `get_stats`:

```rust
pub async fn get_stats(&self) -> Result<DashboardStats, AppError> {
    let ticket_stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'open') as open_tickets,
            COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress_tickets,
            COUNT(*) FILTER (WHERE status = 'resolved') as resolved_tickets,
            COUNT(*) FILTER (WHERE status = 'closed') as closed_tickets,
            COUNT(*) as total_tickets
        FROM tickets
        "#
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let user_stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE role = 'agent') as total_agents,
            COUNT(*) FILTER (WHERE role = 'customer') as total_customers
        FROM users
        "#
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(DashboardStats {
        total_tickets: ticket_stats.total_tickets.unwrap_or(0),
        open_tickets: ticket_stats.open_tickets.unwrap_or(0),
        in_progress_tickets: ticket_stats.in_progress_tickets.unwrap_or(0),
        resolved_tickets: ticket_stats.resolved_tickets.unwrap_or(0),
        closed_tickets: ticket_stats.closed_tickets.unwrap_or(0),
        total_users: user_stats.total_users.unwrap_or(0),
        total_agents: user_stats.total_agents.unwrap_or(0),
        total_customers: user_stats.total_customers.unwrap_or(0),
    })
}
```

### Kenapa `FILTER` Lebih Efisien dari Multiple Query?

Coba bayangkan dua pendekatan:

**Pendekatan naif,** 5 query terpisah:
```sql
SELECT COUNT(*) FROM tickets;
SELECT COUNT(*) FROM tickets WHERE status = 'open';
SELECT COUNT(*) FROM tickets WHERE status = 'in_progress';
-- dst...
```

Setiap query berarti satu bolak-balik ke database. Kalau servernya jauh atau lagi sibuk, latency-nya menumpuk.

**Pendekatan `FILTER`,** 1 query, satu scan:
```sql
SELECT
    COUNT(*) as total,
    COUNT(*) FILTER (WHERE status = 'open') as open
FROM tickets;
```

Database cukup **scan tabel satu kali**, sambil jalan menghitung beberapa counter sekaligus. Hasilnya sama, tapi jauh lebih hemat. Teknik ini disebut *conditional aggregation*: aggregate dengan kondisi tertentu, di PostgreSQL pakai `FILTER (WHERE ...)`.

[ILUSTRASI: Perbandingan dua skenario. Kiri: 5 kotak query terpisah masing-masing menuju database (5 panah); kanan: 1 kotak query tunggal dengan satu panah ke database, hasilnya 5 angka sekaligus. Label "5x bolak-balik" vs "1x bolak-balik"]

---

## Raw String `r#"..."#` untuk SQL Multi-baris

Kamu mungkin sudah perhatikan kita pakai `r#"..."#` di semua query. Ini disebut **raw string literal** di Rust.

SQL multi-baris biasanya mengandung tanda kutip untuk nilai string seperti `'open'` atau `'agent'`. Kalau ditulis dalam string Rust biasa:

```rust
// ❌ Ini error — tanda kutip di dalam string harus di-escape
"SELECT * FROM tickets WHERE status = 'open'"
//                                    ^   ^ konflik!
```

Solusinya raw string:

```rust
// ✅ Tanda kutip dalam raw string tidak perlu di-escape
r#"SELECT * FROM tickets WHERE status = 'open'"#
```

Aturan raw string: dimulai dengan `r#"` dan diakhiri `"#`, isi di dalamnya ditulis apa adanya tanpa escape character. Kalau isi string kamu sendiri mengandung `"#`, tambahkan lebih banyak `#`: `r##"..."##`. Untuk SQL panjang multi-baris, kombinasi raw string dan indentasi membuat query mudah dibaca, persis seperti kamu tulis di database client.

---

## Daftarkan ke `mod.rs` dan `AppState`

Tambahkan kedua repository baru ke `src/repositories/mod.rs`:

```rust
pub mod dashboard_repository;
pub mod response_repository;
pub mod ticket_repository;
pub mod user_repository;
```

Lalu perbarui `AppState` di `src/state.rs` (atau di mana kamu menyimpannya):

```rust
use crate::repositories::{
    dashboard_repository::DashboardRepository,
    response_repository::ResponseRepository,
    ticket_repository::TicketRepository,
    user_repository::UserRepository,
};

#[derive(Clone)]
pub struct AppState {
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
            dashboard_repo: DashboardRepository::new(pool),
        }
    }
}
```

Perhatikan `pool.clone()`, kita clone pool karena `PgPool` di `sqlx` adalah connection pool yang aman di-clone (saat ebook ini ditulis, Maret 2026, sqlx versi 0.8.x). Clone-nya murah karena yang di-clone hanya reference ke pool, bukan koneksi aktual.

---

## Latihan

1. **Tambah method `find_latest_by_ticket_id`** di `ResponseRepository`: ambil hanya balasan terbaru (1 baris) untuk tiket tertentu. Hint: gunakan `ORDER BY created_at DESC` dan `.fetch_optional()`.

2. **Modifikasi `DashboardStats`:** tambahkan field `avg_responses_per_ticket: f64` yang menghitung rata-rata jumlah balasan per tiket. Kamu perlu query ke tabel `ticket_responses` dan bagi dengan total tiket. Pertimbangkan: bagaimana handle kasus `total_tickets = 0` agar tidak terjadi division by zero?

3. **Optimasi query dashboard:** coba gabungkan `ticket_stats` dan `user_stats` menjadi satu query dengan `CROSS JOIN` atau subquery. Bandingkan keterbacaannya dengan versi dua query terpisah. Mana yang lebih kamu sukai dan kenapa?

4. **Error handling:** saat ini kalau database sedang down, kita return `AppError::Internal`. Tambahkan log pesan error sebelum return, menggunakan macro `tracing::error!("Database error: {:?}", e)`. Pastikan kamu sudah tambahkan `tracing` sebagai dependency.
