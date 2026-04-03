use sqlx::Type;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_status", rename_all = "lowercase")]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_priority", rename_all = "lowercase")]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Type)]
#[sqlx(type_name = "ticket_category", rename_all = "lowercase")]
pub enum TicketCategory {
    General,
    Billing,
    Technical,
    Other,
}
