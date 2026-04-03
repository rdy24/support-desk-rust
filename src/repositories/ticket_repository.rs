use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    models::{Ticket, TicketCategory, TicketPriority, TicketStatus},
    common::AppError,
};

#[derive(Clone)]
pub struct TicketRepository {
    pool: PgPool,
}

impl TicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Ticket>, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            "SELECT id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at FROM tickets WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ticket)
    }

    pub async fn find_many(
        &self,
        limit: i64,
        offset: i64,
        category: Option<TicketCategory>,
    ) -> Result<Vec<Ticket>, AppError> {
        let tickets = sqlx::query_as::<_, Ticket>(
            "SELECT id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at FROM tickets
             WHERE ($1::ticket_category IS NULL OR category = $1)
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(category)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(tickets)
    }

    pub async fn create(
        &self,
        customer_id: Uuid,
        category: TicketCategory,
        priority: TicketPriority,
        subject: &str,
        description: &str,
    ) -> Result<Ticket, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            "INSERT INTO tickets (customer_id, category, priority, subject, description) VALUES ($1, $2, $3, $4, $5) RETURNING id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at"
        )
        .bind(customer_id)
        .bind(category)
        .bind(priority)
        .bind(subject)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ticket)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: TicketStatus,
    ) -> Result<Ticket, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            "UPDATE tickets SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING id, customer_id, agent_id, category, priority, status, subject, description, created_at, updated_at"
        )
        .bind(status)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ticket)
    }
}
