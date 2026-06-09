use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::daily_log::DailyLog;

/// A timestamped entry in the daily journal / activity log.
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id: Id<JournalEntry>,
    pub daily_log_id: Id<DailyLog>,
    pub timestamp: AdiyutantDateTime,
    pub entry_type: JournalEntryType,
    pub summary: String,
}

impl JournalEntry {
    pub fn new(daily_log_id: Id<DailyLog>, entry_type: JournalEntryType, summary: String) -> Self {
        Self {
            id: Id::new(),
            daily_log_id,
            timestamp: AdiyutantDateTime::now(),
            entry_type,
            summary,
        }
    }
}

/// What kind of event triggered this journal entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalEntryType {
    CheckInCreated,
    ChecklistCompleted,
    TaskStarted,
    TaskDone,
    TaskMoved,
    NudgeAnswered,
    Note,
    Shutdown,
}

impl JournalEntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JournalEntryType::CheckInCreated => "CheckInCreated",
            JournalEntryType::ChecklistCompleted => "ChecklistCompleted",
            JournalEntryType::TaskStarted => "TaskStarted",
            JournalEntryType::TaskDone => "TaskDone",
            JournalEntryType::TaskMoved => "TaskMoved",
            JournalEntryType::NudgeAnswered => "NudgeAnswered",
            JournalEntryType::Note => "Note",
            JournalEntryType::Shutdown => "Shutdown",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CheckInCreated" => Some(JournalEntryType::CheckInCreated),
            "ChecklistCompleted" => Some(JournalEntryType::ChecklistCompleted),
            "TaskStarted" => Some(JournalEntryType::TaskStarted),
            "TaskDone" => Some(JournalEntryType::TaskDone),
            "TaskMoved" => Some(JournalEntryType::TaskMoved),
            "NudgeAnswered" => Some(JournalEntryType::NudgeAnswered),
            "Note" => Some(JournalEntryType::Note),
            "Shutdown" => Some(JournalEntryType::Shutdown),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_entry_new_creates() {
        let log_id = Id::<DailyLog>::new();
        let entry = JournalEntry::new(log_id, JournalEntryType::Note, "test".into());
        assert_eq!(entry.summary, "test");
        assert_eq!(entry.entry_type, JournalEntryType::Note);
    }

    #[test]
    fn journal_entry_type_round_trip() {
        for t in &[
            JournalEntryType::CheckInCreated,
            JournalEntryType::ChecklistCompleted,
            JournalEntryType::TaskStarted,
            JournalEntryType::TaskDone,
            JournalEntryType::TaskMoved,
            JournalEntryType::NudgeAnswered,
            JournalEntryType::Note,
            JournalEntryType::Shutdown,
        ] {
            let s = t.as_str();
            let back = JournalEntryType::from_str(s).unwrap();
            assert_eq!(*t, back);
        }
    }
}
