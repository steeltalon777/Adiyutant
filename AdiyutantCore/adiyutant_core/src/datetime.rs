use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AdiyutantDateTime(DateTime<Utc>);

impl AdiyutantDateTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_utc(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    pub fn inner(&self) -> DateTime<Utc> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_returns_non_zero() {
        let dt = AdiyutantDateTime::now();
        // Unix timestamp should be > 0 (year 1970+)
        assert!(dt.inner().timestamp() > 0);
    }

    #[test]
    fn inner_returns_chrono_datetime() {
        let dt = AdiyutantDateTime::now();
        let inner = dt.inner();
        // It should be a valid DateTime<Utc>
        assert_eq!(inner.timezone(), Utc);
    }

    #[test]
    fn two_now_values_are_monotonic() {
        let dt1 = AdiyutantDateTime::now();
        let dt2 = AdiyutantDateTime::now();
        // dt2 should be >= dt1 (or very close)
        assert!(dt2 >= dt1);
    }
}
