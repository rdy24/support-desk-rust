use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::{
    common::AppError,
    dto::{LoginDto, RegisterDto},
    models::User,
    models::UserRole,
    repositories::UserRepository,
};
use validator::Validate;

/// Service untuk menangani autentikasi (register, login)
#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
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

        // Langkah 3: Return token (placeholder untuk sekarang, JWT di Bab 28)
        Ok("token_placeholder".to_string())
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
