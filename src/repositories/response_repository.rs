use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    models::TicketResponse,
    common::AppError,
};

#[derive(Clone)]
pub struct ResponseRepository {
    pool: PgPool,
}

impl ResponseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_ticket(&self, ticket_id: Uuid) -> Result<Vec<TicketResponse>, AppError> {
        let responses = sqlx::query_as::<_, TicketResponse>(
            "SELECT id, ticket_id, user_id, message, created_at FROM ticket_responses WHERE ticket_id = $1 ORDER BY created_at DESC"
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(responses)
    }

    pub async fn create(
        &self,
        ticket_id: Uuid,
        user_id: Uuid,
        message: &str,
    ) -> Result<TicketResponse, AppError> {
        let response = sqlx::query_as::<_, TicketResponse>(
            "INSERT INTO ticket_responses (ticket_id, user_id, message) VALUES ($1, $2, $3) RETURNING id, ticket_id, user_id, message, created_at"
        )
        .bind(ticket_id)
        .bind(user_id)
        .bind(message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(response)
    }
}
