use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, UserRole};
use crate::common::AppError;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Cari user berdasarkan ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: Uuid,
            name: String,
            email: String,
            password: String,
            role: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, name, email, password, role::text, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| User {
            id: r.id,
            name: r.name,
            email: r.email,
            password: r.password,
            role: parse_role(&r.role).unwrap_or(UserRole::Customer),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Cari user berdasarkan email (dipakai saat login)
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: Uuid,
            name: String,
            email: String,
            password: String,
            role: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, name, email, password, role::text, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| User {
            id: r.id,
            name: r.name,
            email: r.email,
            password: r.password,
            role: parse_role(&r.role).unwrap_or(UserRole::Customer),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Buat user baru
    pub async fn create(
        &self,
        name: &str,
        email: &str,
        password: &str,
        role: UserRole,
    ) -> Result<User, AppError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: Uuid,
            name: String,
            email: String,
            password: String,
            role: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let role_str = format_role(role);

        let row = sqlx::query_as::<_, UserRow>(
            "INSERT INTO users (name, email, password, role) VALUES ($1, $2, $3, $4::user_role) RETURNING id, name, email, password, role::text, created_at, updated_at"
        )
        .bind(name)
        .bind(email)
        .bind(password)
        .bind(role_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(User {
            id: row.id,
            name: row.name,
            email: row.email,
            password: row.password,
            role: parse_role(&row.role).unwrap_or(UserRole::Customer),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Dapatkan semua users dengan pagination dan optional role filter
    pub async fn find_all(
        &self,
        role: Option<&str>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<User>, i64), AppError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: Uuid,
            name: String,
            email: String,
            password: String,
            role: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let offset = (page - 1) * limit;

        // Build query dengan optional role filter
        let query_str = if role.is_some() {
            "SELECT id, name, email, password, role::text, created_at, updated_at FROM users WHERE role::text = $3 ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        } else {
            "SELECT id, name, email, password, role::text, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        };

        let mut query = sqlx::query_as::<_, UserRow>(query_str)
            .bind(limit)
            .bind(offset);

        if let Some(r) = role {
            query = query.bind(r);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let users = rows.into_iter().map(|r| User {
            id: r.id,
            name: r.name,
            email: r.email,
            password: r.password,
            role: parse_role(&r.role).unwrap_or(UserRole::Customer),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();

        // Count total dengan optional role filter
        let count_query_str = if role.is_some() {
            "SELECT COUNT(*) FROM users WHERE role::text = $1"
        } else {
            "SELECT COUNT(*) FROM users"
        };

        let mut count_query = sqlx::query_scalar::<_, i64>(count_query_str);

        if let Some(r) = role {
            count_query = count_query.bind(r);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((users, total))
    }

    /// Update user
    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        role: Option<&str>,
    ) -> Result<Option<User>, AppError> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: Uuid,
            name: String,
            email: String,
            password: String,
            role: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();
        let mut param_count = 2; // Start at 2 because $1 is id

        if name.is_some() {
            updates.push(format!("name = ${}", param_count));
            bindings.push(name.unwrap().to_string());
            param_count += 1;
        }

        if role.is_some() {
            updates.push(format!("role = ${}::user_role", param_count));
            bindings.push(role.unwrap().to_string());
        }

        // Jika tidak ada field yang diupdate, return existing user
        if updates.is_empty() {
            return self.find_by_id(id).await;
        }

        updates.push("updated_at = NOW()".to_string());

        let query_str = format!(
            "UPDATE users SET {} WHERE id = $1 RETURNING id, name, email, password, role::text, created_at, updated_at",
            updates.join(", ")
        );

        let mut query = sqlx::query_as::<_, UserRow>(&query_str).bind(id);

        for binding in bindings {
            query = query.bind(binding);
        }

        let row = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| User {
            id: r.id,
            name: r.name,
            email: r.email,
            password: r.password,
            role: parse_role(&r.role).unwrap_or(UserRole::Customer),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Hapus user berdasarkan ID
    pub async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(result.is_some())
    }
}

/// Convert UserRole enum to PostgreSQL enum string
fn format_role(role: UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Agent => "agent".to_string(),
        UserRole::Customer => "customer".to_string(),
    }
}

/// Parse string to UserRole enum
fn parse_role(s: &str) -> Option<UserRole> {
    match s {
        "admin" => Some(UserRole::Admin),
        "agent" => Some(UserRole::Agent),
        "customer" => Some(UserRole::Customer),
        _ => None,
    }
}
