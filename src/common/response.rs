use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

// ============================================
// ApiResponse — Standar untuk semua response
// ============================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }
}

// Implement IntoResponse supaya bisa langsung di-return dari handler
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

// ============================================
// PaginatedResponse — Untuk list data dengan pagination
// ============================================

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: u32, limit: u32) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        Self {
            success: true,
            data,
            total,
            page,
            limit,
            total_pages,
        }
    }
}

// Implement IntoResponse untuk PaginatedResponse
impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

// ============================================
// AppError — Error handling yang otomatis jadi Response
// ============================================

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    BadRequest(String),
    ValidationError(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = ApiResponse::<()> {
            success: false,
            message,
            data: None,
        };

        (status, Json(body)).into_response()
    }
}

// ============================================
// AppResult — Type alias untuk Result<T, AppError>
// ============================================

pub type AppResult<T> = Result<T, AppError>;

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_total_zero() {
        let response = PaginatedResponse::new(Vec::<i32>::new(), 0, 1, 10);
        assert_eq!(response.total_pages, 0);
    }

    #[test]
    fn test_pagination_exact_fit() {
        let response = PaginatedResponse::new(vec![1, 2, 3], 10, 1, 10);
        assert_eq!(response.total_pages, 1);
    }

    #[test]
    fn test_pagination_multiple_pages() {
        let response = PaginatedResponse::new(vec![1], 11, 1, 10);
        assert_eq!(response.total_pages, 2);
    }
}
