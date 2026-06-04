use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyLog {
    pub id: Id<Self>,
    pub date: chrono::NaiveDate,
    pub mode: Option<String>,
    pub sleep_score: Option<u8>,
    pub energy: Option<u8>,
    pub mood: Option<u8>,
    pub raw_notes: Option<String>,
    pub ai_summary: Option<String>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl DailyLog {
    pub fn new(date: chrono::NaiveDate) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            date,
            mode: None,
            sleep_score: None,
            energy: None,
            mood: None,
            raw_notes: None,
            ai_summary: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_with_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let log = DailyLog::new(date);
        assert_eq!(log.date, date);
    }

    #[test]
    fn all_optional_fields_default_to_none() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let log = DailyLog::new(date);
        assert!(log.mode.is_none());
        assert!(log.sleep_score.is_none());
        assert!(log.energy.is_none());
        assert!(log.mood.is_none());
        assert!(log.raw_notes.is_none());
        assert!(log.ai_summary.is_none());
    }

    #[test]
    fn two_logs_same_date_have_different_ids() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let log1 = DailyLog::new(date);
        let log2 = DailyLog::new(date);
        assert_ne!(log1.id, log2.id);
    }

    #[test]
    fn serde_round_trip() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let log = DailyLog::new(date);
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: DailyLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log.id, deserialized.id);
        assert_eq!(log.date, deserialized.date);
    }
}
