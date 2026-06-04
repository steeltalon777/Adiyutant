use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanItemStatus {
    Pending,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanItem {
    pub description: String,
    pub status: PlanItemStatus,
    pub estimated_minutes: Option<u32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: Id<Self>,
    pub daily_log_id: Id<crate::model::daily_log::DailyLog>,
    pub title: String,
    pub items: Vec<PlanItem>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl Plan {
    pub fn new(daily_log_id: Id<crate::model::daily_log::DailyLog>, title: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            daily_log_id,
            title,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_item(&mut self, description: String) {
        self.items.push(PlanItem {
            description,
            status: PlanItemStatus::Pending,
            estimated_minutes: None,
            notes: None,
        });
        self.updated_at = AdiyutantDateTime::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::daily_log::DailyLog;

    fn make_daily_log_id() -> Id<DailyLog> {
        DailyLog::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 4).unwrap()).id
    }

    #[test]
    fn new_creates_empty_plan() {
        let dl_id = make_daily_log_id();
        let plan = Plan::new(dl_id, "My Day".into());
        assert_eq!(plan.title, "My Day");
        assert!(plan.items.is_empty());
    }

    #[test]
    fn add_item_creates_pending_item() {
        let dl_id = make_daily_log_id();
        let mut plan = Plan::new(dl_id, "Today".into());
        plan.add_item("Write code".into());
        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.items[0].description, "Write code");
        assert_eq!(plan.items[0].status, PlanItemStatus::Pending);
    }

    #[test]
    fn add_item_updates_updated_at() {
        let dl_id = make_daily_log_id();
        let mut plan = Plan::new(dl_id, "Plan".into());
        let original = plan.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(1));
        plan.add_item("Task".into());
        assert!(plan.updated_at > original);
    }

    #[test]
    fn serde_round_trip_with_item() {
        let dl_id = make_daily_log_id();
        let mut plan = Plan::new(dl_id, "Work".into());
        plan.add_item("Review PR".into());
        let json = serde_json::to_string(&plan).unwrap();
        let deserialized: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan.id.value(), deserialized.id.value());
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].description, "Review PR");
    }
}
