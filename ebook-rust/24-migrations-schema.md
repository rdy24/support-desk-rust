# Bab 24: Migrations dan Schema

Renovasi gedung kantor yang baik tidak asal tebas tembok. Ada blueprint yang tercatat rapi: "Minggu 1: pasang fondasi baru. Minggu 2: tambah ruang meeting. Minggu 3: pindah toilet." Setiap perubahan terdokumentasi, berurutan, dan bisa di-*trace* balik kalau ada yang salah.

Itulah yang namanya **migration** di dunia database.

[ILUSTRASI: Tumpukan folder berlabel "001_fondasi", "002_ruang_meeting", "003_toilet", tiap folder berisi blueprint perubahan gedung yang tersusun rapi dan berurutan]

---

## Kunci Jawaban Latihan Bab 23

### Latihan #1-3: Docker & .env setup

Sudah dikerjakan secara praktik (bukan kode). Verifikasi:
```bash
docker ps  # Container harus Up
cat .env   # File harus ada dengan DATABASE_URL
```

### Latihan #4: max_connections exploration

Dalam pengalaman, tuning `max_connections`:
- Development: 5-10 cukup
- Staging: 20-30
- Production: tergantung RPS (requests per second) server

Kalau sering dapat `"connection timeout"` error, naikkan nilai. Kalau connection idle banyak, turunkan.

### Latihan #5: `.env.example`

File sudah dibuat di "Hasil Akhir Bab 23" (lihat sebelumnya).

### Latihan #6: Docker Compose

File `docker-compose.yml` sudah dibuat. Jalankan dengan:
```bash
docker compose up -d
docker ps  # Verifikasi container Up
```

---

## State Sebelumnya

Sebelum mulai Bab 24, pastikan dari Bab 23 sudah ada:

```
support-desk/
├── .env                        ← Dengan DATABASE_URL yang benar
├── .env.example
├── docker-compose.yml
├── Cargo.toml                  ← sqlx dependency sudah ada
├── src/
│   ├── main.rs                 ← AppState sudah ada
│   ├── db.rs                   ← Fungsi create_pool
│   ├── models/
│   ├── dto/
│   └── common/
```

Dan verifikasi:
```bash
cargo build      # Harus compile
docker compose up -d
cargo run        # Server harus connect ke database tanpa error
```

---

## Apa Itu Migration?

Migration adalah file SQL yang berisi instruksi perubahan struktur database, disimpan secara berurutan. Bukan cuma "jalankan SQL ini sekarang", tapi "catat bahwa SQL ini sudah dijalankan, dan kalau perlu, bisa di-*undo*."

Ini penting saat kerja tim, saat deploy ke server baru, atau saat ada bug dan perlu balik ke versi lama. Tanpa migration, tiap developer punya versi database yang beda-beda dan chaos dimulai dari sana.

Di project TypeScript kita pakai **Drizzle ORM** yang handle ini. Di Rust, **sqlx** punya tool CLI khusus untuk migration.

---

## Install sqlx-cli

**sqlx-cli** adalah command-line tool dari sqlx untuk manage migration. Install sekali, pakai selamanya:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Flag `--no-default-features --features postgres` menginstall versi yang hanya support PostgreSQL, lebih ringan karena tidak membawa driver database lain yang tidak dibutuhkan.

Verifikasi instalasi berhasil:

```bash
sqlx --version
```

> Catatan: sqlx-cli versi 0.8.x (saat ebook ini ditulis, Maret 2026) sudah stabil dan banyak dipakai production.

---

## Buat Migration Files

Buat file migration-nya dengan perintah ini. **Penting: jalankan SATU PER SATU dengan jeda, gunakan flag `-r` untuk reversible migrations**:

```bash
# Langkah 1: Create users table (migration pertama)
sqlx migrate add -r create_users_table
sleep 1

# Langkah 2: Create tickets table (migration kedua - perlu jeda 1 detik)
sqlx migrate add -r create_tickets_table
sleep 1

# Langkah 3: Create ticket responses table (migration ketiga - perlu jeda 1 detik)
sqlx migrate add -r create_ticket_responses_table
sleep 1

# Langkah 4: Add indexes (migration keempat - perlu jeda 1 detik)
sqlx migrate add -r add_indexes
```

**Mengapa harus satu per satu?** Karena SQLx menggunakan **timestamp yang presisi hingga detik** untuk menentukan urutan migration. Jika semua command dijalankan bersamaan dalam 1 detik yang sama, semua migration akan dapat timestamp yang identik dan urutan tidak terjamin. Ini penting karena:
- `create_users_table` harus jalan **PERTAMA** (users table harus exist)
- `create_tickets_table` harus jalan **KEDUA** (butuh reference ke users table)
- `create_ticket_responses_table` harus jalan **KETIGA** (butuh reference ke tickets table)

**Flag `-r` (reversible) sangat penting!** Tanpa `-r`, `sqlx migrate add` hanya membuat 1 file `.sql` biasa, bukan `.up.sql` dan `.down.sql`.

Setiap perintah akan generate dua file di folder `migrations/`:
- `{timestamp}_{nama}.up.sql`: SQL untuk *apply* perubahan
- `{timestamp}_{nama}.down.sql`: SQL untuk *undo* perubahan (rollback)

Hasilnya folder `migrations/` akan terlihat seperti ini (dengan timestamp berbeda **1 detik**):

```
migrations/
├── 20260403021603_create_users_table.up.sql          ← timestamp: 021603
├── 20260403021603_create_users_table.down.sql
├── 20260403021604_create_tickets_table.up.sql        ← timestamp: 021604 (1 detik kemudian)
├── 20260403021604_create_tickets_table.down.sql
├── 20260403021605_create_ticket_responses_table.up.sql  ← timestamp: 021605 (1 detik kemudian)
├── 20260403021605_create_ticket_responses_table.down.sql
├── 20260403021606_add_indexes.up.sql                 ← timestamp: 021606 (1 detik kemudian)
└── 20260403021606_add_indexes.down.sql
```

**Perhatikan timestamp!** Setiap detik harus berbeda (021603, 021604, 021605, 021606). Ini memastikan SQLx menjalankan migrations dalam urutan yang tepat.

### ⚠️ Penting: Flag `-r` dan Jeda Waktu

**Kesalahan umum #1: Tidak menggunakan flag `-r`**
Kalau kamu jalankan `sqlx migrate add <nama>` **tanpa** flag `-r`, hanya akan terbuat **1 file**:
```
migrations/20260403021603_nama.sql  ← HANYA 1 FILE
```

**Kesalahan umum #2: Menjalankan semua command sekaligus**
Jika kamu copy-paste ketiga command sekaligus tanpa jeda:
```bash
sqlx migrate add -r create_users_table
sqlx migrate add -r create_tickets_table
sqlx migrate add -r create_ticket_responses_table
# ❌ SALAH! Semua dapat timestamp yang sama seperti: 20260403021603_*
```

Hasilnya:
```
migrations/
├── 20260403021603_create_users_table.up.sql
├── 20260403021603_create_tickets_table.up.sql      ← Timestamp SAMA!
├── 20260403021603_create_ticket_responses_table.up.sql  ← Timestamp SAMA!
```

Ini akan menyebabkan error saat menjalankan migration karena urutan tidak terjamin.

**Solusi: Jalankan satu per satu dengan `sleep 1`:**
```bash
sqlx migrate add -r create_users_table && sleep 1
sqlx migrate add -r create_tickets_table && sleep 1
sqlx migrate add -r create_ticket_responses_table && sleep 1
sqlx migrate add -r add_indexes
```

Setelah migration pertama pakai `-r`, migration berikutnya otomatis juga reversible.

---

## Schema Users

Isi file `..._create_users_table.up.sql`:

```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM ('admin', 'agent', 'customer');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'customer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Beberapa hal yang perlu diperhatikan:

- `CREATE EXTENSION IF NOT EXISTS "uuid-ossp"`: mengaktifkan extension PostgreSQL untuk generate UUID. Ini **harus ada di migration pertama** karena migration berikutnya bergantung pada UUID.
- `CREATE TYPE user_role AS ENUM (...)`: PostgreSQL mendukung tipe data custom. Ini lebih aman dari `VARCHAR` karena database langsung reject nilai di luar pilihan yang valid.
- `TIMESTAMPTZ`: *timestamp with timezone*. Lebih baik dari `TIMESTAMP` biasa karena PostgreSQL menyimpan dalam UTC dan konversi timezone-nya otomatis.

Untuk rollback (`..._create_users_table.down.sql`):

```sql
DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS user_role;
```

---

## Schema Tickets

Isi file `..._create_tickets_table.up.sql`:

```sql
CREATE TYPE ticket_category AS ENUM ('general', 'billing', 'technical', 'other');
CREATE TYPE ticket_priority AS ENUM ('low', 'medium', 'high', 'urgent');
CREATE TYPE ticket_status AS ENUM ('open', 'in_progress', 'resolved', 'closed');

CREATE TABLE tickets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id UUID NOT NULL REFERENCES users(id),
    agent_id UUID REFERENCES users(id),
    category ticket_category NOT NULL DEFAULT 'general',
    priority ticket_priority NOT NULL DEFAULT 'medium',
    status ticket_status NOT NULL DEFAULT 'open',
    subject VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    tags TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Perhatikan beberapa hal:

- `customer_id UUID NOT NULL REFERENCES users(id)`: ini **foreign key**. Setiap ticket *wajib* punya customer, dan customer itu harus ada di tabel `users`. Database yang jaga integritas ini, bukan aplikasi.
- `agent_id UUID REFERENCES users(id)`: tanpa `NOT NULL`, artinya agent boleh kosong (`NULL`). Ticket baru memang belum tentu langsung punya agent.
- `tags TEXT[] DEFAULT '{}'`: PostgreSQL mendukung array. `TEXT[]` adalah array of string. Default-nya array kosong `{}`.

Untuk rollback:

```sql
DROP TABLE IF EXISTS tickets;
DROP TYPE IF EXISTS ticket_status;
DROP TYPE IF EXISTS ticket_priority;
DROP TYPE IF EXISTS ticket_category;
```

---

## Schema Ticket Responses

Isi file `..._create_ticket_responses_table.up.sql`:

```sql
CREATE TABLE ticket_responses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Yang menarik di sini: `ON DELETE CASCADE` pada `ticket_id`. Kalau sebuah ticket dihapus, semua response-nya ikut terhapus otomatis. Ini logis karena response tanpa ticket-nya tidak ada gunanya.

Untuk rollback:

```sql
DROP TABLE IF EXISTS ticket_responses;
```

[ILUSTRASI: Diagram tiga kotak (users, tickets, ticket_responses) dengan panah yang menunjukkan relasi foreign key: users ke tickets (customer_id dan agent_id), tickets ke ticket_responses (ticket_id), dan users ke ticket_responses (user_id)]

---

## Jalankan Migration

Pastikan environment variable `DATABASE_URL` sudah diset di `.env`:

```
DATABASE_URL=postgres://username:password@localhost:5432/support_desk
```

Lalu jalankan:

```bash
sqlx migrate run
```

Output-nya akan menampilkan tiap migration yang dijalankan secara berurutan. sqlx menyimpan catatan migration yang sudah dijalankan di tabel `_sqlx_migrations`, jadi kalau dijalankan lagi, migration yang sudah selesai akan dilewati otomatis.

---

## Rollback Migration

Kalau ada yang salah dan mau undo migration terakhir:

```bash
sqlx migrate revert
```

Perintah ini menjalankan file `.down.sql` dari migration paling akhir. Satu revert = satu migration mundur.

Itulah kenapa penting untuk selalu menulis file `.down.sql` dengan benar. Kalau file down-nya kosong atau salah, rollback jadi percuma.

---

## Auto-migrate di main.rs

Daripada harus ingat jalankan `sqlx migrate run` setiap deploy, tambahkan migration otomatis saat aplikasi startup:

```rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

Tambahkan baris ini di `main.rs` setelah koneksi database berhasil, sebelum server dijalankan:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    // ... setup router dan jalankan server
}
```

Macro `sqlx::migrate!("./migrations")` bekerja di *compile time*: sqlx membaca folder migrations saat build, bukan saat runtime. Kalau folder migrations tidak ada atau path-nya salah, program langsung gagal compile. Aman dan cepat.

---

## Latihan

Setelah memahami konsep migration, coba kerjakan ini:

1. **Buat database baru** di PostgreSQL lokal dengan nama `support_desk_rust`, lalu set `DATABASE_URL` di `.env`.

2. **Jalankan tiga migration** yang sudah dibuat dengan `sqlx migrate run`. Verifikasi tabelnya terbuat dengan koneksi ke database (pakai `psql` atau GUI seperti TablePlus).

3. **Coba rollback** satu migration dengan `sqlx migrate revert`, lalu jalankan ulang dengan `sqlx migrate run`. Pastikan prosesnya berjalan mulus.

4. **Tambahkan index**: buat migration baru bernama `add_indexes`:
   ```bash
   sqlx migrate add -r add_indexes
   ```
   (Jangan lupa flag `-r` untuk reversible!)
   Isi file `.up.sql`-nya dengan index untuk kolom yang sering di-query:
   ```sql
   CREATE INDEX idx_tickets_customer_id ON tickets(customer_id);
   CREATE INDEX idx_tickets_status ON tickets(status);
   CREATE INDEX idx_ticket_responses_ticket_id ON ticket_responses(ticket_id);
   ```
   Jalankan dan verifikasi index-nya terbuat.

5. **Tambahkan auto-migrate** ke `main.rs` dan pastikan aplikasi bisa start tanpa error.

> Tips: Kalau dapat error `error: DATABASE_URL must be set`, pastikan file `.env` ada di root project dan sudah diisi dengan benar.

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan latihan Bab 24, folder struktur harus seperti ini:

```
support-desk/
├── migrations/                 ← NEW FOLDER
│   ├── 20260301000001_create_users_table.up.sql
│   ├── 20260301000001_create_users_table.down.sql
│   ├── 20260301000002_create_tickets_table.up.sql
│   ├── 20260301000002_create_tickets_table.down.sql
│   ├── 20260301000003_create_ticket_responses_table.up.sql
│   ├── 20260301000003_create_ticket_responses_table.down.sql
│   ├── 20260301000004_add_indexes.up.sql           ← dari latihan #4
│   └── 20260301000004_add_indexes.down.sql
├── src/
│   ├── main.rs                 ← Update: tambah sqlx::migrate!
│   ├── db.rs
│   ├── models/
│   ├── dto/
│   └── common/
└── .env, docker-compose.yml, .env.example, .gitignore
```

**Migration Files** — Isi yang tepat untuk masing-masing:

**File: `migrations/.../create_users_table.up.sql`**
```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM ('admin', 'agent', 'customer');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'customer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**File: `migrations/.../create_users_table.down.sql`**
```sql
DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS user_role;
```

**File: `migrations/.../create_tickets_table.up.sql`**
```sql
CREATE TYPE ticket_status AS ENUM ('open', 'in_progress', 'resolved', 'closed');
CREATE TYPE ticket_priority AS ENUM ('low', 'medium', 'high', 'urgent');
CREATE TYPE ticket_category AS ENUM ('general', 'billing', 'technical', 'other');

CREATE TABLE tickets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id UUID REFERENCES users(id) ON DELETE SET NULL,
    category ticket_category NOT NULL,
    priority ticket_priority NOT NULL,
    status ticket_status NOT NULL DEFAULT 'open',
    subject VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**File: `migrations/.../create_tickets_table.down.sql`**
```sql
DROP TABLE IF EXISTS tickets;
DROP TYPE IF EXISTS ticket_status;
DROP TYPE IF EXISTS ticket_priority;
DROP TYPE IF EXISTS ticket_category;
```

**File: `migrations/.../create_ticket_responses_table.up.sql`**
```sql
CREATE TABLE ticket_responses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**File: `migrations/.../create_ticket_responses_table.down.sql`**
```sql
DROP TABLE IF EXISTS ticket_responses;
```

**File: `migrations/.../add_indexes.up.sql`** (dari latihan #4)
```sql
CREATE INDEX idx_tickets_customer_id ON tickets(customer_id);
CREATE INDEX idx_tickets_status ON tickets(status);
CREATE INDEX idx_ticket_responses_ticket_id ON ticket_responses(ticket_id);
CREATE INDEX idx_users_email ON users(email);
```

**File: `migrations/.../add_indexes.down.sql`**
```sql
DROP INDEX IF EXISTS idx_tickets_customer_id;
DROP INDEX IF EXISTS idx_tickets_status;
DROP INDEX IF EXISTS idx_ticket_responses_ticket_id;
DROP INDEX IF EXISTS idx_users_email;
```

**Update File: `src/main.rs`** — Tambahkan auto-migration di main():

**Lokasi:** Update fungsi `main()` di `src/main.rs` (tambahkan kode auto-migration setelah koneksi database berhasil)

**Kode yang ditambahkan:**

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
```

Tambahkan kode di atas **SETELAH** test query database dan **SEBELUM** setup router. Urutan lengkapnya di `main()`:

```rust
#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Baca DATABASE_URL dari environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL harus di-set di .env");

    // Buat connection pool ke database
    let pool = create_pool(&database_url).await;

    // Verifikasi koneksi berhasil dengan test query
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => println!("✓ Database connected successfully"),
        Err(e) => eprintln!("✗ Database connection failed: {}", e),
    }

    // Jalankan migrations otomatis ← TAMBAH INI
    match sqlx::migrate!("./migrations")
        .run(&pool)
        .await {
        Ok(_) => println!("✓ Migrations executed successfully"),
        Err(e) => {
            eprintln!("✗ Migrations failed: {}", e);
            return;
        }
    }

    // Setup router dengan semua routes
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/tickets", ticket_routes())
        .nest("/users", user_routes());

    // Baca PORT dari environment, default 3000 jika tidak ada
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server berjalan di http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
```

**Penjelasan:**
- `sqlx::migrate!("./migrations")` membaca semua migration files dari folder `./migrations`
- `.run(&pool)` menjalankan semua migration yang belum pernah dijalankan sebelumnya
- SQLx otomatis track yang sudah dijalankan di tabel `_sqlx_migrations`, jadi safe untuk di-run berulang kali
- Error handling dengan `match` memastikan aplikasi berhenti dengan pesan yang jelas kalau migration gagal, daripada silent error

**Verifikasi:**
```bash
# Run migrations:
cargo run
# Output: "Server berjalan di..." → Migrations berhasil dijalankan

# Verifikasi tabel di database:
docker exec -it support-desk-db psql -U postgres -d support_desk -c "\dt"
# Harus muncul: users, tickets, ticket_responses, _sqlx_migrations

# Verifikasi index:
docker exec -it support-desk-db psql -U postgres -d support_desk -c "\di"
# Harus muncul: idx_tickets_customer_id, idx_tickets_status, dll

# Atau masuk psql interaktif:
docker exec -it support-desk-db psql -U postgres -d support_desk
# Di psql:
\dt     # lihat tables
\di     # lihat indexes
\q      # keluar
```

Database schema sekarang ready untuk repository implementation di Bab 25.
