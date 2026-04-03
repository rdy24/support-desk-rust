use uuid::Uuid;
use crate::{
    common::AppError,
    models::{Ticket, TicketResponse, CreateTicketResponseDto},
    repositories::{TicketRepository, ResponseRepository},
    dto::{CreateTicketDto, UpdateTicketDto, TicketFilters},
    services::Claims,
};

/// Service untuk menangani bisnis logic tiket
#[derive(Clone)]
pub struct TicketService {
    ticket_repo: TicketRepository,
    response_repo: ResponseRepository,
}

impl TicketService {
    pub fn new(ticket_repo: TicketRepository, response_repo: ResponseRepository) -> Self {
        Self { ticket_repo, response_repo }
    }

    /// Buat tiket baru (hanya customer)
    pub async fn create(
        &self,
        dto: CreateTicketDto,
        claims: &Claims,
    ) -> Result<Ticket, AppError> {
        // Hanya customer yang boleh buat tiket
        if claims.role != "customer" {
            return Err(AppError::Forbidden(
                "Hanya customer yang bisa membuat ticket".to_string(),
            ));
        }

        // Ambil customer_id dari JWT (lebih aman daripada dari request body)
        let customer_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Internal("Invalid user id".to_string()))?;

        self.ticket_repo.create(&dto, customer_id).await
    }

    /// Ambil tiket berdasarkan ID dengan cek akses
    pub async fn get_by_id(
        &self,
        ticket_id: Uuid,
        claims: &Claims,
    ) -> Result<Ticket, AppError> {
        let ticket = self
            .ticket_repo
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

        self.check_access(&ticket, claims)?;
        Ok(ticket)
    }

    /// Ambil list tiket dengan filtering berbasis role
    pub async fn get_many(
        &self,
        filters: TicketFilters,
        claims: &Claims,
    ) -> Result<(Vec<Ticket>, i64), AppError> {
        let user_id = Uuid::parse_str(&claims.sub).ok();

        // Tentukan filter berdasarkan role
        let (customer_filter, agent_filter) = match claims.role.as_str() {
            "customer" => (user_id, None),
            "agent" => (None, None),
            "admin" => (None, None),
            _ => return Err(AppError::Forbidden("Role tidak valid".to_string())),
        };

        self.ticket_repo
            .find_many(
                customer_filter,
                agent_filter,
                filters.status.as_deref(),
                filters.page.unwrap_or(1) as i64,
                filters.limit.unwrap_or(10) as i64,
            )
            .await
    }

    /// Update tiket (hanya agent/admin)
    pub async fn update(
        &self,
        ticket_id: Uuid,
        dto: UpdateTicketDto,
        claims: &Claims,
    ) -> Result<Ticket, AppError> {
        // Customer tidak boleh update tiket
        if claims.role == "customer" {
            return Err(AppError::Forbidden(
                "Customer tidak bisa mengubah ticket".to_string(),
            ));
        }

        let updated = self
            .ticket_repo
            .update(ticket_id, &dto)
            .await?
            .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

        Ok(updated)
    }

    /// Hapus tiket (selalu forbidden)
    pub async fn delete(&self, _ticket_id: Uuid, _claims: &Claims) -> Result<(), AppError> {
        Err(AppError::Forbidden(
            "Ticket tidak bisa dihapus".to_string(),
        ))
    }

    /// Tambah response ke tiket (dengan cek akses)
    pub async fn add_response(
        &self,
        ticket_id: Uuid,
        dto: CreateTicketResponseDto,
        claims: &Claims,
    ) -> Result<TicketResponse, AppError> {
        // Cek apakah ticket ada dan user punya akses
        let ticket = self
            .ticket_repo
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

        self.check_access(&ticket, claims)?;

        // Ambil user_id dari JWT
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Internal("Invalid user id".to_string()))?;

        self.response_repo
            .create(ticket_id, user_id, dto.message)
            .await
    }

    /// Ambil semua response untuk satu ticket (dengan cek akses)
    pub async fn get_responses(
        &self,
        ticket_id: Uuid,
        claims: &Claims,
    ) -> Result<Vec<TicketResponse>, AppError> {
        // Cek apakah ticket ada dan user punya akses
        let ticket = self
            .ticket_repo
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ticket tidak ditemukan".to_string()))?;

        self.check_access(&ticket, claims)?;

        self.response_repo.find_by_ticket_id(ticket_id).await
    }

    /// Private method: cek apakah user punya akses ke ticket
    fn check_access(&self, ticket: &Ticket, claims: &Claims) -> Result<(), AppError> {
        match claims.role.as_str() {
            "admin" => Ok(()),
            "agent" => Ok(()),
            "customer" => {
                if ticket.customer_id.to_string() == claims.sub {
                    Ok(())
                } else {
                    Err(AppError::Forbidden(
                        "Kamu tidak punya akses ke ticket ini".to_string(),
                    ))
                }
            }
            _ => Err(AppError::Forbidden("Role tidak dikenal".to_string())),
        }
    }
}
