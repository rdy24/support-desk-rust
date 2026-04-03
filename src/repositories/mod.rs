pub mod user_repository;
pub mod ticket_repository;
pub mod response_repository;
pub mod dashboard_repository;

pub use user_repository::UserRepository;
pub use ticket_repository::TicketRepository;
pub use response_repository::ResponseRepository;
pub use dashboard_repository::{DashboardRepository, DashboardStats};
