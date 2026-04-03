use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Ticket, TicketStatus, TicketPriority, TicketCategory};
use crate::dto::{CreateTicketDto, UpdateTicketDto};
use crate::common::AppError;

#[derive(Clone)]
pub struct TicketRepository {
    pool: PgPool,
}

impl TicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Cari ticket berdasarkan ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Ticket>, AppError> {
        #[derive(sqlx::FromRow)]
        struct TicketRow {
            id: Uuid,
            customer_id: Uuid,
            agent_id: Option<Uuid>,
            category: String,
            priority: String,
            status: String,
            subject: String,
            description: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, TicketRow>(
            "SELECT id, customer_id, agent_id, category::text, priority::text, status::text, subject, description, created_at, updated_at FROM tickets WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| Ticket {
            id: r.id,
            customer_id: r.customer_id,
            agent_id: r.agent_id,
            category: parse_category(&r.category).unwrap_or(TicketCategory::General),
            priority: parse_priority(&r.priority).unwrap_or(TicketPriority::Medium),
            status: parse_status(&r.status).unwrap_or(TicketStatus::Open),
            subject: r.subject,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Cari banyak tickets dengan filter dan pagination
    pub async fn find_many(
        &self,
        customer_id: Option<Uuid>,
        agent_id: Option<Uuid>,
        status: Option<&str>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Ticket>, i64), AppError> {
        #[derive(sqlx::FromRow)]
        struct TicketRow {
            id: Uuid,
            customer_id: Uuid,
            agent_id: Option<Uuid>,
            category: String,
            priority: String,
            status: String,
            subject: String,
            description: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let offset = (page - 1) * limit;

        let rows = sqlx::query_as::<_, TicketRow>(
            "SELECT id, customer_id, agent_id, category::text, priority::text, status::text, subject, description, created_at, updated_at FROM tickets
             WHERE ($1::uuid IS NULL OR customer_id = $1)
               AND ($2::uuid IS NULL OR agent_id = $2)
               AND ($3::text IS NULL OR status::text = $3)
             ORDER BY created_at DESC
             LIMIT $4 OFFSET $5"
        )
        .bind(customer_id)
        .bind(agent_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let tickets = rows.into_iter().map(|r| Ticket {
            id: r.id,
            customer_id: r.customer_id,
            agent_id: r.agent_id,
            category: parse_category(&r.category).unwrap_or(TicketCategory::General),
            priority: parse_priority(&r.priority).unwrap_or(TicketPriority::Medium),
            status: parse_status(&r.status).unwrap_or(TicketStatus::Open),
            subject: r.subject,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();

        let total: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tickets
             WHERE ($1::uuid IS NULL OR customer_id = $1)
               AND ($2::uuid IS NULL OR agent_id = $2)
               AND ($3::text IS NULL OR status::text = $3)"
        )
        .bind(customer_id)
        .bind(agent_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((tickets, total))
    }

    /// Buat ticket baru
    pub async fn create(
        &self,
        dto: &CreateTicketDto,
        customer_id: Uuid,
    ) -> Result<Ticket, AppError> {
        #[derive(sqlx::FromRow)]
        struct TicketRow {
            id: Uuid,
            customer_id: Uuid,
            agent_id: Option<Uuid>,
            category: String,
            priority: String,
            status: String,
            subject: String,
            description: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, TicketRow>(
            "INSERT INTO tickets (customer_id, category, priority, status, subject, description)
             VALUES ($1, $2::ticket_category, $3::ticket_priority, $4::ticket_status, $5, $6)
             RETURNING id, customer_id, agent_id, category::text, priority::text, status::text, subject, description, created_at, updated_at"
        )
        .bind(customer_id)
        .bind(&dto.category)
        .bind(&dto.priority)
        .bind("open")
        .bind(&dto.subject)
        .bind(&dto.description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(Ticket {
            id: row.id,
            customer_id: row.customer_id,
            agent_id: row.agent_id,
            category: parse_category(&row.category).unwrap_or(TicketCategory::General),
            priority: parse_priority(&row.priority).unwrap_or(TicketPriority::Medium),
            status: parse_status(&row.status).unwrap_or(TicketStatus::Open),
            subject: row.subject,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Update ticket (partial update)
    pub async fn update(
        &self,
        id: Uuid,
        dto: &UpdateTicketDto,
    ) -> Result<Option<Ticket>, AppError> {
        #[derive(sqlx::FromRow)]
        struct TicketRow {
            id: Uuid,
            customer_id: Uuid,
            agent_id: Option<Uuid>,
            category: String,
            priority: String,
            status: String,
            subject: String,
            description: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let status = dto.status.as_ref().map(|s| s.as_str());

        let row = sqlx::query_as::<_, TicketRow>(
            "UPDATE tickets SET status = COALESCE($2::ticket_status, status)
             WHERE id = $1
             RETURNING id, customer_id, agent_id, category::text, priority::text, status::text, subject, description, created_at, updated_at"
        )
        .bind(id)
        .bind(status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| Ticket {
            id: r.id,
            customer_id: r.customer_id,
            agent_id: r.agent_id,
            category: parse_category(&r.category).unwrap_or(TicketCategory::General),
            priority: parse_priority(&r.priority).unwrap_or(TicketPriority::Medium),
            status: parse_status(&r.status).unwrap_or(TicketStatus::Open),
            subject: r.subject,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Hapus ticket berdasarkan ID
    pub async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM tickets WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(result.is_some())
    }
}

/// Parse string to TicketCategory enum
fn parse_category(s: &str) -> Option<TicketCategory> {
    match s {
        "general" => Some(TicketCategory::General),
        "billing" => Some(TicketCategory::Billing),
        "technical" => Some(TicketCategory::Technical),
        "other" => Some(TicketCategory::Other),
        _ => None,
    }
}

/// Parse string to TicketPriority enum
fn parse_priority(s: &str) -> Option<TicketPriority> {
    match s {
        "low" => Some(TicketPriority::Low),
        "medium" => Some(TicketPriority::Medium),
        "high" => Some(TicketPriority::High),
        "urgent" => Some(TicketPriority::Urgent),
        _ => None,
    }
}

/// Parse string to TicketStatus enum
fn parse_status(s: &str) -> Option<TicketStatus> {
    match s {
        "open" => Some(TicketStatus::Open),
        "in_progress" => Some(TicketStatus::InProgress),
        "resolved" => Some(TicketStatus::Resolved),
        "closed" => Some(TicketStatus::Closed),
        _ => None,
    }
}
