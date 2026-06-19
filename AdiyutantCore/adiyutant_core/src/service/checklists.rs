use super::*;
use crate::datetime::AdiyutantDateTime;
use crate::dto::{ChecklistRunDto, ChecklistTemplateDto};
use crate::model::checklist_run::{ChecklistAnswer, ChecklistRun};
use crate::model::checklist_template::{ChecklistItem, ChecklistItemKind, ChecklistTemplate};
use crate::model::journal_entry::{JournalEntry, JournalEntryType};

impl AdiyutantCoreService {
    /// List all available checklist templates.
    pub fn list_checklist_templates(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<ChecklistTemplateDto>, crate::error::CoreError> {
        let templates = if let Some(cat) = category {
            self.store.list_checklist_templates_by_category(cat)?
        } else {
            self.store.list_checklist_templates()?
        };
        Ok(templates
            .iter()
            .map(|t| ChecklistTemplateDto {
                id: t.id.value().to_string(),
                title: t.title.clone(),
                category: t.category.clone(),
                item_count: t.items.len(),
                is_active: t.is_active,
            })
            .collect())
    }

    /// Auto-seed a default checklist template for a category.
    fn seed_default_checklist(
        &self,
        category: &str,
    ) -> Result<ChecklistTemplate, crate::error::CoreError> {
        let (title, items) = match category {
            "morning" => (
                "Morning Check-in",
                vec![
                    ("Did you sleep well?", ChecklistItemKind::Scale),
                    ("What is your energy level?", ChecklistItemKind::Scale),
                    ("What is your mood?", ChecklistItemKind::Scale),
                    ("Any notes for today?", ChecklistItemKind::Text),
                ],
            ),
            "evening" => (
                "Evening Review",
                vec![
                    ("How was your day?", ChecklistItemKind::Text),
                    ("What went well?", ChecklistItemKind::Text),
                    ("What could improve?", ChecklistItemKind::Text),
                ],
            ),
            "shutdown" => (
                "Shutdown Routine",
                vec![
                    ("Review completed tasks", ChecklistItemKind::Checkbox),
                    ("Plan for tomorrow", ChecklistItemKind::Checkbox),
                    ("Set alarms", ChecklistItemKind::Checkbox),
                    ("Wind down time", ChecklistItemKind::Checkbox),
                ],
            ),
            "day" => (
                "Mid-day Check",
                vec![
                    ("How is your energy?", ChecklistItemKind::Scale),
                    ("Are you on track?", ChecklistItemKind::Choice),
                    ("Need to adjust plan?", ChecklistItemKind::Checkbox),
                ],
            ),
            "recovery" => (
                "Recovery Check",
                vec![
                    ("How are you feeling?", ChecklistItemKind::Scale),
                    ("Rest enough?", ChecklistItemKind::Checkbox),
                    ("Ready to resume?", ChecklistItemKind::Checkbox),
                ],
            ),
            _ => {
                return Err(crate::error::CoreError::NotFound(format!(
                    "unknown category: {category}"
                )));
            }
        };

        let mut template = ChecklistTemplate::new(title.to_string(), category.to_string());
        for (i, (question, kind)) in items.iter().enumerate() {
            template.items.push(ChecklistItem::new(
                template.id,
                question.to_string(),
                *kind,
                i as u32 + 1,
            ));
        }

        self.store.insert_checklist_template(&template)?;
        Ok(template)
    }

    /// Run a checklist (start a run for the first matching template by category).
    pub fn run_checklist(
        &self,
        category: &str,
    ) -> Result<ChecklistRunDto, crate::error::CoreError> {
        let templates = self.store.list_checklist_templates_by_category(category)?;
        let template = if let Some(t) = templates.first() {
            t.clone()
        } else {
            self.seed_default_checklist(category)?
        };

        let daily_log_id = self.get_or_create_today_log()?;
        let run = ChecklistRun::new(template.id, daily_log_id);
        self.store.insert_checklist_run(&run)?;

        Ok(ChecklistRunDto {
            id: run.id.value().to_string(),
            template_title: template.title.clone(),
            started_at: run.started_at.inner().to_rfc3339(),
            completed_at: String::new(),
            answer_count: 0,
        })
    }

    /// Answer a single checklist item in a run.
    pub fn answer_checklist_item(
        &self,
        run_id: &str,
        item_id: &str,
        value: &str,
        comment: Option<&str>,
    ) -> Result<ChecklistRunDto, crate::error::CoreError> {
        let run_uuid = crate::service::helpers::parse_id::<ChecklistRun>(run_id)?;
        let mut run = self
            .store
            .get_checklist_run(run_uuid)?
            .ok_or_else(|| crate::error::CoreError::NotFound(format!("checklist_run {run_id}")))?;

        if run.completed_at.is_some() {
            return Err(crate::error::CoreError::InvalidInput(
                "checklist run already completed".into(),
            ));
        }

        let item_uuid = crate::service::helpers::parse_id::<ChecklistItem>(item_id)?;
        let mut answer = ChecklistAnswer::new(run.id, item_uuid, value.to_string());
        answer.comment = comment.map(String::from);
        answer.answered_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        run.answers.push(answer);
        self.store.update_checklist_run(&run)?;

        Ok(ChecklistRunDto {
            id: run.id.value().to_string(),
            template_title: String::new(),
            started_at: run.started_at.inner().to_rfc3339(),
            completed_at: run
                .completed_at
                .map(|dt| dt.inner().to_rfc3339())
                .unwrap_or_default(),
            answer_count: run.answers.len(),
        })
    }

    /// Complete a checklist run (transactional: update_run + journal entry).
    pub fn complete_checklist_run(
        &self,
        run_id: &str,
    ) -> Result<ChecklistRunDto, crate::error::CoreError> {
        let uuid = crate::service::helpers::parse_id::<ChecklistRun>(run_id)?;
        let mut run = self
            .store
            .get_checklist_run(uuid)?
            .ok_or_else(|| crate::error::CoreError::NotFound(format!("checklist_run {run_id}")))?;

        if run.completed_at.is_some() {
            return Err(crate::error::CoreError::InvalidInput(
                "checklist already completed".into(),
            ));
        }

        run.completed_at = Some(AdiyutantDateTime::from_utc(self.time.now_utc()));

        let entry = JournalEntry::new(
            run.daily_log_id,
            JournalEntryType::ChecklistCompleted,
            format!("Checklist completed: {}", run.id.value()),
        );

        // Atomic: update_checklist_run + insert_journal_entry
        self.store.complete_checklist_composite(&run, &entry)?;

        let template_title = self
            .store
            .get_checklist_template(run.template_id)
            .ok()
            .and_then(|t| t)
            .map(|t| t.title)
            .unwrap_or_default();

        Ok(ChecklistRunDto {
            id: run.id.value().to_string(),
            template_title,
            started_at: run.started_at.inner().to_rfc3339(),
            completed_at: run
                .completed_at
                .map(|dt| dt.inner().to_rfc3339())
                .unwrap_or_default(),
            answer_count: run.answers.len(),
        })
    }
}
