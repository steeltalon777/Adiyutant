use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckInType {
    Morning,
    Day,
    Evening,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckIn {
    pub id: Id<Self>,
    pub daily_log_id: Id<crate::model::daily_log::DailyLog>,
    pub check_in_type: CheckInType,
    pub raw_input: String,
    pub structured_data: Option<serde_json::Value>,
    pub agent_response: Option<String>,
    pub created_at: AdiyutantDateTime,
}

impl CheckIn {
    pub fn new(
        daily_log_id: Id<crate::model::daily_log::DailyLog>,
        check_in_type: CheckInType,
        raw_input: String,
    ) -> Self {
        Self {
            id: Id::new(),
            daily_log_id,
            check_in_type,
            raw_input,
            structured_data: None,
            agent_response: None,
            created_at: AdiyutantDateTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::daily_log::DailyLog;
    use chrono::NaiveDate;
    use serde_json::json;

    fn make_daily_log_id() -> Id<DailyLog> {
        let date = NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        DailyLog::new(date).id
    }

    #[test]
    fn new_creates_check_in() {
        let daily_log_id = make_daily_log_id();
        let ci = CheckIn::new(daily_log_id, CheckInType::Morning, "feeling great".into());
        assert_eq!(ci.check_in_type, CheckInType::Morning);
        assert_eq!(ci.raw_input, "feeling great");
        assert_eq!(ci.daily_log_id, daily_log_id);
    }

    #[test]
    fn check_in_type_serialize_snake_case() {
        assert_eq!(
            serde_json::to_value(CheckInType::Morning).unwrap(),
            json!("morning")
        );
        assert_eq!(
            serde_json::to_value(CheckInType::Shutdown).unwrap(),
            json!("shutdown")
        );
    }

    #[test]
    fn two_checkins_same_log_have_different_ids() {
        let daily_log_id = make_daily_log_id();
        let ci1 = CheckIn::new(daily_log_id, CheckInType::Day, "ok".into());
        let ci2 = CheckIn::new(daily_log_id, CheckInType::Day, "ok".into());
        assert_ne!(ci1.id, ci2.id);
    }

    #[test]
    fn serde_round_trip_with_structured_data() {
        let daily_log_id = make_daily_log_id();
        let mut ci = CheckIn::new(daily_log_id, CheckInType::Evening, "tired".into());
        ci.structured_data = Some(json!({"sleep_hours": 6}));
        let json = serde_json::to_string(&ci).unwrap();
        let deserialized: CheckIn = serde_json::from_str(&json).unwrap();
        assert_eq!(ci.id, deserialized.id);
        assert_eq!(
            deserialized.structured_data.unwrap(),
            json!({"sleep_hours": 6})
        );
    }
}
