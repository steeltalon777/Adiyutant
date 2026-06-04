use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Focus,
    Rest,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimerDefinition {
    pub id: Id<Self>,
    pub title: String,
    pub duration_seconds: u64,
    pub mode: TimerMode,
    pub created_at: AdiyutantDateTime,
}

impl TimerDefinition {
    pub fn new(title: String, duration_seconds: u64, mode: TimerMode) -> Self {
        Self {
            id: Id::new(),
            title,
            duration_seconds,
            mode,
            created_at: AdiyutantDateTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_timer() {
        let t = TimerDefinition::new("Focus".into(), 1500, TimerMode::Focus);
        assert_eq!(t.title, "Focus");
        assert_eq!(t.duration_seconds, 1500);
        assert_eq!(t.mode, TimerMode::Focus);
    }

    #[test]
    fn mode_serialize_snake_case() {
        use serde_json::json;
        assert_eq!(
            serde_json::to_value(TimerMode::Focus).unwrap(),
            json!("focus")
        );
        assert_eq!(
            serde_json::to_value(TimerMode::Rest).unwrap(),
            json!("rest")
        );
    }

    #[test]
    fn serde_round_trip() {
        let t = TimerDefinition::new("Pomodoro".into(), 1500, TimerMode::Focus);
        let json = serde_json::to_string(&t).unwrap();
        let deserialized: TimerDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(t.id, deserialized.id);
        assert_eq!(t.duration_seconds, deserialized.duration_seconds);
    }
}
