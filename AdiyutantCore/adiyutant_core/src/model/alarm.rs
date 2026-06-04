use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlarmDefinition {
    pub id: Id<Self>,
    pub title: String,
    pub time: chrono::NaiveTime,
    pub repeat_rule: Option<String>,
    pub enabled: bool,
    pub platform_binding_id: Option<String>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl AlarmDefinition {
    pub fn new(title: String, time: chrono::NaiveTime) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            title,
            time,
            repeat_rule: None,
            enabled: true,
            platform_binding_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_alarm() {
        let time = chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap();
        let a = AlarmDefinition::new("Wake up".into(), time);
        assert_eq!(a.title, "Wake up");
        assert_eq!(a.time, time);
        assert!(a.enabled);
    }

    #[test]
    fn alarm_enabled_by_default() {
        let time = chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let a = AlarmDefinition::new("Morning".into(), time);
        assert!(a.enabled);
    }

    #[test]
    fn serde_round_trip() {
        let time = chrono::NaiveTime::from_hms_opt(22, 30, 0).unwrap();
        let a = AlarmDefinition::new("Sleep".into(), time);
        let json = serde_json::to_string(&a).unwrap();
        let deserialized: AlarmDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(a.id, deserialized.id);
        assert_eq!(a.time, deserialized.time);
    }
}
