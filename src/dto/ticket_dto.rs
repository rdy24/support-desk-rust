use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketDto {
    #[validate(length(min = 5, max = 200, message = "Subject harus 5-200 karakter"))]
    pub subject: String,

    #[validate(length(min = 10, message = "Deskripsi minimal 10 karakter"))]
    pub description: String,

    #[validate(custom(function = "validate_category"))]
    pub category: String,

    #[validate(custom(function = "validate_priority"))]
    pub priority: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTicketDto {
    #[validate(length(min = 5, max = 200, message = "Subject harus 5-200 karakter"))]
    pub subject: Option<String>,

    #[validate(custom(function = "validate_status"))]
    pub status: Option<String>,
}

fn validate_category(category: &str) -> Result<(), validator::ValidationError> {
    match category {
        "general" | "billing" | "technical" | "other" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_category");
            err.message = Some("Category harus: general, billing, technical, atau other".into());
            Err(err)
        }
    }
}

fn validate_priority(priority: &str) -> Result<(), validator::ValidationError> {
    match priority {
        "low" | "medium" | "high" | "urgent" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_priority");
            err.message = Some("Priority harus: low, medium, high, atau urgent".into());
            Err(err)
        }
    }
}

fn validate_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "open" | "in_progress" | "resolved" | "closed" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_status");
            err.message = Some("Status harus: open, in_progress, resolved, atau closed".into());
            Err(err)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TicketFilters {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    #[serde(default)]
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_category_general() {
        let result = validate_category("general");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_category_billing() {
        let result = validate_category("billing");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_category_technical() {
        let result = validate_category("technical");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_category_other() {
        let result = validate_category("other");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_category_invalid() {
        let result = validate_category("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_priority_low() {
        let result = validate_priority("low");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_priority_medium() {
        let result = validate_priority("medium");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_priority_high() {
        let result = validate_priority("high");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_priority_urgent() {
        let result = validate_priority("urgent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_priority_invalid() {
        let result = validate_priority("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_status_open() {
        let result = validate_status("open");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_status_in_progress() {
        let result = validate_status("in_progress");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_status_resolved() {
        let result = validate_status("resolved");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_status_closed() {
        let result = validate_status("closed");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_status_invalid() {
        let result = validate_status("invalid");
        assert!(result.is_err());
    }
}
