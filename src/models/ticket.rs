use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use super::{TicketStatus, TicketPriority, TicketCategory};

// ============================================
// STRUCT: Ticket (model utama, untuk database)
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: Uuid,
    pub customer_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Uuid>,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub subject: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// DTO: CreateTicketDto (untuk input dari client)
// ============================================
// DTO hanya berisi field yang BOLEH dikirim client.
// Field seperti id, createdAt, status tidak boleh diisi client.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketDto {
    pub subject: String,
    pub description: String,
    pub category: String,
    pub priority: String,
}

// ============================================
// Dari latihan Bab 20
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TicketResponse {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketResponseDto {
    pub message: String,
}
