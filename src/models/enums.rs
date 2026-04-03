use sqlx::Type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Customer,
}

#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "ticket_status", rename_all = "lowercase")]
pub enum TicketStatus {
    Open,
    #[serde(rename = "in_progress")]
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "ticket_priority", rename_all = "lowercase")]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "ticket_category", rename_all = "lowercase")]
pub enum TicketCategory {
    General,
    Billing,
    Technical,
    Other,
}
