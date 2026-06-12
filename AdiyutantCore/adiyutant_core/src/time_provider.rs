use chrono::{DateTime, NaiveDate, Timelike, Utc};

pub trait TimeProvider: Send + Sync {
    fn now_utc(&self) -> DateTime<Utc>;
    fn today(&self) -> NaiveDate;
    fn hour(&self) -> u32;
}

pub struct RealTimeProvider;

impl TimeProvider for RealTimeProvider {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn today(&self) -> NaiveDate {
        Utc::now().date_naive()
    }

    fn hour(&self) -> u32 {
        Utc::now().hour()
    }
}

pub struct FakeTimeProvider {
    fixed_time: DateTime<Utc>,
}

impl FakeTimeProvider {
    pub fn new(year: i32, month: u32, day: u32, hour: u32) -> Self {
        let date = NaiveDate::from_ymd_opt(year, month, day).expect("invalid date");
        let time = chrono::NaiveTime::from_hms_opt(hour, 0, 0).expect("invalid time");
        let naive = chrono::NaiveDateTime::new(date, time);
        Self {
            fixed_time: naive.and_utc(),
        }
    }
}

impl TimeProvider for FakeTimeProvider {
    fn now_utc(&self) -> DateTime<Utc> {
        self.fixed_time
    }

    fn today(&self) -> NaiveDate {
        self.fixed_time.date_naive()
    }

    fn hour(&self) -> u32 {
        self.fixed_time.hour()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn fake_time_provider_returns_fixed_values() {
        let tp = FakeTimeProvider::new(2026, 6, 10, 14);
        assert_eq!(tp.hour(), 14);
        assert_eq!(tp.today(), NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
        let now = tp.now_utc();
        assert_eq!(now.hour(), 14);
        assert_eq!(now.year(), 2026);
    }

    #[test]
    fn fake_time_provider_is_deterministic() {
        let tp = FakeTimeProvider::new(2026, 1, 1, 0);
        let t1 = tp.now_utc();
        let t2 = tp.now_utc();
        assert_eq!(t1, t2);
    }

    #[test]
    fn real_time_provider_returns_current_time() {
        let tp = RealTimeProvider;
        let before = Utc::now();
        let result = tp.now_utc();
        let after = Utc::now();
        assert!(result >= before);
        assert!(result <= after);
    }
}
