use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::daily_log::DailyLog;

/// A daily plan composed of plan items for a given day.
///
/// Assembled by querying `plan_items` grouped by `daily_log_id`.
/// This is a read-model, not a separate stored entity.
#[derive(Debug, Clone)]
pub struct DayPlan {
    pub id: Id<DayPlan>,
    pub date: chrono::NaiveDate,
    pub daily_log_id: Id<DailyLog>,
    pub items: Vec<PlanItem>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

/// A single task within a day plan.
#[derive(Debug, Clone)]
pub struct PlanItem {
    pub id: Id<PlanItem>,
    pub daily_log_id: Id<DailyLog>,
    pub title: String,
    pub description: Option<String>,
    pub quadrant: EisenhowerQuadrant,
    pub planned_start: Option<chrono::NaiveTime>,
    pub planned_end: Option<chrono::NaiveTime>,
    pub status: PlanItemStatus,
    pub priority: u8,
    pub source: PlanningMode,
    pub waiting_review_at: Option<chrono::NaiveDate>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl PlanItem {
    pub fn new(daily_log_id: Id<DailyLog>, title: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            daily_log_id,
            title,
            description: None,
            quadrant: EisenhowerQuadrant::ImportantNotUrgent,
            planned_start: None,
            planned_end: None,
            status: PlanItemStatus::Planned,
            priority: 5,
            source: PlanningMode::Manual,
            waiting_review_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Eisenhower Matrix quadrant for task prioritization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EisenhowerQuadrant {
    ImportantUrgent,
    ImportantNotUrgent,
    NotImportantUrgent,
    NotImportantNotUrgent,
}

impl EisenhowerQuadrant {
    pub fn as_str(&self) -> &'static str {
        match self {
            EisenhowerQuadrant::ImportantUrgent => "important-urgent",
            EisenhowerQuadrant::ImportantNotUrgent => "important-not-urgent",
            EisenhowerQuadrant::NotImportantUrgent => "not-important-urgent",
            EisenhowerQuadrant::NotImportantNotUrgent => "not-important-not-urgent",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "important-urgent" => Some(EisenhowerQuadrant::ImportantUrgent),
            "important-not-urgent" | "important_not_urgent" => {
                Some(EisenhowerQuadrant::ImportantNotUrgent)
            }
            "not-important-urgent" | "not_important_urgent" => {
                Some(EisenhowerQuadrant::NotImportantUrgent)
            }
            "not-important-not-urgent" | "not_important_not_urgent" => {
                Some(EisenhowerQuadrant::NotImportantNotUrgent)
            }
            _ => None,
        }
    }
}

/// Current execution status of a plan item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanItemStatus {
    Planned,
    Started,
    Done,
    Skipped,
    Moved,
    Waiting,
    Archived,
    Cancelled,
}

impl PlanItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanItemStatus::Planned => "Planned",
            PlanItemStatus::Started => "Started",
            PlanItemStatus::Done => "Done",
            PlanItemStatus::Skipped => "Skipped",
            PlanItemStatus::Moved => "Moved",
            PlanItemStatus::Waiting => "Waiting",
            PlanItemStatus::Archived => "Archived",
            PlanItemStatus::Cancelled => "Cancelled",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Planned" | "planned" => Some(PlanItemStatus::Planned),
            "Started" | "started" => Some(PlanItemStatus::Started),
            "Done" | "done" => Some(PlanItemStatus::Done),
            "Skipped" | "skipped" => Some(PlanItemStatus::Skipped),
            "Moved" | "moved" => Some(PlanItemStatus::Moved),
            "Waiting" | "waiting" => Some(PlanItemStatus::Waiting),
            "Archived" | "archived" => Some(PlanItemStatus::Archived),
            "Cancelled" | "cancelled" => Some(PlanItemStatus::Cancelled),
            _ => None,
        }
    }
}

/// How a plan item was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningMode {
    Manual,
    AiGenerated,
    Template,
    CarryOver,
    LocalRule,
}

impl PlanningMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanningMode::Manual => "Manual",
            PlanningMode::AiGenerated => "AiGenerated",
            PlanningMode::Template => "Template",
            PlanningMode::CarryOver => "CarryOver",
            PlanningMode::LocalRule => "LocalRule",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Manual" | "manual" => Some(PlanningMode::Manual),
            "AiGenerated" | "ai_generated" => Some(PlanningMode::AiGenerated),
            "Template" | "template" => Some(PlanningMode::Template),
            "CarryOver" | "carry_over" => Some(PlanningMode::CarryOver),
            "LocalRule" | "local_rule" => Some(PlanningMode::LocalRule),
            _ => None,
        }
    }
}

/// Decision for reviewing a waiting task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitingDecision {
    Keep,
    Resume,
    Delete,
    Archive,
}

impl WaitingDecision {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "keep" | "Keep" => Some(WaitingDecision::Keep),
            "resume" | "start" | "Resume" | "Start" => Some(WaitingDecision::Resume),
            "delete" | "Delete" => Some(WaitingDecision::Delete),
            "archive" | "Archive" => Some(WaitingDecision::Archive),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_item_new_creates_planned() {
        let _date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let log_id = Id::<DailyLog>::new();
        let item = PlanItem::new(log_id, "Test task".into());
        assert_eq!(item.title, "Test task");
        assert_eq!(item.status, PlanItemStatus::Planned);
        assert_eq!(item.priority, 5);
        assert_eq!(item.source, PlanningMode::Manual);
    }

    #[test]
    fn eisenhower_quadrant_round_trip() {
        for q in &[
            EisenhowerQuadrant::ImportantUrgent,
            EisenhowerQuadrant::ImportantNotUrgent,
            EisenhowerQuadrant::NotImportantUrgent,
            EisenhowerQuadrant::NotImportantNotUrgent,
        ] {
            let s = q.as_str();
            let back = EisenhowerQuadrant::from_str(s).unwrap();
            assert_eq!(*q, back);
        }
    }

    #[test]
    fn plan_item_status_round_trip() {
        for s in &[
            PlanItemStatus::Planned,
            PlanItemStatus::Started,
            PlanItemStatus::Done,
            PlanItemStatus::Skipped,
            PlanItemStatus::Moved,
            PlanItemStatus::Waiting,
            PlanItemStatus::Archived,
            PlanItemStatus::Cancelled,
        ] {
            let str = s.as_str();
            let back = PlanItemStatus::from_str(str).unwrap();
            assert_eq!(*s, back);
        }
    }

    #[test]
    fn waiting_decision_from_str() {
        assert_eq!(
            WaitingDecision::from_str("keep"),
            Some(WaitingDecision::Keep)
        );
        assert_eq!(
            WaitingDecision::from_str("Resume"),
            Some(WaitingDecision::Resume)
        );
        assert_eq!(
            WaitingDecision::from_str("delete"),
            Some(WaitingDecision::Delete)
        );
        assert!(WaitingDecision::from_str("invalid").is_none());
    }
}
