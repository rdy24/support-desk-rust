use uuid::Uuid;
use crate::{
    common::AppError,
    models::User,
    repositories::UserRepository,
    services::Claims,
    dto::UpdateUserDto,
};

/// Service untuk menangani bisnis logic user
#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    /// Ambil profil user sendiri dari JWT claims
    pub async fn get_me(&self, claims: &Claims) -> Result<User, AppError> {
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Internal("Invalid user id".to_string()))?;

        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
    }

    /// Ambil semua user dengan optional role filter
    pub async fn get_all(
        &self,
        role: Option<&str>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<User>, i64), AppError> {
        self.user_repo.find_all(role, page, limit).await
    }

    /// Ambil user berdasarkan ID
    pub async fn get_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
    }

    /// Update user
    pub async fn update(&self, user_id: Uuid, dto: UpdateUserDto) -> Result<User, AppError> {
        // Cek apakah ada field yang diupdate
        if dto.name.is_none() && dto.role.is_none() {
            return Err(AppError::BadRequest(
                "Tidak ada field yang diupdate".to_string(),
            ));
        }

        self.user_repo
            .update(user_id, dto.name.as_deref(), dto.role.as_deref())
            .await?
            .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))
    }

    /// Hapus user (dengan cek self-delete)
    pub async fn delete(&self, target_id: Uuid, claims: &Claims) -> Result<(), AppError> {
        // Tidak boleh menghapus diri sendiri
        if target_id.to_string() == claims.sub {
            return Err(AppError::Forbidden(
                "Tidak bisa menghapus akun sendiri".to_string(),
            ));
        }

        self.user_repo.delete(target_id).await?;
        Ok(())
    }
}
