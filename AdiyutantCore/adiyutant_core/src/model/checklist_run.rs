use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::checklist_template::ChecklistItem;
use crate::model::checklist_template::ChecklistTemplate;
use crate::model::daily_log::DailyLog;
use serde::{Deserialize, Serialize};

/// An instance of running a checklist template.
#[derive(Debug, Clone)]
pub struct ChecklistRun {
    pub id: Id<ChecklistRun>,
    pub template_id: Id<ChecklistTemplate>,
    pub daily_log_id: Id<DailyLog>,
    pub started_at: AdiyutantDateTime,
    pub completed_at: Option<AdiyutantDateTime>,
    pub answers: Vec<ChecklistAnswer>,
}

impl ChecklistRun {
    pub fn new(template_id: Id<ChecklistTemplate>, daily_log_id: Id<DailyLog>) -> Self {
        Self {
            id: Id::new(),
            template_id,
            daily_log_id,
            started_at: AdiyutantDateTime::now(),
            completed_at: None,
            answers: vec![],
        }
    }
}

/// A single answer within a checklist run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistAnswer {
    pub id: Id<ChecklistAnswer>,
    pub checklist_run_id: Id<ChecklistRun>,
    pub checklist_item_id: Id<ChecklistItem>,
    pub value: String,
    pub comment: Option<String>,
    pub answered_at: AdiyutantDateTime,
}

impl ChecklistAnswer {
    pub fn new(
        checklist_run_id: Id<ChecklistRun>,
        checklist_item_id: Id<ChecklistItem>,
        value: String,
    ) -> Self {
        Self {
            id: Id::new(),
            checklist_run_id,
            checklist_item_id,
            value,
            comment: None,
            answered_at: AdiyutantDateTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::checklist_template::ChecklistTemplate;
    use crate::model::daily_log::DailyLog;

    #[test]
    fn checklist_run_new_not_completed() {
        let template_id = Id::<ChecklistTemplate>::new();
        let log_id = Id::<DailyLog>::new();
        let run = ChecklistRun::new(template_id, log_id);
        assert!(run.completed_at.is_none());
        assert!(run.answers.is_empty());
    }

    #[test]
    fn checklist_answer_new() {
        let run_id = Id::<ChecklistRun>::new();
        let item_id = Id::<ChecklistItem>::new();
        let answer = ChecklistAnswer::new(run_id, item_id, "yes".into());
        assert_eq!(answer.value, "yes");
    }
}
