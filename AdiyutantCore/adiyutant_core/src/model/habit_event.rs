use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitEventStatus {
    Done,
    Skipped,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitEventLevel {
    Min,
    Light,
    Base,
    Full,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HabitEvent {
    pub id: Id<Self>,
    pub habit_id: Id<crate::model::habit::Habit>,
    pub date_time: AdiyutantDateTime,
    pub status: HabitEventStatus,
    pub level: HabitEventLevel,
    pub comment: Option<String>,
}

impl HabitEvent {
    pub fn new(
        habit_id: Id<crate::model::habit::Habit>,
        status: HabitEventStatus,
        level: HabitEventLevel,
    ) -> Self {
        Self {
            id: Id::new(),
            habit_id,
            date_time: AdiyutantDateTime::now(),
            status,
            level,
            comment: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::habit::Habit;

    fn make_habit_id() -> Id<Habit> {
        Habit::new("read".into()).id
    }

    #[test]
    fn new_creates_habit_event() {
        let hid = make_habit_id();
        let event = HabitEvent::new(hid, HabitEventStatus::Done, HabitEventLevel::Base);
        assert_eq!(event.habit_id, hid);
        assert_eq!(event.status, HabitEventStatus::Done);
        assert_eq!(event.level, HabitEventLevel::Base);
    }

    #[test]
    fn status_serialize_snake_case() {
        use serde_json::json;
        assert_eq!(
            serde_json::to_value(HabitEventStatus::Done).unwrap(),
            json!("done")
        );
        assert_eq!(
            serde_json::to_value(HabitEventStatus::Skipped).unwrap(),
            json!("skipped")
        );
    }

    #[test]
    fn level_serialize_snake_case() {
        use serde_json::json;
        assert_eq!(
            serde_json::to_value(HabitEventLevel::Light).unwrap(),
            json!("light")
        );
        assert_eq!(
            serde_json::to_value(HabitEventLevel::Custom).unwrap(),
            json!("custom")
        );
    }

    #[test]
    fn serde_round_trip() {
        let hid = make_habit_id();
        let event = HabitEvent::new(hid, HabitEventStatus::Partial, HabitEventLevel::Min);
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: HabitEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.status, deserialized.status);
    }
}
