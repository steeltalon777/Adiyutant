use crate::dto::CoreErrorDto;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("storage error: {0}")]
    Storage(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    pub fn to_dto(&self) -> CoreErrorDto {
        match self {
            CoreError::InvalidInput(msg) => CoreErrorDto {
                code: "INVALID_INPUT".to_string(),
                message: format!("invalid input: {msg}"),
                recoverable: true,
                suggested_action: "Check input".to_string(),
            },
            CoreError::NotFound(msg) => CoreErrorDto {
                code: "NOT_FOUND".to_string(),
                message: format!("not found: {msg}"),
                recoverable: true,
                suggested_action: "Check item exists".to_string(),
            },
            CoreError::Internal(msg) => CoreErrorDto {
                code: "INTERNAL_ERROR".to_string(),
                message: format!("internal error: {msg}"),
                recoverable: false,
                suggested_action: "Contact support".to_string(),
            },
            CoreError::Storage(msg) => CoreErrorDto {
                code: "STORAGE_ERROR".to_string(),
                message: format!("storage error: {msg}"),
                recoverable: false,
                suggested_action: "Retry or check disk".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_result_ok() {
        let result: CoreResult<i32> = Ok(42);
        assert!(result.is_ok());
        #[allow(clippy::unnecessary_literal_unwrap)]
        {
            assert_eq!(result.unwrap(), 42);
        }
    }

    #[test]
    fn core_result_err() {
        let result: CoreResult<i32> = Err(CoreError::InvalidInput("bad data".into()));
        assert!(result.is_err());
    }

    #[test]
    fn core_error_display_invalid_input() {
        let err = CoreError::InvalidInput("bad".into());
        assert_eq!(format!("{}", err), "invalid input: bad");
    }

    #[test]
    fn core_error_display_not_found() {
        let err = CoreError::NotFound("missing".into());
        assert_eq!(format!("{}", err), "not found: missing");
    }

    #[test]
    fn core_error_display_internal() {
        let err = CoreError::Internal("oops".into());
        assert_eq!(format!("{}", err), "internal error: oops");
    }

    #[test]
    fn core_error_to_dto_invalid_input() {
        let err = CoreError::InvalidInput("bad".into());
        let dto = err.to_dto();
        assert_eq!(dto.code, "INVALID_INPUT");
        assert!(dto.recoverable);
        assert_eq!(dto.suggested_action, "Check input");
    }

    #[test]
    fn core_error_to_dto_not_found() {
        let err = CoreError::NotFound("missing".into());
        let dto = err.to_dto();
        assert_eq!(dto.code, "NOT_FOUND");
        assert!(dto.recoverable);
        assert_eq!(dto.suggested_action, "Check item exists");
    }

    #[test]
    fn core_error_to_dto_internal() {
        let err = CoreError::Internal("oops".into());
        let dto = err.to_dto();
        assert_eq!(dto.code, "INTERNAL_ERROR");
        assert!(!dto.recoverable);
        assert_eq!(dto.suggested_action, "Contact support");
    }

    #[test]
    fn core_error_to_dto_storage() {
        let err = CoreError::Storage("disk full".into());
        let dto = err.to_dto();
        assert_eq!(dto.code, "STORAGE_ERROR");
        assert!(!dto.recoverable);
        assert_eq!(dto.suggested_action, "Retry or check disk");
    }
}
