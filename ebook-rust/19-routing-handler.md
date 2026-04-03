# Bab 19: Routing dan Handler

Di bab sebelumnya kita sudah berhasil menjalankan server pertama dengan satu route `/health`. Topik bab ini: bagaimana mengatur lalu lintas request yang masuk ke server. Inilah inti dari **routing dan handler**.

---

## State Sebelumnya (Checklist sebelum mulai)

Pastikan sudah selesai Bab 18 dan memiliki:

- ✅ Project `support-desk` sudah dibuat dengan `cargo new`
- ✅ `Cargo.toml` sudah berisi semua dependencies (axum, tokio, serde, dll)
- ✅ `src/main.rs` sudah punya server dengan route `/health`
- ✅ Server bisa dijalankan dengan `cargo run` tanpa error
- ✅ Latihan Bab 18 sudah dicoba (minimal #3: eksplorasi error message)

Kalau ada yang belum, kembali ke Bab 18 terlebih dahulu.

---

## Analogi: Resepsionis Hotel

[ILUSTRASI: resepsionis hotel yang menerima tamu dan mengarahkan mereka ke departemen yang tepat — tamu yang mau check-in ke front desk, tamu yang mau pijat ke spa, tamu yang komplain ke manajer]

Bayangkan kamu datang ke hotel besar. Di pintu masuk ada resepsionis yang tugasnya satu: **mengarahkan tamu ke tempat yang tepat**.

Tamu mau check-in diarahkan ke counter A. Tamu mau pesan makanan diarahkan ke restoran di lantai 2. Tamu mau komplain disambungkan ke manajer.

**Router di Axum bekerja persis seperti resepsionis ini.** Setiap request HTTP yang masuk akan "disambut" oleh router, lalu diteruskan ke fungsi yang tepat berdasarkan dua hal: **URL-nya** (path) dan **jenis requestnya** (HTTP method).

---

## Router di Axum

Router di Axum dibuat dengan `Router::new()`, lalu kita tambahkan route satu per satu pakai `.route()`.

```rust
use axum::{routing::get, Router};

let app = Router::new()
    .route("/health", get(health_check))
    .route("/tickets", get(get_tickets));
```

Format dasar `.route()` adalah:

```
.route(PATH, METHOD(HANDLER))
```

- **PATH**: URL-nya, misalnya `"/tickets"` atau `"/tickets/{id}"`
- **METHOD**: HTTP method-nya (get, post, patch, delete)
- **HANDLER**: nama fungsi yang akan dipanggil

---

## HTTP Methods

HTTP method itu seperti "maksud kedatangan" tamu. Dalam konteks API:

| Method | Analogi | Kegunaan Umum |
|--------|---------|---------------|
| `GET` | Tanya informasi | Ambil data |
| `POST` | Serahkan formulir | Buat data baru |
| `PATCH` | Minta perubahan | Update sebagian data |
| `DELETE` | Minta hapus | Hapus data |

Di Axum, kita import method-method ini dari `axum::routing`:

```rust
use axum::routing::{get, post, patch, delete};
```

Satu path bisa menangani lebih dari satu method sekaligus:

```rust
.route("/tickets", get(get_tickets).post(create_ticket))
```

Artinya: kalau ada `GET /tickets` → panggil `get_tickets`. Kalau ada `POST /tickets` → panggil `create_ticket`.

---

## Handler Function

Handler itu fungsi Rust biasa, tapi ada dua syarat: harus `async`, dan return type-nya harus `impl IntoResponse`.

`IntoResponse` adalah trait, anggap saja ini "kontrak" yang bilang: "tipe ini bisa diubah jadi HTTP response." Banyak tipe di Axum yang sudah implement trait ini, seperti `Json<T>`, `StatusCode`, atau tuple `(StatusCode, Json<T>)`.

Handler paling sederhana:

```rust
async fn health_check() -> &'static str {
    "OK"
}
```

String `&'static str` sudah implement `IntoResponse`, jadi ini valid.

---

## Extractor: Ambil Data dari Request

Ini bagian yang bikin Axum terasa "ajaib". Axum punya sistem yang disebut **Extractor**, yaitu cara untuk mengambil data dari request HTTP langsung lewat parameter fungsi.

Jargon dulu: **extractor** = tipe khusus yang, ketika kamu taruh di parameter handler, Axum otomatis akan mengisinya dengan data dari request.

[ILUSTRASI: formulir hotel yang diisi otomatis — tamu cukup menyerahkan KTP, dan formulir langsung terisi nama, alamat, tanggal lahir tanpa perlu diketik manual]

### Path\<T\> — Data dari URL

Kalau URL kita ada bagian dinamis seperti `/tickets/{id}`, kita pakai `Path<T>` untuk mengambil nilai `:id`.

```rust
async fn get_ticket(Path(id): Path<u32>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "id": id, "title": "Contoh ticket" }
    })))
}
```

Penjelasan sintaks `Path(id): Path<u32>`: `Path<u32>` berarti kita ambil dari URL dengan tipe `u32` (bilangan bulat positif), lalu `Path(id)` "unpack" nilai di dalamnya ke variabel `id`.

Route-nya didefinisikan dengan `:id`:

```rust
.route("/{id}", get(get_ticket))
```

### Query\<T\> — Data dari Query String

Query string itu bagian URL setelah tanda `?`. Misalnya `/tickets?page=1&limit=10&status=open`.

Kita buat struct dulu untuk menampung parameternya:

```rust
#[derive(Deserialize)]
struct TicketFilters {
    page: Option<u32>,
    limit: Option<u32>,
    status: Option<String>,
}
```

Kenapa `Option`? Karena query string itu opsional, user mungkin tidak mengirim semua parameter. Kalau kita pakai `u32` langsung (tanpa `Option`), Axum akan error jika parameter tidak ada di URL.

Lalu di handler:

```rust
async fn get_tickets(Query(filters): Query<TicketFilters>) -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "page": filters.page.unwrap_or(1),
        "limit": filters.limit.unwrap_or(10)
    }))
}
```

`unwrap_or(1)` artinya: kalau `page` tidak dikirim (nilainya `None`), gunakan default `1`.

### Json\<T\> — Data dari Request Body

Untuk request `POST` atau `PATCH`, data biasanya dikirim di body dalam format JSON. Kita pakai extractor `Json<T>`.

```rust
async fn create_ticket(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    println!("Body diterima: {:?}", body);
    (StatusCode::CREATED, Json(json!({
        "success": true,
        "message": "Ticket berhasil dibuat"
    })))
}
```

Di sini kita pakai `Json<Value>`, artinya kita terima JSON apapun tanpa validasi struktur dulu. Di bab selanjutnya kita akan pakai struct tertentu supaya lebih aman.

> **Catatan penting:** Extractor `Json<T>` untuk request body harus diletakkan **terakhir** di antara parameter handler. Ini aturan Axum karena body hanya bisa dibaca sekali.

---

## Return Type Handler

Axum fleksibel soal apa yang bisa di-return dari handler. Beberapa opsi umum:

```rust
// Hanya status code
async fn delete_ticket() -> StatusCode {
    StatusCode::NO_CONTENT
}

// Hanya JSON
async fn get_tickets() -> Json<Value> {
    Json(json!({ "data": [] }))
}

// Status code + JSON (paling umum untuk API)
async fn create_ticket() -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(json!({ "success": true })))
}
```

Tuple `(StatusCode, Json<Value>)` adalah pola yang paling sering kita pakai. Status code-nya standar HTTP: `StatusCode::OK` (200), `StatusCode::CREATED` (201), `StatusCode::NOT_FOUND` (404), `StatusCode::BAD_REQUEST` (400), `StatusCode::INTERNAL_SERVER_ERROR` (500).

---

## Nested Routes

Kalau endpoint kita banyak, kita bisa kelompokkan dengan `Router::nest()`. Ini seperti membuat sub-resepsionis untuk setiap departemen.

```rust
fn ticket_routes() -> Router {
    Router::new()
        .route("/", get(get_tickets).post(create_ticket))
        .route("/{id}", get(get_ticket))
}

// Di main:
let app = Router::new()
    .route("/health", get(health_check))
    .nest("/tickets", ticket_routes());
```

Dengan `nest("/tickets", ticket_routes())`, `/` di dalam `ticket_routes` menjadi `/tickets`, dan `{id}` menjadi `/tickets/{id}`. Endpoint yang tersedia sekarang: `GET /health`, `GET /tickets`, `POST /tickets`, `GET /tickets/{id}`.

Ini membuat kode lebih terorganisir. Nanti kita bisa punya `user_routes()`, `auth_routes()`, dll., masing-masing di file terpisah.

---

## Kode Lengkap Bab Ini

```rust
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct TicketFilters {
    page: Option<u32>,
    limit: Option<u32>,
    #[serde(default)]
    status: Option<String>,
    priority: Option<String>,
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
        "status": filters.status,
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
        .route("/", get(get_tickets))
        .route("/", axum::routing::post(create_ticket))
        .route("/{id}", get(get_ticket))
        .route("/{id}", axum::routing::delete(delete_ticket))
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

Jalankan dengan `cargo run`, lalu coba dengan curl:

```bash
# Test health
curl http://localhost:3000/health

# Test GET tickets dengan filter
curl "http://localhost:3000/tickets?page=2&limit=5"

# Test GET ticket by ID
curl http://localhost:3000/tickets/42

# Test POST ticket
curl -X POST http://localhost:3000/tickets \
  -H "Content-Type: application/json" \
  -d '{"title": "Server down", "priority": "high"}'
```

---

## Latihan

Sebelum lanjut ke bab berikutnya, coba kerjakan ini:

1. **Tambah route DELETE:** buat handler `delete_ticket(Path(id): Path<u32>)` yang return `StatusCode::NO_CONTENT`. Tambahkan ke route `"{id}"` dengan `.delete(delete_ticket)`.

2. **Tambah field ke `TicketFilters`:** tambahkan field `priority: Option<String>` dan tampilkan nilainya di response JSON.

3. **Buat `user_routes()`:** buat fungsi baru dengan route `GET /users` dan `GET /users/{id}`, lalu nest-kan ke router utama di `/users`. ⚠️ **MANDATORY untuk Bab 20**

Kalau semua endpoint bisa diakses dan return response yang benar, kita siap lanjut ke bab berikutnya.

---

## Hasil Akhir Bab Ini

Setelah menyelesaikan semua latihan, `src/main.rs` kamu harus terlihat seperti ini:

```rust
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct TicketFilters {
    page: Option<u32>,
    limit: Option<u32>,
    #[serde(default)]
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
        "status": filters.status,
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

// dari latihan #1
async fn delete_ticket(Path(id): Path<u32>) -> StatusCode {
    println!("Menghapus ticket {}", id);
    StatusCode::NO_CONTENT
}

fn ticket_routes() -> Router {
    Router::new()
        .route("/", get(get_tickets))
        .route("/", axum::routing::post(create_ticket))
        .route("/{id}", get(get_ticket))
        .route("/{id}", axum::routing::delete(delete_ticket))
}

// dari latihan #3
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

**Verifikasi** — test semua endpoint:
```bash
cargo run

# Di terminal lain:
curl http://localhost:3000/health
# Output: OK

curl "http://localhost:3000/tickets?page=1&limit=5&priority=high&status=open"
# Output: {"success":true,"data":[],"page":1,"limit":5,"status":"open","priority":"high"}

curl http://localhost:3000/tickets/42
# Output: {"success":true,"data":{"id":42,"title":"Contoh ticket"}}

curl -X POST http://localhost:3000/tickets -H "Content-Type: application/json" -d '{"title":"Test"}'
# Output: {"success":true,"message":"Ticket berhasil dibuat"} dengan status 201

curl -X DELETE http://localhost:3000/tickets/42
# Output: kosong (body) dengan status 204

curl http://localhost:3000/users
# Output: {"success":true,"data":[]}

curl http://localhost:3000/users/1
# Output: {"success":true,"data":{"id":1,"name":"Contoh user"}}
```

Semuanya harus return 200 atau 204, tidak ada 404 atau error.
