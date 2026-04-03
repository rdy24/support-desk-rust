# Bab 20: Serde — Serialisasi JSON

Di bab sebelumnya kita sudah punya routing yang jalan dan bisa kirim JSON. Tapi JSON-nya masih asal-asalan, pakai `Json<Value>` yang bisa isi apa saja, tanpa struktur jelas. Di bab ini kita rapikan dengan mendefinisikan struct yang proper pakai **Serde**.

---

## Kunci Jawaban Latihan Bab 19

### Latihan #1: Tambah route DELETE

```rust
async fn delete_ticket(Path(id): Path<u32>) -> StatusCode {
    println!("Menghapus ticket {}", id);
    StatusCode::NO_CONTENT
}

fn ticket_routes() -> Router {
    Router::new()
        .route("/", get(get_tickets).post(create_ticket))
        .route("/{id}", get(get_ticket).delete(delete_ticket))  // ← tambah .delete()
}
```

Status code `204 NO_CONTENT` adalah standar HTTP untuk operasi DELETE yang berhasil tanpa mengembalikan data.

### Latihan #2: Tambah field `priority` ke TicketFilters

```rust
#[derive(Deserialize)]
struct TicketFilters {
    page: Option<u32>,
    limit: Option<u32>,
    status: Option<String>,
    priority: Option<String>,  // ← TAMBAH INI
}

async fn get_tickets(Query(filters): Query<TicketFilters>) -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "page": filters.page.unwrap_or(1),
        "limit": filters.limit.unwrap_or(10),
        "priority": filters.priority  // ← TAMBAH INI
    }))
}
```

Test dengan: `curl "http://localhost:3000/tickets?priority=high"`

### Latihan #3: Buat `user_routes()`

```rust
async fn get_users() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": []
    }))
}

async fn get_user(Path(id): Path<u32>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "id": id, "name": "Contoh user" }
    })))
}

fn user_routes() -> Router {
    Router::new()
        .route("/", get(get_users))
        .route("/{id}", get(get_user))
}

// Di main(), tambah nest:
let app = Router::new()
    .route("/health", get(health_check))
    .nest("/tickets", ticket_routes())
    .nest("/users", user_routes());  // ← TAMBAH INI
```

Test dengan: `curl http://localhost:3000/users` dan `curl http://localhost:3000/users/1`

---

## State Sebelumnya

Pastikan sudah selesai Bab 19. Jika sudah, `src/main.rs` kamu seharusnya terlihat seperti ini (hasil dari semua latihan Bab 19):

```rust
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct TicketFilters {
    page: Option<u32>,
    limit: Option<u32>,
    status: Option<String>,
    priority: Option<String>,  // dari latihan #2
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_tickets(Query(filters): Query<TicketFilters>) -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "page": filters.page.unwrap_or(1),
        "limit": filters.limit.unwrap_or(10),
        "priority": filters.priority
    }))
}

async fn get_ticket(Path(id): Path<u32>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "id": id, "title": "Contoh ticket" }
    })))
}

async fn create_ticket(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    println!("Body diterima: {:?}", body);
    (StatusCode::CREATED, Json(json!({
        "success": true,
        "message": "Ticket berhasil dibuat"
    })))
}

async fn delete_ticket(Path(id): Path<u32>) -> StatusCode {
    println!("Menghapus ticket {}", id);
    StatusCode::NO_CONTENT
}

fn ticket_routes() -> Router {
    Router::new()
        .route("/", get(get_tickets).post(create_ticket))
        .route("/{id}", get(get_ticket).delete(delete_ticket))
}

async fn get_users() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": []
    }))
}

async fn get_user(Path(id): Path<u32>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "id": id, "name": "Contoh user" }
    })))
}

fn user_routes() -> Router {
    Router::new()
        .route("/", get(get_users))
        .route("/{id}", get(get_user))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/tickets", ticket_routes())
        .nest("/users", user_routes());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server berjalan di http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

Jika `src/main.rs` kamu belum seperti di atas, selesaikan latihan Bab 19 terlebih dahulu, terutama:
- Latihan #1: DELETE handler
- Latihan #2: tambah `priority` field
- Latihan #3: buat `user_routes()` dan nest di main router

---

## Penerjemah Bahasa Antara Rust dan Dunia Luar

Bayangkan kamu punya teman yang cuma bisa bahasa Jawa, dan kamu mau ngobrol sama dia tapi kamu cuma bisa bahasa Indonesia. Kamu butuh **penerjemah** di tengah.

Situasi serupa terjadi di aplikasi web. Rust berbicara dalam struct, enum, tipe data statis. Dunia luar (HTTP, database, file) berbicara dalam teks: JSON, XML, CSV. Kalau kita kirim data ke client, Rust perlu *menerjemahkan* struct jadi JSON. Kalau client kirim data ke kita, Rust perlu *menerjemahkan* JSON jadi struct. Proses ini disebut:

- **Serialisasi:** Rust → JSON (kirim ke luar)
- **Deserialisasi:** JSON → Rust (terima dari luar)

**Serde** adalah penerjemah universal untuk Rust.

[ILUSTRASI: Diagram dua arah, struct Rust di kiri, JSON di kanan, Serde sebagai jembatan di tengah dengan label "serialize" dan "deserialize"]

---

## Apa Itu Serde?

**Serde** kependekan dari **Ser**ialize dan **De**serialize. Ini adalah library paling populer di ekosistem Rust untuk konversi data, hampir semua library Rust yang butuh JSON, YAML, TOML, atau format lain pakai Serde di baliknya.

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

> Versi di atas adalah yang stabil saat ebook ini ditulis (Maret 2026). Selalu cek [crates.io](https://crates.io) untuk versi terbaru.

Perhatikan `features = ["derive"]` di serde, ini penting. Tanpa feature `derive`, kita harus tulis kode konversi manual untuk setiap struct. Dengan `derive`, cukup tambahkan satu baris.

---

## Derive Serialize dan Deserialize

Di Rust, `#[derive(...)]` adalah cara otomatis agar Rust men-*generate* implementasi trait untuk kita. Bayangkan seperti ini: kamu minta Rust, "tolong buatkan kode konversi JSON untuk struct ini secara otomatis."

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}
```

Dengan `#[derive(Serialize, Deserialize)]`, struct `User` ini bisa diubah jadi JSON string dengan `serde_json::to_string(&user)`, dan dibuat dari JSON string dengan `serde_json::from_str::<User>(json_str)`. Tanpa Serde, kamu harus tulis puluhan baris kode manual untuk hal yang sama.

---

## Attribute Serde yang Berguna

Serde punya banyak "attribute", yaitu instruksi tambahan yang kita tempel di struct atau field untuk mengatur perilaku konversi. Ini seperti sticky note yang kamu tempel di dokumen sebelum diserahkan ke penerjemah.

### `#[serde(rename_all = "camelCase")]`

Rust punya konvensi penamaan `snake_case` (huruf kecil, dipisah underscore). Tapi JSON dari JavaScript/TypeScript biasanya `camelCase`. Attribute ini otomatis mengkonversi semua nama field saat serialisasi/deserialisasi.

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub customer_id: u32,  // → JSON: "customerId"
    pub created_at: String, // → JSON: "createdAt"
}
```

### `#[serde(skip_serializing_if = "Option::is_none")]`

Field bertipe `Option<T>` bisa bernilai `None`. Defaultnya, Serde akan tetap menyertakan field tersebut di JSON sebagai `null`. Kalau kita mau skip field yang `None`, pakai attribute ini.

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub agent_id: Option<u32>,  // kalau None, tidak muncul di JSON sama sekali
```

### `#[serde(skip_serializing)]`

Kadang ada field yang kita butuhkan di dalam Rust tapi **tidak boleh** dikirim ke client. Contoh klasik: password. Kita perlu simpan di struct untuk proses internal, tapi jangan sampai bocor ke response API.

```rust
#[serde(skip_serializing)]
pub password: String,  // tidak akan muncul di JSON response
```

### `#[serde(rename = "nama_baru")]`

Untuk rename field spesifik (bukan semua field):

```rust
pub created_at: String,
#[serde(rename = "ts")]
pub timestamp: u64,  // di JSON muncul sebagai "ts"
```

---

## Struktur Folder Models

Sekarang kita definisikan model-model utama untuk Support Desk. Pertama, buat folder baru `src/models/`. Folder ini akan berisi semua struct untuk User, Ticket, DTO, dan Response.

**Struktur yang akan kita buat:**

```
src/
├── main.rs
└── models/
    ├── mod.rs          ← file ini daftarkan semua module
    ├── user.rs         ← struct User
    ├── ticket.rs       ← struct Ticket + CreateTicketDto
    └── api_response.rs ← struct ApiResponse<T>
```

---

## Model User dan Ticket

### File 1: `src/models/user.rs`

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

Beberapa hal penting di sini. `Uuid` menghasilkan ID unik yang tidak bisa ditebak, lebih aman dari integer auto-increment. `DateTime<Utc>` adalah timestamp dengan timezone UTC dari library `chrono`. `#[serde(skip_serializing)]` di `password` bersifat krusial untuk keamanan karena password hash tidak boleh pernah dikirim ke client. `Clone` dipasang agar struct bisa di-copy, berguna saat kita perlu pass ke beberapa tempat.

### File 2: `src/models/ticket.rs`

File ini berisi **dua** struct: `Ticket` (model utama) dan `CreateTicketDto` (untuk input dari client).

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================
// STRUCT: Ticket (model utama, untuk database)
// ============================================

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

// ============================================
// DTO: CreateTicketDto (untuk input dari client)
// ============================================
// DTO hanya berisi field yang BOLEH dikirim client.
// Field seperti id, createdAt, status tidak boleh diisi client.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketDto {
    pub subject: String,
    pub description: String,
    pub category: String,
    pub priority: String,
}
```

**Penjelasan:**
- `agent_id` bertipe `Option<Uuid>` karena tiket baru belum tentu punya agen. Kalau belum assign, nilainya `None` dan tidak akan muncul di JSON response.
- `CreateTicketDto` hanya punya `Deserialize` (terima dari client), bukan `Serialize` (kirim ke client).

---

---

## Contoh Pemakaian ApiResponse

Response API yang konsisten membuat frontend lebih mudah dan debugging lebih nyaman. Contoh pemakaian di handler nantinya akan terlihat seperti ini:

```rust
// Response sukses dengan data
ApiResponse::ok(user, "User ditemukan")

// Response error tanpa data
ApiResponse::error("User tidak ditemukan")
```

JSON yang dihasilkan:

```json
// Sukses
{ "success": true, "message": "User ditemukan", "data": { "id": "...", "name": "..." } }

// Error — field "data" tidak muncul sama sekali
{ "success": false, "message": "User tidak ditemukan" }
```

---

### File 3: `src/models/api_response.rs`

File ini berisi wrapper response generik `ApiResponse<T>`:

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

`<T: Serialize>` berarti "T bisa tipe apa saja, asal bisa di-serialize". Ini adalah generics yang kita pelajari di Bab 14, dipakai di dunia nyata.

---

## Module Declaration: `src/models/mod.rs`

Sekarang gabungkan semua model dalam satu module dengan membuat `src/models/mod.rs`:

```rust
pub mod api_response;
pub mod ticket;
pub mod user;

pub use api_response::ApiResponse;
pub use ticket::Ticket;
pub use user::User;
```

**Catatan penting:**
- `CreateTicketDto` **tidak** diexport dari models, karena akan dipindah ke folder `src/dto/` di Bab 21
- Hanya `Ticket` (model utama) yang diexport dari sini

Daftarkan di `src/main.rs`:

```rust
mod models;
```

Setelah itu, di mana saja dalam project, kita bisa tulis:

```rust
use crate::models::{ApiResponse, Ticket, User};
```

---

## Latihan

Coba kerjakan latihan berikut untuk memastikan kamu paham:

1. **Buat struct `TicketResponse`:** model untuk respons tiket (bukan tiket itu sendiri). Field: `id: Uuid`, `ticket_id: Uuid`, `user_id: Uuid`, `message: String`, `created_at: DateTime<Utc>`. Tambahkan derive yang sesuai dan `rename_all = "camelCase"`.

2. **Buat `CreateTicketResponseDto`:** DTO untuk menerima input respons tiket dari agent. Hanya boleh ada field `message: String`.

3. **Uji serialisasi manual:** di `main.rs` atau file test, buat instance `User` dummy lalu print dengan `serde_json::to_string_pretty(&user).unwrap()`. Verifikasi bahwa field `password` tidak muncul di output.

4. **Tantangan:** Ubah field `role` di `User` menjadi enum `Role` dengan variant `Admin`, `Agent`, `Customer`. Tambahkan `#[serde(rename_all = "lowercase")]` di enum agar di JSON muncul sebagai `"admin"`, `"agent"`, `"customer"`. ⚠️ **OPTIONAL**

---

## Hasil Akhir Bab Ini

Setelah bab ini, struktur folder dan file harus seperti ini:

```
support-desk/
├── Cargo.toml
├── src/
│   ├── main.rs            ← update: tambah `mod models;`
│   └── models/
│       ├── mod.rs         ← NEW
│       ├── user.rs        ← NEW
│       ├── ticket.rs      ← NEW
│       └── api_response.rs ← NEW
└── target/
```

**File 1: `src/models/user.rs`**
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

**File 2: `src/models/ticket.rs`**
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketDto {
    pub subject: String,
    pub description: String,
    pub category: String,
    pub priority: String,
}
```

**File 3: `src/models/api_response.rs`**
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

**File 4: `src/models/mod.rs`**
```rust
pub mod api_response;
pub mod ticket;
pub mod user;

pub use api_response::ApiResponse;
pub use ticket::{CreateTicketDto, Ticket};
pub use user::User;
```

**Update File 5: `src/main.rs`** — Tambahkan `mod models;` di paling atas sebelum imports:
```rust
mod models;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;

// ... rest of code tetap sama (TicketFilters, handlers, routes)
```

**Verifikasi:**
```bash
cargo build
# Harus compile tanpa error

cargo run
curl http://localhost:3000/health
# Output: OK
```

Setelah ini, folder `src/models/` siap dipakai di bab-bab berikutnya.

---

Di bab berikutnya kita tambahkan validasi input dengan `validator` crate, karena saat ini kalau client kirim `subject` kosong, kita tidak punya cara untuk menolaknya.
