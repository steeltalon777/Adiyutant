use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_result_ok() {
        let result: CoreResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
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
}
