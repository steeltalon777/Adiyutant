use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReminderDefinition {
    pub id: Id<Self>,
    pub title: String,
    pub message: Option<String>,
    pub schedule_rule: String,
    pub next_fire_at: Option<AdiyutantDateTime>,
    pub enabled: bool,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl ReminderDefinition {
    pub fn new(title: String, schedule_rule: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            title,
            message: None,
            schedule_rule,
            next_fire_at: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_reminder() {
        let r = ReminderDefinition::new("Drink water".into(), "every 2h".into());
        assert_eq!(r.title, "Drink water");
        assert_eq!(r.schedule_rule, "every 2h");
        assert!(r.enabled);
    }

    #[test]
    fn reminder_enabled_by_default() {
        let r = ReminderDefinition::new("Stretch".into(), "every 1h".into());
        assert!(r.enabled);
    }

    #[test]
    fn serde_round_trip() {
        let r = ReminderDefinition::new("Stand up".into(), "0 */30 * * * *".into());
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: ReminderDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(r.id, deserialized.id);
        assert_eq!(r.schedule_rule, deserialized.schedule_rule);
    }
}
