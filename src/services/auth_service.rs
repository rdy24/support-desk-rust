use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::{
    common::AppError,
    dto::{LoginDto, RegisterDto},
    models::User,
    models::UserRole,
    repositories::UserRepository,
};
use validator::Validate;

/// JWT Claims yang disimpan di dalam token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user id (UUID as string)
    pub email: String,
    pub role: String,     // "admin", "agent", "customer"
    pub exp: usize,       // expiry timestamp (unix seconds)
}

/// Service untuk menangani autentikasi (register, login, JWT)
#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, jwt_secret: String) -> Self {
        Self { user_repo, jwt_secret }
    }

    /// Generate JWT token untuk user
    pub fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 7 * 24 * 3600; // 7 hari

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: format_role(&user.role),
            exp: expiry,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Register: Daftar akun pengguna baru
    pub async fn register(&self, dto: RegisterDto) -> Result<User, AppError> {
        // Validasi input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Langkah 1: Cek apakah email sudah digunakan
        if self.user_repo.find_by_email(&dto.email).await?.is_some() {
            return Err(AppError::Conflict("Email sudah digunakan".to_string()));
        }

        // Langkah 2: Hash password
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Argon2::default()
            .hash_password(dto.password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        // Langkah 3: Parse role dari string ke enum
        let role = parse_role(&dto.role)?;

        // Langkah 4: Simpan user baru ke database
        let user = self
            .user_repo
            .create(&dto.name, &dto.email, &hashed, role)
            .await?;

        Ok(user)
    }

    /// Login: Masuk ke sistem
    pub async fn login(&self, dto: LoginDto) -> Result<String, AppError> {
        // Validasi input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Langkah 1: Cari user berdasarkan email
        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Email atau password salah".to_string()))?;

        // Langkah 2: Verifikasi password yang dimasukkan dengan hash yang tersimpan
        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Argon2::default()
            .verify_password(dto.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Email atau password salah".to_string()))?;

        // Langkah 3: Generate token JWT
        let token = self.generate_token(&user)?;
        Ok(token)
    }
}

/// Helper: Konversi string role ke enum UserRole
fn parse_role(role: &str) -> Result<UserRole, AppError> {
    match role {
        "customer" => Ok(UserRole::Customer),
        "agent" => Ok(UserRole::Agent),
        _ => Err(AppError::BadRequest(
            "Role harus 'customer' atau 'agent'".to_string(),
        )),
    }
}

/// Helper: Konversi enum UserRole ke string (lowercase)
fn format_role(role: &UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Agent => "agent".to_string(),
        UserRole::Customer => "customer".to_string(),
    }
}

/// Verify JWT token (pub function untuk digunakan di middleware)
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Token tidak valid atau expired".to_string()))
}
