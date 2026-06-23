use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

/// A questionnaire / checklist template that can be run multiple times.
#[derive(Debug, Clone)]
pub struct ChecklistTemplate {
    pub id: Id<ChecklistTemplate>,
    pub title: String,
    /// Stable human/AI-readable identifier used by portable bundle import.
    /// Optional to keep backward compatibility with pre-8.4A rows.
    pub slug: Option<String>,
    pub category: String, // morning, day, evening, shutdown, recovery
    pub items: Vec<ChecklistItem>,
    pub version: u32,
    pub is_active: bool,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl ChecklistTemplate {
    pub fn new(title: String, category: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            title,
            slug: None,
            category,
            items: vec![],
            version: 1,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A single item / question within a checklist template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Id<ChecklistItem>,
    pub template_id: Id<ChecklistTemplate>,
    pub question: String,
    pub kind: ChecklistItemKind,
    pub options: Vec<String>,
    pub order: u32,
}

impl ChecklistItem {
    pub fn new(
        template_id: Id<ChecklistTemplate>,
        question: String,
        kind: ChecklistItemKind,
        order: u32,
    ) -> Self {
        Self {
            id: Id::new(),
            template_id,
            question,
            kind,
            options: vec![],
            order,
        }
    }
}

/// The type of input expected for a checklist item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecklistItemKind {
    Checkbox,
    Choice,
    Scale,
    Text,
    OptionalComment,
    HabitEvent,
    TaskLink,
    TimerStart,
}

impl ChecklistItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChecklistItemKind::Checkbox => "Checkbox",
            ChecklistItemKind::Choice => "Choice",
            ChecklistItemKind::Scale => "Scale",
            ChecklistItemKind::Text => "Text",
            ChecklistItemKind::OptionalComment => "OptionalComment",
            ChecklistItemKind::HabitEvent => "HabitEvent",
            ChecklistItemKind::TaskLink => "TaskLink",
            ChecklistItemKind::TimerStart => "TimerStart",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Checkbox" => Some(ChecklistItemKind::Checkbox),
            "Choice" => Some(ChecklistItemKind::Choice),
            "Scale" => Some(ChecklistItemKind::Scale),
            "Text" => Some(ChecklistItemKind::Text),
            "OptionalComment" => Some(ChecklistItemKind::OptionalComment),
            "HabitEvent" => Some(ChecklistItemKind::HabitEvent),
            "TaskLink" => Some(ChecklistItemKind::TaskLink),
            "TimerStart" => Some(ChecklistItemKind::TimerStart),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checklist_template_new_active() {
        let t = ChecklistTemplate::new("Morning".into(), "morning".into());
        assert!(t.is_active);
        assert_eq!(t.version, 1);
        assert!(t.items.is_empty());
    }

    #[test]
    fn checklist_item_kind_round_trip() {
        for k in &[
            ChecklistItemKind::Checkbox,
            ChecklistItemKind::Choice,
            ChecklistItemKind::Scale,
            ChecklistItemKind::Text,
            ChecklistItemKind::OptionalComment,
            ChecklistItemKind::HabitEvent,
            ChecklistItemKind::TaskLink,
            ChecklistItemKind::TimerStart,
        ] {
            let s = k.as_str();
            let back = ChecklistItemKind::from_str(s).unwrap();
            assert_eq!(*k, back);
        }
    }
}
