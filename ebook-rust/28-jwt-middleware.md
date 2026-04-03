# Bab 28: Autentikasi — JWT dan Middleware

Bayangkan kamu masuk ke sebuah gedung kantor. Satpam minta KTP, verifikasi data kamu, lalu kasih kartu akses (badge) yang berlaku satu minggu. Setiap kali mau masuk ruangan, kamu tinggal tempel badge itu; satpam nggak perlu telepon HRD lagi buat cek siapa kamu. Semua info sudah ada di badge.

JWT (JSON Web Token) persis kayak badge itu. Sekali login berhasil, server kasih token. Setiap request berikutnya, kamu bawa token itu di header, dan server langsung tahu kamu siapa tanpa perlu query database lagi.

[ILUSTRASI: diagram alur login → server generate token → client simpan token → client kirim token di setiap request → server verifikasi token]

---

## Kunci Jawaban & State Sebelumnya

**Kunci Jawaban Latihan Bab 27:**
- Register handler dengan password hashing (argon2) sudah di handler auth dari "Hasil Akhir Bab 27"
- Login handler dengan password verification sudah lengkap

**State Sebelumnya:**
Dari Bab 27, folder `src/handlers/` harus punya auth handler lengkap dengan register & login. `src/services/` punya auth service dengan hash/verify password logic.

---

## Apa Itu JWT?

JWT punya tiga bagian, dipisah titik:

```
Header.Payload.Signature
```

- **Header:** algoritma enkripsi yang dipakai (biasanya `HS256`)
- **Payload:** data yang kamu simpan, misalnya user id, email, role
- **Signature:** tanda tangan digital, dibuat dari header + payload + secret key

Karena ada signature, kalau ada yang coba ubah payload-nya, signaturenya langsung invalid. Token nggak bisa dipalsuin tanpa tahu secret key.

Beberapa istilah penting: *claims* adalah data yang disimpan di payload JWT, `sub` adalah konvensi standar untuk menyimpan id user ("subject"), dan `exp` adalah timestamp kapan token expired.

Tambahkan dependency di `Cargo.toml`:

```toml
[dependencies]
jsonwebtoken = "9"  # saat ebook ini ditulis, Maret 2026
serde = { version = "1", features = ["derive"] }
```

Buat struct untuk claims, yaitu data yang bakal disimpan di dalam token:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // user id
    pub email: String,
    pub role: String,
    pub exp: usize,     // expiry timestamp (unix seconds)
}
```

---

## Generate Token

Buat fungsi untuk generate token pas user berhasil login. Tempatkan ini di `auth_service.rs` atau modul auth kamu.

```rust
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_token(user: &User, secret: &str) -> Result<String, AppError> {
    let expiry = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 7 * 24 * 3600; // 7 hari

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp: expiry,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}
```

Yang terjadi di sini: hitung waktu expired (sekarang + 7 hari dalam Unix timestamp), buat struct `Claims` dengan data user, lalu `encode` dari crate `jsonwebtoken` menghasilkan string JWT. `Header::default()` pakai algoritma `HS256` secara default, cukup untuk kebanyakan kasus.

---

## Verify Token

Saat user kirim request dengan token, kita perlu verifikasi token itu valid dan belum expired.

```rust
use jsonwebtoken::{decode, DecodingKey, Validation};

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Token tidak valid".to_string()))
}
```

`Validation::default()` sudah otomatis cek signature dan memastikan token belum expired (field `exp`). Kalau salah satu gagal, langsung return `AppError::Unauthorized`.

---

## Custom Extractor: AuthUser

Ini bagian paling elegan dari Axum. Daripada parse header Authorization secara manual di setiap handler, kita buat **custom extractor**, di mana Axum akan otomatis jalankan logika auth sebelum handler dieksekusi.

*Extractor* adalah tipe yang implement trait `FromRequestParts`. Axum memanggil fungsi ini otomatis ketika tipe tersebut ada di parameter handler.

[ILUSTRASI: diagram satpam (FromRequestParts) berdiri di depan pintu ruangan (handler) — semua tamu harus lewat satpam dulu sebelum masuk]

```rust
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Token diperlukan".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Format token salah".to_string()))?;

        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_default();
        let claims = verify_token(token, &jwt_secret)?;
        Ok(AuthUser(claims))
    }
}
```

Alur kerjanya: ambil header `Authorization` (kalau tidak ada, return 401), strip prefix `"Bearer "` (kalau format salah, return 401), verify token (kalau invalid/expired, return 401), lalu return `AuthUser(claims)` kalau semua OK.

---

## Pakai AuthUser di Handler

Tinggal tambahkan `AuthUser` sebagai parameter di handler manapun yang butuh autentikasi:

```rust
async fn get_my_profile(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> AppResult<impl IntoResponse> {
    let user = state.user_repo.find_by_id(claims.sub.parse()?).await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))?;
    Ok(ApiResponse::ok(user, "Berhasil"))
}
```

Axum otomatis panggil `AuthUser::from_request_parts` sebelum handler jalan. Kalau auth gagal, handler bahkan tidak dieksekusi; langsung balik error response. `AuthUser(claims)` adalah pattern destructuring, kita langsung ambil `Claims` dari dalam wrapper `AuthUser`.

---

## Update Login untuk Return JWT

Handler login sekarang perlu generate dan return token. Update handler login kamu:

```rust
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let user = state.user_repo.find_by_email(&payload.email).await?
        .ok_or_else(|| AppError::Unauthorized("Email atau password salah".to_string()))?;

    let is_valid = verify_password(&payload.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::Unauthorized("Email atau password salah".to_string()));
    }

    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal(anyhow::anyhow!("JWT_SECRET tidak dikonfigurasi")))?;

    let token = generate_token(&user, &jwt_secret)?;

    Ok(ApiResponse::ok(
        serde_json::json!({ "token": token }),
        "Login berhasil",
    ))
}
```

Pastikan `JWT_SECRET` ada di `.env` kamu:

```env
JWT_SECRET=rahasia-yang-panjang-dan-aman-minimal-32-karakter
```

Jangan pakai secret yang pendek atau mudah ditebak. Secret ini yang menjamin nobody bisa forge token palsu.

---

## Latihan

1. **Buat endpoint `/me`** yang return profil user yang sedang login menggunakan `AuthUser` extractor. Test dengan Postman: coba tanpa token, dengan token expired, dan dengan token valid.

2. **Role-based guard:** buat extractor `AdminUser` yang sama seperti `AuthUser` tapi tambahkan pengecekan `claims.role == "admin"`. Kalau bukan admin, return `AppError::Forbidden`. Terapkan di endpoint admin.

3. **Eksplorasi:** Coba decode JWT kamu di [jwt.io](https://jwt.io), paste token yang di-generate dan lihat isi payload-nya. Perhatikan bahwa payload *bisa dibaca* tanpa secret, jadi jangan simpan password atau data sensitif di JWT.

4. **Refresh token (opsional):** Riset perbedaan access token (expire pendek, misal 15 menit) vs refresh token (expire panjang, misal 30 hari). Kapan pola ini lebih baik dari token 7 hari yang kita pakai sekarang?
