use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Habit {
    pub id: Id<Self>,
    pub name: String,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub target: Option<String>,
    pub is_active: bool,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl Habit {
    pub fn new(name: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            name,
            description: None,
            frequency: None,
            target: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_habit() {
        let habit = Habit::new("read".into());
        assert_eq!(habit.name, "read");
        assert!(habit.is_active);
    }

    #[test]
    fn habit_active_by_default() {
        let habit = Habit::new("exercise".into());
        assert!(habit.is_active);
    }

    #[test]
    fn serde_round_trip() {
        let habit = Habit::new("meditate".into());
        let json = serde_json::to_string(&habit).unwrap();
        let deserialized: Habit = serde_json::from_str(&json).unwrap();
        assert_eq!(habit.id, deserialized.id);
        assert_eq!(habit.name, deserialized.name);
    }

    #[test]
    fn two_habits_same_name_have_different_ids() {
        let h1 = Habit::new("run".into());
        let h2 = Habit::new("run".into());
        assert_ne!(h1.id, h2.id);
    }
}
