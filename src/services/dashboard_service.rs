use crate::{
    common::AppError,
    repositories::{DashboardRepository, DashboardStats},
};

/// Service untuk menangani dashboard statistics
#[derive(Clone)]
pub struct DashboardService {
    dashboard_repo: DashboardRepository,
}

impl DashboardService {
    pub fn new(dashboard_repo: DashboardRepository) -> Self {
        Self { dashboard_repo }
    }

    /// Ambil dashboard statistics
    pub async fn get_stats(&self) -> Result<DashboardStats, AppError> {
        self.dashboard_repo.get_stats().await
    }
}
