use adiyutant_core::error::CoreError;

pub trait Store {
    type Error;

    fn health_check(&self) -> Result<(), Self::Error>;
}

pub struct NoopStore;

impl Store for NoopStore {
    type Error = CoreError;

    fn health_check(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_store_health_check_ok() {
        let store = NoopStore;
        assert!(store.health_check().is_ok());
    }
}
