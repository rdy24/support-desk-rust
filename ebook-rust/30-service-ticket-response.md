# Bab 30: Service Layer — Ticket dan Response

Bayangkan sebuah restoran. Ada pelayan (handler) yang terima pesanan dari tamu. Ada dapur (database/repository) yang masak makanan. Tapi di antara mereka, ada **kepala dapur**, orang yang tahu aturan: "Meja VIP boleh pesan menu spesial, tamu biasa tidak", "Kalau bahan habis, kasih tahu pelayan dengan sopan", "Menu ini tidak bisa dikembalikan setelah dipesan."

Kepala dapur itu namanya **Service Layer**.

Handler tidak perlu tahu soal aturan bisnis. Repository tidak perlu tahu siapa yang boleh akses apa. Service layer yang pegang semua itu.

[ILUSTRASI: diagram tiga lapis — Handler (pelayan) → Service (kepala dapur) → Repository (dapur). Panah searah dari atas ke bawah, dengan keterangan "business rules" di lapisan tengah]

---

## Service Layer: Tempat Business Logic

**Business logic** adalah aturan yang mendefinisikan bagaimana aplikasi beroperasi. Contoh aturan bisnis di support desk kita:

Hanya customer yang boleh buat ticket. Customer hanya boleh lihat ticket miliknya sendiri. Ticket tidak boleh dihapus oleh siapapun. Agent bisa update ticket, customer tidak bisa.

Semua aturan itu **tidak boleh** ditaruh di handler (handler cuma routing) dan **tidak boleh** ditaruh di repository (repository cuma query database). Tempatnya di service.

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 29:**
- Custom extractor `AdminOnly`, `AgentOnly`, `CustomerOnly` sudah di middleware dari "Hasil Akhir Bab 29"
- Handler sekarang bisa pakai extractor itu untuk guard

**State Sebelumnya:**
Dari Bab 29, role-based access control dengan custom extractor sudah siap. Bab 30 fokus ke service layer untuk business logic (validasi ticket owner, update status rules, etc).

---

## TicketService

Buat file `src/services/ticket_service.rs`:

```rust
use uuid::Uuid;
use crate::{
    common::AppError,
    models::{Ticket, TicketResponse},
    repositories::{TicketRepository, ResponseRepository},
    dto::{CreateTicketDto, UpdateTicketDto, TicketFilters},
    auth::Claims,
};

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

`TicketService` menyimpan dua repository sebagai *dependency*, yang disebut **dependency injection**: service tidak bikin repository sendiri, melainkan menerimanya dari luar. Lebih mudah di-test, lebih fleksibel.

---

## create: Hanya Customer

```rust
pub async fn create(
    &self,
    dto: CreateTicketDto,
    claims: &Claims,
) -> Result<Ticket, AppError> {
    if claims.role != "customer" {
        return Err(AppError::Forbidden(
            "Hanya customer yang bisa membuat ticket".to_string()
        ));
    }

    let customer_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid user id")))?;

    self.ticket_repo.create(&dto, customer_id).await
}
```

Ada dua hal penting di sini. Pertama, role dicek di awal. Kalau bukan customer, langsung tolak tanpa perlu lanjut. Kedua, `customer_id` diambil dari token JWT, bukan dari body request. Kalau dari body, user bisa kirim ID orang lain dan buat ticket atas nama mereka. Token lebih aman karena sudah diverifikasi JWT.

`claims.sub` adalah *subject* dari JWT yang berisi user ID yang sedang login. `Uuid::parse_str` konversi string ke UUID, dengan `.map_err` untuk ubah error parsing jadi `AppError::Internal`.

---

## get_by_id: Dengan Cek Akses

```rust
pub async fn get_by_id(
    &self,
    ticket_id: Uuid,
    claims: &Claims,
) -> Result<Ticket, AppError> {
    let ticket = self.ticket_repo.find_by_id(ticket_id).await?
        .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

    self.check_access(&ticket, claims)?;
    Ok(ticket)
}
```

Polanya: ambil data dulu, lalu cek akses. Kenapa bukan cek akses dulu? Karena untuk cek akses, kita butuh data ticketnya (siapa pemiliknya). Tidak bisa cek tanpa data.

`.ok_or_else()` mengonversi `Option<T>` ke `Result<T, E>`. Kalau `None` (ticket tidak ada di DB), jadi `Err(AppError::NotFound)`.

---

## check_access: Private Method

```rust
fn check_access(&self, ticket: &Ticket, claims: &Claims) -> Result<(), AppError> {
    match claims.role.as_str() {
        "admin" => Ok(()),
        "agent" => Ok(()),
        "customer" => {
            if ticket.customer_id.to_string() == claims.sub {
                Ok(())
            } else {
                Err(AppError::Forbidden(
                    "Kamu tidak punya akses ke ticket ini".to_string()
                ))
            }
        }
        _ => Err(AppError::Forbidden("Role tidak dikenal".to_string())),
    }
}
```

Method ini `fn` biasa (bukan `async fn`) karena tidak ada operasi database, murni logika perbandingan.

`fn` tanpa `pub` artinya **private**: hanya bisa dipanggil dari dalam `TicketService`. Ini disengaja: cek akses adalah detail internal, tidak perlu diekspos ke luar. Pattern `match` di sini bersih: admin dan agent langsung `Ok(())`, customer dicek kepemilikan, role lain langsung forbidden.

---

## get_many: Filter Berbasis Role

```rust
pub async fn get_many(
    &self,
    filters: TicketFilters,
    claims: &Claims,
) -> Result<(Vec<Ticket>, i64), AppError> {
    let user_id = Uuid::parse_str(&claims.sub).ok();

    let (customer_filter, agent_filter) = match claims.role.as_str() {
        "customer" => (user_id, None),
        "agent"    => (None, None),
        "admin"    => (None, None),
        _ => return Err(AppError::Forbidden("Role tidak valid".to_string())),
    };

    self.ticket_repo.find_many(
        customer_filter,
        agent_filter,
        filters.status.as_deref(),
        filters.page.unwrap_or(1),
        filters.limit.unwrap_or(10),
    ).await
}
```

Return type `(Vec<Ticket>, i64)` adalah tuple berisi list ticket dan total count untuk pagination.

Bedanya di sini: customer dapat `customer_filter = Some(user_id)` sehingga repository menambahkan `WHERE customer_id = ?`, sementara agent dan admin mendapat kedua filter sebagai `None` sehingga semua ticket dikembalikan. `.ok()` pada `Uuid::parse_str` membiarkan error parsing jadi `None`. Kalau parsing gagal, query tidak akan match apapun, yang merupakan perilaku yang aman. `filters.page.unwrap_or(1)` memastikan nilai default kalau user tidak mengirim parameter page.

---

## update: Hanya Agent/Admin

```rust
pub async fn update(
    &self,
    ticket_id: Uuid,
    dto: UpdateTicketDto,
    claims: &Claims,
) -> Result<Ticket, AppError> {
    if claims.role == "customer" {
        return Err(AppError::Forbidden(
            "Customer tidak bisa mengubah ticket".to_string()
        ));
    }

    let updated = self.ticket_repo.update(ticket_id, &dto).await?
        .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

    Ok(updated)
}
```

Aturan bisnis: customer tidak boleh update ticket, misalnya mengubah status atau assignee. Itu wewenang agent dan admin.

Perhatikan kebalikan dari `create`: di sini yang diblokir adalah customer (`if role == "customer"`), bukan yang diizinkan. Kadang lebih mudah menulis "siapa yang tidak boleh" daripada "siapa yang boleh", tergantung konteks.

---

## delete: Selalu Forbidden

```rust
pub async fn delete(
    &self,
    _ticket_id: Uuid,
    _claims: &Claims,
) -> Result<(), AppError> {
    Err(AppError::Forbidden("Ticket tidak bisa dihapus".to_string()))
}
```

Sederhana tapi penting. Underscore `_` di depan parameter artinya "parameter ini ada tapi sengaja tidak dipakai", sehingga Rust tidak akan warning soal unused variable.

Method ini tetap ada meskipun selalu return error karena **handler tetap perlu memanggil service**, bukan langsung return error sendiri. Kalau logika berubah di masa depan (misalnya admin boleh hapus), cukup edit service tanpa menyentuh handler.

---

## add_response

```rust
pub async fn add_response(
    &self,
    ticket_id: Uuid,
    message: &str,
    claims: &Claims,
) -> Result<TicketResponse, AppError> {
    let ticket = self.ticket_repo.find_by_id(ticket_id).await?
        .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

    self.check_access(&ticket, claims)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid user id")))?;

    self.response_repo.create(ticket_id, user_id, message).await
}
```

Sebelum tambah response, dua syarat harus terpenuhi: ticket ada (`find_by_id`) dan user punya akses ke ticket tersebut (`check_access`). Baru setelah itu response disimpan ke database. Itulah kenapa service layer penting: dua validasi ini harus selalu jalan bersama, tidak bisa dilewati salah satunya.

---

## TicketHandler

Handler tidak boleh tahu soal business rules. Tugasnya: ekstrak data dari request, panggil service, kembalikan response.

```rust
// src/handlers/ticket_handler.rs

pub struct TicketHandler {
    service: TicketService,
}

impl TicketHandler {
    pub fn new(service: TicketService) -> Self {
        Self { service }
    }

    pub async fn create(
        &self,
        Json(dto): Json<CreateTicketDto>,
        Extension(claims): Extension<Claims>,
    ) -> Result<impl IntoResponse, AppError> {
        let ticket = self.service.create(dto, &claims).await?;
        Ok((StatusCode::CREATED, Json(ticket)))
    }

    pub async fn get_by_id(
        &self,
        Path(ticket_id): Path<Uuid>,
        Extension(claims): Extension<Claims>,
    ) -> Result<impl IntoResponse, AppError> {
        let ticket = self.service.get_by_id(ticket_id, &claims).await?;
        Ok(Json(ticket))
    }

    pub async fn delete(
        &self,
        Path(ticket_id): Path<Uuid>,
        Extension(claims): Extension<Claims>,
    ) -> Result<impl IntoResponse, AppError> {
        self.service.delete(ticket_id, &claims).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
```

Handler hanya tiga langkah: ekstrak → panggil service → wrap response. Tidak ada `if role == "customer"` di sini. Semua aturan sudah di service.

[ILUSTRASI: perbandingan dua kolom — "Handler yang salah" (berisi if-else role check, query database langsung) vs "Handler yang benar" (hanya ekstrak data, panggil service, return response)]

---

## Latihan

1. **Modifikasi `get_many`**: Tambahkan logika khusus untuk agent, di mana agent hanya boleh lihat ticket yang di-assign ke mereka atau yang belum di-assign (unassigned). Hint: tambahkan `agent_filter = Some(user_id)` untuk agent, dan sesuaikan query di repository.

2. **Tambah method `get_responses`**: Buat method di `TicketService` yang ambil semua response untuk satu ticket. Pastikan ada cek akses sebelum return data.

3. **Audit aturan bisnis**: Buka file `ticket.service.ts` di project TypeScript asli. Cari aturan bisnis yang belum diimplementasi di Rust. Tulis daftarnya, lalu implementasikan satu per satu.
