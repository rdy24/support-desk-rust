# Bab 31: Service Layer — User dan Dashboard

Auth sudah berjalan. Sekarang saatnya bangun bagian yang mengurus **siapa saja yang ada di sistem** dan **bagaimana kondisi sistem secara keseluruhan**.

---

## Analogi: HRD dan Bagian Pelaporan

Bayangkan sebuah perusahaan dengan dua divisi yang berbeda tugas. **HRD** mengurus data karyawan: siapa yang masuk, siapa yang keluar, update jabatan, lihat profil. **Bagian Pelaporan** tugasnya cuma satu: bikin laporan bulanan, berapa tiket masuk, berapa yang selesai, berapa yang masih pending.

HRD tidak ikut campur laporan. Bagian pelaporan tidak ikut campur data karyawan. Keduanya punya *sumber data sendiri* (repository) dan *staf sendiri* (service).

Itulah `UserService` dan `DashboardService` di bab ini.

[ILUSTRASI: dua gedung kantor terpisah — satu bertulis "HRD / UserService" dan satu "Pelaporan / DashboardService", keduanya terhubung ke "Database" di tengah]

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 30:**
- TicketService dengan business logic (owner validation, status transitions) sudah lengkap di "Hasil Akhir Bab 30"
- ResponseService untuk ticket responses juga sudah ada

**State Sebelumnya:**
Dari Bab 30, service layer untuk ticket sudah lengkap dengan validasi business logic. Bab 31 tambah UserService dan DashboardService untuk mengelola users dan metrics.

---

## UserService

`UserService` adalah "HRD"-nya aplikasi kita. Dia menerima permintaan dari handler, lalu meneruskannya ke `UserRepository` untuk berurusan dengan database.

```rust
pub struct UserService {
    user_repo: UserRepository,
}
```

Dependency injection sederhana: `UserService` membawa `UserRepository` sebagai "anak buah"-nya.

---

### get_me — Profil Sendiri

Endpoint `GET /users/me` memungkinkan user melihat profilnya sendiri berdasarkan token JWT yang dia kirim.

```rust
pub async fn get_me(&self, claims: &Claims) -> Result<User, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid user id")))?;

    self.user_repo.find_by_id(user_id).await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
}
```

`claims.sub` berisi user ID dalam bentuk string dari JWT, lalu diparse jadi `Uuid`. Kalau parsing gagal, itu error internal yang seharusnya tidak terjadi kalau token valid. Setelah itu user dicari di database. Kalau tidak ada, return `NotFound`.

`ok_or_else` adalah idiom Rust untuk mengubah `Option<T>` jadi `Result<T, E>`. Kalau `Option` bernilai `None`, jalankan closure untuk buat error.

---

### get_all — List dengan Filter Role

Admin bisa lihat semua user, dengan opsi filter berdasarkan role dan pagination.

```rust
pub async fn get_all(&self, role: Option<&str>, page: i64, limit: i64) -> Result<(Vec<User>, i64), AppError> {
    self.user_repo.find_all(role, page, limit).await
}
```

`Option<&str>` untuk `role` artinya filter ini opsional. Kalau tidak dikirim, tampilkan semua. Return type `(Vec<User>, i64)` adalah tuple: list user plus total count untuk pagination. Method ini dipakai oleh tiga endpoint sekaligus: `GET /agents` dengan filter role `"agent"`, `GET /customers` dengan filter role `"customer"`, dan `GET /users` yang bisa tanpa filter atau dengan query param.

---

### get_by_id

Admin bisa lihat detail satu user berdasarkan ID.

```rust
pub async fn get_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
    self.user_repo.find_by_id(user_id).await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
}
```

Sama polanya dengan `get_me`, bedanya ID langsung diterima dari path parameter, bukan dari JWT claims.

---

### update

Admin bisa update data user: nama, email, role, dll.

```rust
pub async fn update(&self, user_id: Uuid, dto: UpdateUserDto) -> Result<User, AppError> {
    self.user_repo.update(user_id, &dto).await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
}
```

`UpdateUserDto` (Data Transfer Object) adalah struct yang berisi field-field yang boleh diupdate. Itulah kenapa pakai DTO terpisah, bukan langsung struct `User`, karena kita tidak mau semua field bisa diubah sembarangan. Misalnya `created_at` atau `password` tidak boleh diubah lewat endpoint ini.

---

### delete — Tidak Boleh Hapus Diri Sendiri

Ada aturan bisnis khusus di sini: **admin tidak boleh menghapus akunnya sendiri**.

```rust
pub async fn delete(&self, target_id: Uuid, requester_claims: &Claims) -> Result<(), AppError> {
    if target_id.to_string() == requester_claims.sub {
        return Err(AppError::Forbidden("Tidak bisa menghapus akun sendiri".to_string()));
    }

    self.user_repo.delete(target_id).await?;
    Ok(())
}
```

Logikanya: bandingkan `target_id` (siapa yang mau dihapus) dengan `requester_claims.sub` (siapa yang meminta). Kalau sama, tolak dengan `403 Forbidden`. Logika ini ada di service karena ini aturan bisnis, bukan urusan HTTP.

[ILUSTRASI: diagram alur delete — "Apakah target_id == requester_id?" → Ya → Return Forbidden → Tidak → Hapus dari database]

---

## DashboardService

"Bagian Pelaporan"-nya aplikasi kita. Tugasnya satu.

```rust
pub struct DashboardService {
    dashboard_repo: DashboardRepository,
}

impl DashboardService {
    pub async fn get_stats(&self) -> Result<DashboardStats, AppError> {
        self.dashboard_repo.get_stats().await
    }
}
```

`DashboardStats` adalah struct yang berisi angka-angka statistik: total tiket, tiket open, tiket closed, total user, dll. Semua angka itu dikumpulkan oleh `DashboardRepository` lewat query agregasi ke database.

`DashboardRepository` dipisah sendiri karena query statistik biasanya kompleks, dengan banyak `COUNT`, `GROUP BY`, dan join beberapa tabel. Lebih bersih dibanding mencampurnya ke `UserRepository` atau `TicketRepository`.

---

## UserHandler dan DashboardHandler

Handler adalah "resepsionis" yang menerima request HTTP, validasi, lalu teruskan ke service.

Struktur `UserHandler`:

```rust
pub struct UserHandler {
    user_service: UserService,
}

impl UserHandler {
    // GET /users/me
    pub async fn get_me(&self, claims: Claims) -> impl IntoResponse { ... }

    // GET /agents
    pub async fn get_agents(&self, query: Query<PaginationQuery>) -> impl IntoResponse { ... }

    // GET /customers
    pub async fn get_customers(&self, query: Query<PaginationQuery>) -> impl IntoResponse { ... }

    // GET /users
    pub async fn get_all(&self, query: Query<UserFilterQuery>) -> impl IntoResponse { ... }

    // GET /users/{id}
    pub async fn get_by_id(&self, Path(id): Path<Uuid>) -> impl IntoResponse { ... }

    // PATCH /users/{id}
    pub async fn update(&self, Path(id): Path<Uuid>, Json(dto): Json<UpdateUserDto>) -> impl IntoResponse { ... }

    // DELETE /users/{id}
    pub async fn delete(&self, Path(id): Path<Uuid>, claims: Claims) -> impl IntoResponse { ... }
}
```

Dan `DashboardHandler`:

```rust
pub struct DashboardHandler {
    dashboard_service: DashboardService,
}

impl DashboardHandler {
    // GET /dashboard/stats
    pub async fn get_stats(&self) -> impl IntoResponse { ... }
}
```

Setiap handler method mengikuti pola yang sama: terima parameter dari request (path, query, body, claims), panggil method service yang sesuai, lalu return response JSON kalau sukses. Kalau error, `AppError` otomatis dikonversi ke HTTP response yang tepat.

---

## Siap untuk Bab 32!

Semua piece sudah ada: `UserRepository` dan `DashboardRepository` untuk urusan database, `UserService` dan `DashboardService` untuk logika bisnis, serta `UserHandler` dan `DashboardHandler` untuk urusan HTTP.

Yang belum: **menghubungkan semuanya jadi satu Router Axum yang utuh**. Di bab 32, kita akan buat router lengkap yang mendaftarkan semua endpoint, memasang middleware autentikasi di tempat yang tepat, dan merangkai semua dependency dari handler sampai repository.

---

## Latihan

1. **Buat struct `UserService`** dengan field `user_repo: UserRepository`. Implementasikan method `get_me` dan `get_by_id`. Pastikan keduanya mengembalikan `AppError::NotFound` kalau user tidak ada.

2. **Tambahkan validasi di `update`**: sebelum memanggil repository, cek apakah `dto` punya minimal satu field yang tidak `None`. Kalau semua field `None` (tidak ada yang diubah), return `AppError::BadRequest("Tidak ada field yang diupdate".to_string())`.

3. **Tulis test untuk `delete`**: buat dua skenario. Pertama, hapus user yang berbeda ID (harus sukses). Kedua, hapus dengan `target_id` sama dengan `requester_claims.sub` (harus return `AppError::Forbidden`).

4. **Eksplorasi**: `DashboardService` sangat sederhana karena semua logika ada di repository. Menurut kamu, kapan lebih baik menaruh logika di service vs langsung di repository? Tulis pendapatmu dalam komentar kode.
