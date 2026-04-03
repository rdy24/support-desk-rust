# Bab 27: Autentikasi - Register dan Login

Bayangkan kamu punya loker di gym. Loker itu dikunci dengan kode kombinasi. Kalau ada orang lain minta kode kamu, kamu nggak mau kasih, kan? Tapi gimana kalau petugas gym butuh verifikasi bahwa kamu pemilik loker itu tanpa kamu harus kasih kode aslinya?

Itulah inti dari autentikasi modern. Sistem kita perlu tahu "ini beneran kamu", tapi tanpa menyimpan password aslimu. Di bab ini, kita bangun dua fitur fundamental: **register** (daftar akun baru) dan **login** (masuk ke sistem).

JWT (token yang kamu dapatkan setelah login) akan dibahas detail di bab 28. Di bab ini, login akan kembalikan placeholder dulu, dan kita fokus ke bagian yang lebih kritis: **password hashing** dan **alur autentikasi**.

[ILUSTRASI: Dua jalur paralel — "Register" (pengguna baru → hash password → simpan ke DB → selesai) dan "Login" (pengguna lama → cari email → verifikasi hash → beri token)]

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 26:**
- Dashboard queries sudah di `DashboardRepository` dari "Hasil Akhir Bab 26"

**State Sebelumnya:**
Folder `src/repositories/` harus lengkap dengan 4 repositories (user, ticket, response, dashboard). AppState sudah include semuanya.

---

## Kenapa Password Tidak Boleh Disimpan Plaintext

**Plaintext** artinya teks apa adanya, tanpa diubah. Kalau kamu simpan password `rahasia123` langsung ke database, itulah plaintext.

Masalahnya: kalau database bocor (dan ini terjadi terus di dunia nyata), semua password langsung terbaca. Hacker bisa login sebagai siapapun.

Analoginya begini. Menyimpan password plaintext itu seperti memfotokopi KTP asli pengguna, lalu menyimpannya di laci kantor. Kalau laci dibobol, semua data identitas langsung ketahuan. Menyimpan password dalam bentuk hash lebih seperti menyimpan sidik jari pengguna. Kalau data bocor, hacker dapat sidik jari, tapi sidik jari nggak bisa dikembalikan jadi identitas asli. Kamu tetap bisa verifikasi "apa sidik jari ini cocok?" tanpa perlu tahu orangnya siapa.

**Hash** adalah fungsi satu arah: `password → hash` mudah, tapi `hash → password` hampir mustahil. Setiap kali login, kita hash password yang diketik, lalu bandingkan dengan hash yang tersimpan.

---

## argon2: Password Hashing yang Aman

Ada banyak algoritma hashing. **Argon2** adalah yang direkomendasikan saat ini karena dirancang lambat secara sengaja, sehingga brute force (coba ribuan kombinasi) jadi sangat mahal secara komputasi.

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
argon2 = "0.5"  # saat ebook ini ditulis, Maret 2026
```

Dua operasi utama yang kita pakai:

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

// Hash password (saat register)
let salt = SaltString::generate(&mut OsRng);
let hashed = Argon2::default()
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

// Verifikasi (saat login)
let parsed_hash = PasswordHash::new(&stored_hash)?;
Argon2::default().verify_password(input.as_bytes(), &parsed_hash)?;
```

**Salt** adalah string acak yang ditambahkan sebelum hashing. Ini mencegah dua pengguna dengan password sama menghasilkan hash yang sama, sehingga hacker tidak bisa pakai "rainbow table" (daftar hash yang sudah dihitung sebelumnya).

---

## AuthService

`AuthService` bertanggung jawab atas logika register dan login. Dia butuh akses ke database lewat `UserRepository`, sesuai pola yang sudah kita bangun di bab 25.

```
src/
├── services/
│   ├── mod.rs
│   └── auth_service.rs
```

Struktur dasarnya:

```rust
// src/services/auth_service.rs
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::{
    common::AppError,
    dto::{LoginDto, RegisterDto},
    models::User,
    repositories::UserRepository,
};

pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }
}
```

`AuthService::new` menerima `UserRepository` sebagai argumen, yaitu **dependency injection**. Service tidak membuat repo sendiri; dia menerima dari luar. Itulah kenapa mudah diganti saat testing.

---

## Register: Daftar Akun Baru

Alur register ada tiga langkah: cek apakah email sudah dipakai, hash password, lalu simpan user baru ke database.

```rust
pub async fn register(&self, dto: RegisterDto) -> Result<User, AppError> {
    // Langkah 1: Cek email sudah ada
    if let Some(_) = self.user_repo.find_by_email(&dto.email).await? {
        return Err(AppError::Conflict("Email sudah digunakan".to_string()));
    }

    // Langkah 2: Hash password
    let salt = SaltString::generate(&mut OsRng);
    let hashed = Argon2::default()
        .hash_password(dto.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .to_string();

    // Langkah 3: Simpan ke DB
    let user = self.user_repo
        .create(&dto.name, &dto.email, &hashed, "customer")
        .await?;

    Ok(user)
}
```

`AppError::Conflict` menghasilkan HTTP 409, yang memberi tahu client bahwa data sudah ada, bukan sekadar "server error". `map_err(|e| AppError::Internal(...))` mengonversi error dari library argon2 ke `AppError` kita; Rust memaksa kita eksplisit soal error handling. User yang dikembalikan dari `create()` hanya berisi data yang aman, tanpa field password hash.

---

## Login: Masuk ke Sistem

Alur login juga tiga langkah, tapi berbeda: cari user berdasarkan email, verifikasi password yang dimasukkan dengan hash yang tersimpan, lalu generate token.

```rust
pub async fn login(&self, dto: LoginDto) -> Result<String, AppError> {
    // Langkah 1: Cari user
    let user = self.user_repo
        .find_by_email(&dto.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Email atau password salah".to_string()))?;

    // Langkah 2: Verifikasi password
    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Argon2::default()
        .verify_password(dto.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Email atau password salah".to_string()))?;

    // Langkah 3: Return token (sementara)
    Ok("token_placeholder".to_string())
}
```

Perhatikan satu hal penting: pesan error untuk "email tidak ditemukan" dan "password salah" **sengaja dibuat sama**: `"Email atau password salah"`. Ini praktik keamanan standar. Kalau pesannya berbeda, hacker bisa tahu email mana yang terdaftar, lalu fokus mencoba password-nya saja.

`.ok_or_else()` mengubah `Option<User>` menjadi `Result<User, AppError>`. Kalau `find_by_email` kembalikan `None` (user tidak ada), langsung return `Unauthorized`.

[ILUSTRASI: Flowchart login — mulai dari "Cari user by email" → percabangan "Ada?" → Tidak: return error "Email atau password salah" → Ya: "Verifikasi password" → percabangan "Cocok?" → Tidak: return error yang sama → Ya: "Return token"]

---

## AuthHandler — Route HTTP

Handler bertugas menerima request HTTP, memanggil service, dan memformat response. Buat file baru:

```
src/
├── handlers/
│   └── auth_handler.rs
```

```rust
// src/handlers/auth_handler.rs
use axum::{extract::State, http::StatusCode, Json};
use crate::{
    common::AppError,
    dto::{LoginDto, RegisterDto},
    services::AuthService,
    AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let user = state.auth_service.register(dto).await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "data": user
        })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = state.auth_service.login(dto).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": { "token": token }
    })))
}
```

Daftarkan route di router utama:

```rust
// src/router.rs atau main.rs
use axum::routing::post;

let app = Router::new()
    .route("/auth/register", post(auth_handler::register))
    .route("/auth/login", post(auth_handler::login))
    .with_state(state);
```

Register mengembalikan `StatusCode::CREATED` (HTTP 201) karena memang membuat resource baru. Login cukup 200 OK.

---

## Ringkasan Alur

| Aksi | Endpoint | Langkah Kunci |
|------|----------|---------------|
| Register | POST /auth/register | Cek duplikat → hash password → simpan |
| Login | POST /auth/login | Cari user → verifikasi hash → return token |

Error yang mungkin muncul:

- `409 Conflict`: email sudah digunakan (register)
- `401 Unauthorized`: email/password salah (login)
- `500 Internal`: error tak terduga dari hashing atau DB

---

## Latihan

1. Tambahkan validasi input di `RegisterDto`: email harus format valid, password minimal 8 karakter. Gunakan `validator` crate yang sudah kita pelajari di bab 21.

2. Di handler `register`, setelah berhasil simpan user, coba langsung panggil `login` dengan credential yang sama dan kembalikan token sekaligus, sehingga user tidak perlu login lagi setelah register. Apa konsekuensi desain dari pendekatan ini?

3. Buat unit test untuk `AuthService::register`: skenario sukses, skenario email duplikat. Gunakan mock `UserRepository` atau database test terpisah. *(Petunjuk: lihat bab 26 tentang testing dengan SQLx.)*

4. Perhatikan pesan error di login yang sengaja dibuat ambigu. Dalam konteks internal (misalnya logging), apakah kamu tetap ingin log alasan sesungguhnya (email tidak ada vs password salah)? Implementasikan logging yang berbeda tanpa mengekspos perbedaan itu ke client.
