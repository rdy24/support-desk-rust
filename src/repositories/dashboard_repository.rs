use sqlx::PgPool;
use serde::Serialize;
use crate::common::AppError;

/// Statistik dashboard untuk aplikasi
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_tickets: i64,
    pub open_tickets: i64,
    pub in_progress_tickets: i64,
    pub resolved_tickets: i64,
    pub closed_tickets: i64,
    pub total_users: i64,
    pub total_agents: i64,
    pub total_customers: i64,
    pub avg_responses_per_ticket: f64,
}

/// Repository untuk mengambil statistik dashboard
/// Dashboard berfokus pada aggregate queries (COUNT, SUM, AVG)
#[derive(Clone)]
pub struct DashboardRepository {
    pool: PgPool,
}

impl DashboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Ambil statistik lengkap untuk dashboard
    pub async fn get_stats(&self) -> Result<DashboardStats, AppError> {
        // --- Statistik Tiket ---
        #[derive(sqlx::FromRow)]
        struct TicketStatsRow {
            total_tickets: i64,
            open_tickets: i64,
            in_progress_tickets: i64,
            resolved_tickets: i64,
            closed_tickets: i64,
        }

        let ticket_stats = sqlx::query_as::<_, TicketStatsRow>(
            r#"SELECT
                COUNT(*) AS total_tickets,
                COUNT(*) FILTER (WHERE status = 'open') AS open_tickets,
                COUNT(*) FILTER (WHERE status = 'in_progress') AS in_progress_tickets,
                COUNT(*) FILTER (WHERE status = 'resolved') AS resolved_tickets,
                COUNT(*) FILTER (WHERE status = 'closed') AS closed_tickets
            FROM tickets"#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // --- Statistik User ---
        #[derive(sqlx::FromRow)]
        struct UserStatsRow {
            total_users: i64,
            total_agents: i64,
            total_customers: i64,
        }

        let user_stats = sqlx::query_as::<_, UserStatsRow>(
            r#"SELECT
                COUNT(*) AS total_users,
                COUNT(*) FILTER (WHERE role = 'agent') AS total_agents,
                COUNT(*) FILTER (WHERE role = 'customer') AS total_customers
            FROM users"#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // --- (Latihan #2) Rata-rata responses per tiket ---
        let total_responses: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ticket_responses"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let avg_responses_per_ticket = if ticket_stats.total_tickets > 0 {
            total_responses as f64 / ticket_stats.total_tickets as f64
        } else {
            0.0
        };

        Ok(DashboardStats {
            total_tickets: ticket_stats.total_tickets,
            open_tickets: ticket_stats.open_tickets,
            in_progress_tickets: ticket_stats.in_progress_tickets,
            resolved_tickets: ticket_stats.resolved_tickets,
            closed_tickets: ticket_stats.closed_tickets,
            total_users: user_stats.total_users,
            total_agents: user_stats.total_agents,
            total_customers: user_stats.total_customers,
            avg_responses_per_ticket,
        })
    }
}
