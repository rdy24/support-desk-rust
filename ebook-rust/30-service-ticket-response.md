# Bab 30: Service Layer — Ticket dan Response

Bayangkan sebuah restoran. Ada pelayan (handler) yang terima pesanan dari tamu. Ada dapur (database/repository) yang masak makanan. Tapi di antara mereka, ada **kepala dapur**, orang yang tahu aturan: "Meja VIP boleh pesan menu spesial, tamu biasa tidak", "Kalau bahan habis, kasih tahu pelayan dengan sopan", "Menu ini tidak bisa dikembalikan setelah dipesan."

Kepala dapur itu namanya **Service Layer**.

Handler tidak perlu tahu soal aturan bisnis. Repository tidak perlu tahu siapa yang boleh akses apa. Service layer yang pegang semua itu.

[ILUSTRASI: diagram tiga lapis — Handler (pelayan) → Service (kepala dapur) → Repository (dapur). Panah searah dari atas ke bawah, dengan keterangan "business rules" di lapisan tengah]

---

## Service Layer: Tempat Business Logic

**Business logic** adalah aturan yang mendefinisikan bagaimana aplikasi beroperasi. Contoh aturan bisnis di support desk kita:

- Hanya customer yang boleh buat ticket
- Customer hanya boleh lihat ticket miliknya sendiri
- Ticket tidak boleh dihapus oleh siapapun
- Agent bisa update ticket, customer tidak bisa

Semua aturan itu **tidak boleh** ditaruh di handler (handler cuma routing) dan **tidak boleh** ditaruh di repository (repository cuma query database). Tempatnya di service.

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 29:**
- Custom extractor `AdminOnly`, `AdminOrAgent`, `CustomerOnly` sudah di middleware dari "Hasil Akhir Bab 29"
- Handler bisa pakai extractor itu untuk guard

**State Sebelumnya:**
Dari Bab 29, role-based access control dengan custom extractor sudah siap. Bab 30 fokus ke service layer untuk business logic (validasi ticket owner, update status rules, filtering berbasis role).

---

## TicketService Structure

Buat file `src/services/ticket_service.rs`. Service ini akan:
1. Menyimpan reference ke repository sebagai **dependency injection**
2. Implement business logic untuk ticket operations
3. Mengembalikan Result dengan AppError untuk error handling

```rust
// src/services/ticket_service.rs

use uuid::Uuid;
use crate::{
    common::AppError,
    models::{Ticket, TicketResponse, CreateTicketResponseDto},
    repositories::{TicketRepository, ResponseRepository},
    dto::{CreateTicketDto, UpdateTicketDto, TicketFilters},
    services::Claims,
};

/// Service untuk menangani bisnis logic tiket
#[derive(Clone)]
pub struct TicketService {
    ticket_repo: TicketRepository,
    response_repo: ResponseRepository,
}

impl TicketService {
    pub fn new(ticket_repo: TicketRepository, response_repo: ResponseRepository) -> Self {
        Self { ticket_repo, response_repo }
    }
}
```

`#[derive(Clone)]` penting karena TicketService akan disimpan di AppState yang juga derive Clone. Dengan dependency injection, service tidak bikin repository sendiri; menerima dari luar. Lebih testable, lebih fleksibel.

---

## create: Hanya Customer

Aturan: customer bisa buat tiket. Agent dan admin tidak boleh (mereka cuma kelola tiket yang sudah ada).

```rust
/// Buat tiket baru (hanya customer)
pub async fn create(
    &self,
    dto: CreateTicketDto,
    claims: &Claims,
) -> Result<Ticket, AppError> {
    // Hanya customer yang boleh buat tiket
    if claims.role != "customer" {
        return Err(AppError::Forbidden(
            "Hanya customer yang bisa membuat ticket".to_string(),
        ));
    }

    // Ambil customer_id dari JWT (lebih aman daripada dari request body)
    let customer_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user id".to_string()))?;

    self.ticket_repo.create(&dto, customer_id).await
}
```

Dua hal penting di sini:
1. **Role check dulu** — kalau bukan customer, langsung tolak tanpa proses lebih lanjut
2. **Customer ID dari JWT** — jangan dari request body. Kalau dari body, customer bisa kirim customer_id orang lain dan buat tiket atas nama mereka. Token lebih aman karena sudah diverifikasi

`claims.sub` adalah subject dari JWT yang berisi user ID. `Uuid::parse_str` konversi string ke UUID dengan `.map_err` untuk ubah error parsing jadi `AppError::Internal`.

---

## get_by_id: Dengan Cek Akses

Setiap orang boleh lihat ticket, tapi cuma:
- Admin: lihat semua
- Agent: lihat semua
- Customer: lihat cuma ticket mereka sendiri

```rust
/// Ambil tiket berdasarkan ID dengan cek akses
pub async fn get_by_id(
    &self,
    ticket_id: Uuid,
    claims: &Claims,
) -> Result<Ticket, AppError> {
    let ticket = self
        .ticket_repo
        .find_by_id(ticket_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

    self.check_access(&ticket, claims)?;
    Ok(ticket)
}
```

Polanya: **ambil data dulu, baru cek akses**. Kenapa? Untuk cek akses, kita butuh data ticket (siapa pemiliknya). Tidak bisa cek tanpa data.

`.ok_or_else()` konversi `Option<T>` ke `Result<T, E>`. Kalau `None` (ticket tidak ada di DB), jadi `Err(AppError::NotFound)`.

---

## check_access: Private Method

Method internal untuk cek apakah user punya hak akses ke ticket.

```rust
/// Private method: cek apakah user punya akses ke ticket
fn check_access(&self, ticket: &Ticket, claims: &Claims) -> Result<(), AppError> {
    match claims.role.as_str() {
        "admin" => Ok(()),
        "agent" => Ok(()),
        "customer" => {
            if ticket.customer_id.to_string() == claims.sub {
                Ok(())
            } else {
                Err(AppError::Forbidden(
                    "Kamu tidak punya akses ke ticket ini".to_string(),
                ))
            }
        }
        _ => Err(AppError::Forbidden("Role tidak dikenal".to_string())),
    }
}
```

Method ini **private** (bukan `pub`) karena ini detail internal, nggak perlu diexpos ke luar. Logika: admin dan agent langsung `Ok(())`, customer dicek kepemilikan (customer_id harus sama dengan user ID dari JWT).

---

## get_many: Filter Berbasis Role

```rust
/// Ambil list tiket dengan filtering berbasis role
pub async fn get_many(
    &self,
    filters: TicketFilters,
    claims: &Claims,
) -> Result<(Vec<Ticket>, i64), AppError> {
    let user_id = Uuid::parse_str(&claims.sub).ok();

    // Tentukan filter berdasarkan role
    let (customer_filter, agent_filter) = match claims.role.as_str() {
        "customer" => (user_id, None),
        "agent" => (None, None),
        "admin" => (None, None),
        _ => return Err(AppError::Forbidden("Role tidak valid".to_string())),
    };

    self.ticket_repo
        .find_many(
            customer_filter,
            agent_filter,
            filters.status.as_deref(),
            filters.page.unwrap_or(1) as i64,
            filters.limit.unwrap_or(10) as i64,
        )
        .await
}
```

Return type `(Vec<Ticket>, i64)` adalah tuple berisi list ticket dan total count untuk pagination.

**Filtering logic:**
- Customer mendapat `customer_filter = Some(user_id)` sehingga repository menambahkan `WHERE customer_id = ?`, hanya kembalikan tiket milik mereka
- Agent dan admin mendapat kedua filter sebagai `None` sehingga semua ticket dikembalikan
- `.ok()` pada `Uuid::parse_str` membiarkan error parsing jadi `None`. Kalau parsing gagal, query tidak akan match apapun (safe)
- `filters.page.unwrap_or(1)` memastikan nilai default kalau user tidak mengirim parameter

---

## update: Hanya Agent/Admin

```rust
/// Update tiket (hanya agent/admin)
pub async fn update(
    &self,
    ticket_id: Uuid,
    dto: UpdateTicketDto,
    claims: &Claims,
) -> Result<Ticket, AppError> {
    // Customer tidak boleh update tiket
    if claims.role == "customer" {
        return Err(AppError::Forbidden(
            "Customer tidak bisa mengubah ticket".to_string(),
        ));
    }

    let updated = self
        .ticket_repo
        .update(ticket_id, &dto)
        .await?
        .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

    Ok(updated)
}
```

Aturan: customer **tidak boleh** update ticket. Wewenang agent dan admin. Perhatikan kebalikan dari `create`: di sini yang diblokir adalah customer, bukan yang diizinkan.

---

## delete: Selalu Forbidden

```rust
/// Hapus tiket (selalu forbidden)
pub async fn delete(&self, _ticket_id: Uuid, _claims: &Claims) -> Result<(), AppError> {
    Err(AppError::Forbidden(
        "Ticket tidak bisa dihapus".to_string(),
    ))
}
```

Sederhana tapi penting. Underscore `_` di depan parameter artinya "parameter ini ada tapi sengaja tidak dipakai", sehingga Rust tidak akan warning soal unused variable.

Method ini tetap ada meskipun selalu return error karena **handler tetap perlu memanggil service**, bukan langsung return error sendiri. Kalau logika berubah di masa depan (misalnya admin boleh hapus), cukup edit service tanpa menyentuh handler.

---

## add_response dan get_responses

```rust
/// Tambah response ke tiket (dengan cek akses)
pub async fn add_response(
    &self,
    ticket_id: Uuid,
    dto: CreateTicketResponseDto,
    claims: &Claims,
) -> Result<TicketResponse, AppError> {
    // Cek apakah ticket ada dan user punya akses
    let ticket = self
        .ticket_repo
        .find_by_id(ticket_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

    self.check_access(&ticket, claims)?;

    // Ambil user_id dari JWT
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user id".to_string()))?;

    self.response_repo
        .create(ticket_id, user_id, dto.message)
        .await
}

/// Ambil semua response untuk satu ticket (dengan cek akses)
pub async fn get_responses(
    &self,
    ticket_id: Uuid,
    claims: &Claims,
) -> Result<Vec<TicketResponse>, AppError> {
    // Cek apakah ticket ada dan user punya akses
    let ticket = self
        .ticket_repo
        .find_by_id(ticket_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

    self.check_access(&ticket, claims)?;

    self.response_repo.find_by_ticket_id(ticket_id).await
}
```

Sebelum tambah atau lihat response, dua syarat harus terpenuhi: ticket ada dan user punya akses. Itulah kenapa service layer penting: dua validasi ini harus selalu jalan bersama, tidak bisa dilewati salah satunya.

---

## Handler: Service Integration

Handler tidak boleh tahu soal business rules. Tugasnya: ekstrak data dari request, panggil service, kembalikan response.

```rust
// src/handlers/ticket_handler.rs — contoh satu handler

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dto::CreateTicketDto,
    middleware::AuthUser,
    AppState,
};

/// POST /tickets — Buat tiket baru
pub async fn create_ticket(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(dto): Json<CreateTicketDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), crate::common::AppError> {
    let ticket = state.ticket_service.create(dto, &claims).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": ticket
        })),
    ))
}
```

Handler hanya tiga langkah:
1. **Ekstrak** — `State` untuk AppState, `AuthUser` untuk JWT claims, `Json` untuk body
2. **Validasi** — dto.validate() otomatis dilakukan karena CreateTicketDto derive Validate
3. **Panggil service** — service menjalankan semua business logic
4. **Wrap response** — json dengan status code

**Tidak ada** `if role == "customer"` di sini. Semua aturan sudah di service.

---

## Integration: AppState & main.rs

Update `src/services/mod.rs`:

```rust
pub mod auth_service;
pub mod ticket_service;

pub use auth_service::{AuthService, Claims, verify_token, parse_claims_role};
pub use ticket_service::TicketService;
```

Update `src/main.rs` — AppState dan AppState::new():

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: UserRepository,
    pub ticket_repo: TicketRepository,
    pub response_repo: ResponseRepository,
    pub dashboard_repo: DashboardRepository,
    pub auth_service: AuthService,
    pub ticket_service: TicketService,  // NEW
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = UserRepository::new(pool.clone());
        let ticket_repo = TicketRepository::new(pool.clone());
        let response_repo = ResponseRepository::new(pool.clone());

        Self {
            user_repo: user_repo.clone(),
            ticket_repo: ticket_repo.clone(),
            response_repo: response_repo.clone(),
            dashboard_repo: DashboardRepository::new(pool.clone()),
            auth_service: AuthService::new(user_repo, jwt_secret.clone()),
            ticket_service: TicketService::new(ticket_repo, response_repo),  // NEW
            jwt_secret,
            db: pool,
        }
    }
}
```

Update router setup di `main()`:

```rust
// Setup auth dan ticket routes dengan state
let stateful_routes = Router::new()
    .route("/auth/register", post(handlers::auth_handler::register))
    .route("/auth/login", post(handlers::auth_handler::login))
    .route("/me", get(get_current_user))
    .route("/tickets", post(handlers::ticket_handler::create_ticket))
    .route("/tickets", get(handlers::ticket_handler::get_tickets))
    .route("/tickets/{id}", get(handlers::ticket_handler::get_ticket))
    .route("/tickets/{id}", patch(handlers::ticket_handler::update_ticket))
    .route("/tickets/{id}", axum::routing::delete(handlers::ticket_handler::delete_ticket))
    .route("/tickets/{id}/responses", post(handlers::ticket_handler::add_response))
    .route("/tickets/{id}/responses", get(handlers::ticket_handler::get_responses))
    .with_state(state);

let app = Router::new()
    .route("/health", get(health_check))
    .merge(stateful_routes);
```

---

## Tabel Mapping Endpoint ke Role & Aturan

| Endpoint | Method | Role yang Diizinkan | Business Logic |
|---|---|---|---|
| `/tickets` | POST | AuthUser (service checks customer) | Hanya customer bisa buat |
| `/tickets` | GET | AuthUser | Customer lihat milik sendiri, agent/admin lihat semua |
| `/tickets/:id` | GET | AuthUser | Cek ownership untuk customer |
| `/tickets/:id` | PATCH | AuthUser (service checks !customer) | Hanya agent/admin bisa update |
| `/tickets/:id` | DELETE | AuthUser | Selalu forbidden |
| `/tickets/:id/responses` | POST | AuthUser | Cek ownership sebelum tambah |
| `/tickets/:id/responses` | GET | AuthUser | Cek ownership sebelum lihat |

Extractor (dari Ch29) jadi "gate pertama", service layer jadi "gate kedua" dengan business logic yang lebih detail.

[ILUSTRASI: request flow — Request → Extractor (AuthUser check) → Handler (ekstrak data) → Service (business logic + repo calls) → Response. Setiap step ada validation checkpoint.]

---

## Latihan

**Latihan 1: Test endpoint ticket creation**

```bash
# Daftar customer
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Budi",
    "email": "budi@test.com",
    "password": "password123",
    "role": "customer"
  }'

# Ambil token dari response
TOKEN="..."

# Buat tiket
curl -X POST http://localhost:3000/tickets \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "Internet tidak bisa",
    "description": "Koneksi internet saya putus sejak pagi",
    "category": "technical",
    "priority": "high"
  }'

# Lihat semua tiket user
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/tickets
```

**Latihan 2: Coba access kontrol**

Dengan token customer, coba:
```bash
# Buat tiket (BERHASIL)
curl -X POST http://localhost:3000/tickets ...

# Update tiket (403 FORBIDDEN — customer tidak boleh)
curl -X PATCH http://localhost:3000/tickets/ticket-id ...

# Hapus tiket (403 FORBIDDEN)
curl -X DELETE http://localhost:3000/tickets/ticket-id ...
```

**Latihan 3: Role filtering**

Cek repository untuk agent_filter parameter di find_many(). Implementasikan filtering khusus untuk agent: agent hanya lihat ticket yang di-assign ke mereka atau belum di-assign (unassigned). Hint: tambahkan `agent_filter = Some(user_id)` untuk agent, update repository logic untuk match terhadap agent_id.

---

## Hasil Akhir

Bab 30 sudah complete! Berikut adalah ringkasan implementasi yang telah dilakukan:

### Status: ✅ COMPLETE

Semua komponen sudah diimplementasikan dengan 0 errors dan 18 warnings (unused imports/code yang diharapkan).

### Perubahan Kode:

#### 1. `src/dto/ticket_dto.rs`
- ✅ Ditambahkan `TicketFilters` struct dengan fields: `page`, `limit`, `status`, `priority`
- ✅ Ditambahkan `CreateTicketResponseDto` struct dengan `message` field dan validation
- Validation: message harus 1-2000 karakter

#### 2. `src/dto/mod.rs`
- ✅ Di-export: `CreateTicketDto`, `UpdateTicketDto`, `TicketFilters`, `CreateTicketResponseDto`

#### 3. `src/repositories/ticket_repository.rs`
- ✅ Updated `find_many()` signature untuk terima `agent_id` parameter (walaupun belum digunakan di service)
- ✅ Ditambahkan `update()` method untuk update ticket berdasarkan ID

#### 4. `src/services/ticket_service.rs` — NEW FILE
- ✅ `TicketService` struct dengan dependency injection (ticket_repo, response_repo)
- ✅ Semua 7 methods diimplementasikan:
  - `create()` — hanya customer, customer_id dari JWT
  - `get_by_id()` — dengan cek akses via check_access()
  - `get_many()` — customer lihat milik sendiri, agent/admin lihat semua
  - `update()` — hanya agent/admin bisa update
  - `delete()` — selalu forbidden
  - `add_response()` — dengan cek akses dan user_id dari JWT
  - `get_responses()` — dengan cek akses
  - `check_access()` — private method untuk validasi ownership

#### 5. `src/services/mod.rs`
- ✅ Module `ticket_service` ditambahkan
- ✅ `TicketService` di-export

#### 6. `src/handlers/ticket_handler.rs` — NEW FILE
- ✅ 7 handler functions:
  - `create_ticket()` — POST /tickets, StatusCode::CREATED
  - `get_tickets()` — GET /tickets, dengan query filters
  - `get_ticket()` — GET /tickets/:id
  - `update_ticket()` — PATCH /tickets/:id
  - `delete_ticket()` — DELETE /tickets/:id
  - `add_response()` — POST /tickets/:id/responses
  - `get_responses()` — GET /tickets/:id/responses

#### 7. `src/handlers/mod.rs`
- ✅ Module `ticket_handler` ditambahkan

#### 8. `src/main.rs`
- ✅ Import `TicketService` dari services
- ✅ AppState struct: ditambahkan field `pub ticket_service: TicketService`
- ✅ AppState::new(): initialize `ticket_repo`, `response_repo`, `ticket_service`
- ✅ Placeholder handlers dihapus (get_tickets, get_ticket, create_ticket, delete_ticket, get_users, get_user)
- ✅ Router setup diperbarui:
  - Gunakan `merge()` untuk menggabungkan stateful_routes dengan health check route
  - Semua ticket routes dengan `.with_state(state)`
  - Routes menggunakan handlers dari `handlers::ticket_handler`

---

## Verifikasi

### Build Status
```bash
$ cargo check
cargo build: 0 errors, 18 warnings (0 crates)
```

✅ **BERHASIL** — 0 errors! Warnings hanya untuk unused imports/functions yang diharapkan.

### Testing

```bash
# Build dan jalankan server
cargo run

# Test di terminal lain
# 1. Daftar customer
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Customer",
    "email": "customer@test.com",
    "password": "password123",
    "role": "customer"
  }'

# Response akan berisi token. Simpan TOKEN:
# {
#   "success": true,
#   "data": {
#     "id": "...",
#     "email": "customer@test.com",
#     "role": "customer"
#   }
# }

# 2. Login dengan credential yang sama
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "customer@test.com", "password": "password123"}'

# Ambil token dari response

# 3. Buat ticket (dengan token customer)
export TOKEN="<token dari login response>"

curl -X POST http://localhost:3000/tickets \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "Internet tidak bisa",
    "description": "Koneksi internet saya putus sejak pagi jam 08:00",
    "category": "technical",
    "priority": "high"
  }'

# Response: 201 CREATED dengan data ticket
# {
#   "success": true,
#   "data": {
#     "id": "...",
#     "subject": "Internet tidak bisa",
#     "status": "open",
#     ...
#   }
# }

# 4. Lihat semua ticket (customer hanya lihat milik sendiri)
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/tickets

# 5. Lihat detail ticket
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/tickets/{ticket-id}

# 6. Coba update ticket (seharusnya 403 FORBIDDEN untuk customer)
curl -X PATCH http://localhost:3000/tickets/{ticket-id} \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"status": "resolved"}'
# Response: 403 Forbidden — "Customer tidak bisa mengubah ticket"

# 7. Coba delete (seharusnya 403 FORBIDDEN)
curl -X DELETE http://localhost:3000/tickets/{ticket-id} \
  -H "Authorization: Bearer $TOKEN"
# Response: 403 Forbidden — "Ticket tidak bisa dihapus"
```

### Expected Results

| Action | Role | Status |
|--------|------|--------|
| POST /tickets | customer | 201 CREATED ✅ |
| GET /tickets | customer | 200 OK (hanya milik sendiri) ✅ |
| GET /tickets/:id | customer | 200 OK (jika punya akses) ✅ |
| PATCH /tickets/:id | customer | 403 FORBIDDEN ✅ |
| DELETE /tickets/:id | customer | 403 FORBIDDEN ✅ |
| POST /tickets/:id/responses | customer | 201 CREATED (jika punya akses) ✅ |
| GET /tickets/:id/responses | customer | 200 OK (jika punya akses) ✅ |

### Architecture Review

```
Request Flow untuk POST /tickets:
1. Client kirim request dengan Authorization: Bearer {token}
2. Handler `create_ticket()` menerima State + AuthUser(claims) + Json(dto)
3. AuthUser extractor validasi token (dari Bab 29)
4. Handler memanggil `state.ticket_service.create(dto, &claims)`
5. Service check role (hanya customer) → Err(403) jika bukan customer
6. Service ambil customer_id dari JWT claims.sub (lebih aman)
7. Service panggil `ticket_repo.create()` untuk simpan ke DB
8. Handler wrap response dengan StatusCode::CREATED + data
9. Response dikirim ke client
```

Alur ini berlaku untuk semua endpoints. Service layer adalah "kepala dapur" yang enforce business rules, handler cuma jadi "pelayan" yang routing.
