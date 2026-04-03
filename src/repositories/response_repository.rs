use sqlx::PgPool;
use uuid::Uuid;
use crate::models::TicketResponse;
use crate::common::AppError;

/// Repository untuk mengelola ticket responses (balasan tiket)
#[derive(Clone)]
pub struct ResponseRepository {
    pool: PgPool,
}

impl ResponseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Simpan balasan baru untuk tiket
    pub async fn create(
        &self,
        ticket_id: Uuid,
        user_id: Uuid,
        message: String,
    ) -> Result<TicketResponse, AppError> {
        #[derive(sqlx::FromRow)]
        struct ResponseRow {
            id: Uuid,
            ticket_id: Uuid,
            user_id: Uuid,
            message: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, ResponseRow>(
            r#"INSERT INTO ticket_responses (ticket_id, user_id, message)
               VALUES ($1, $2, $3)
               RETURNING id, ticket_id, user_id, message, created_at"#
        )
        .bind(ticket_id)
        .bind(user_id)
        .bind(&message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(TicketResponse {
            id: row.id,
            ticket_id: row.ticket_id,
            user_id: row.user_id,
            message: row.message,
            created_at: row.created_at,
        })
    }

    /// Ambil semua balasan untuk satu tiket, urut dari paling lama
    pub async fn find_by_ticket_id(
        &self,
        ticket_id: Uuid,
    ) -> Result<Vec<TicketResponse>, AppError> {
        #[derive(sqlx::FromRow)]
        struct ResponseRow {
            id: Uuid,
            ticket_id: Uuid,
            user_id: Uuid,
            message: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let rows = sqlx::query_as::<_, ResponseRow>(
            r#"SELECT id, ticket_id, user_id, message, created_at
               FROM ticket_responses
               WHERE ticket_id = $1
               ORDER BY created_at ASC"#
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| TicketResponse {
            id: r.id,
            ticket_id: r.ticket_id,
            user_id: r.user_id,
            message: r.message,
            created_at: r.created_at,
        }).collect())
    }

    /// (Latihan #1) Ambil balasan terbaru untuk satu tiket
    pub async fn find_latest_by_ticket_id(
        &self,
        ticket_id: Uuid,
    ) -> Result<Option<TicketResponse>, AppError> {
        #[derive(sqlx::FromRow)]
        struct ResponseRow {
            id: Uuid,
            ticket_id: Uuid,
            user_id: Uuid,
            message: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, ResponseRow>(
            r#"SELECT id, ticket_id, user_id, message, created_at
               FROM ticket_responses
               WHERE ticket_id = $1
               ORDER BY created_at DESC
               LIMIT 1"#
        )
        .bind(ticket_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| TicketResponse {
            id: r.id,
            ticket_id: r.ticket_id,
            user_id: r.user_id,
            message: r.message,
            created_at: r.created_at,
        }))
    }
}
