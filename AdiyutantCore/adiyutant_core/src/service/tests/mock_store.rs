use crate::error::CoreError;
use crate::id::Id;
use crate::model::action_proposal::ActionProposal;
use crate::model::action_proposal::ProposalStatus;
use crate::model::alarm::AlarmDefinition;
use crate::model::check_in::CheckIn;

use crate::model::checklist_run::ChecklistRun;
use crate::model::checklist_template::ChecklistTemplate;
use crate::model::context_document::ContextDocument;
use crate::model::daily_log::DailyLog;
use crate::model::day_plan::{PlanItem, PlanItemStatus};
use crate::model::habit::Habit;
use crate::model::habit_event::HabitEvent;
use crate::model::import_run::ImportRun;
use crate::model::journal_entry::JournalEntry;
use crate::model::plan::Plan;
use crate::model::reminder::ReminderDefinition;
use crate::model::task_checkpoint::TaskCheckpoint;
use crate::model::timer::TimerDefinition;
use crate::store::Store;
use chrono::NaiveDate;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct MockStore {
    daily_logs: RefCell<HashMap<String, DailyLog>>,
    check_ins: RefCell<HashMap<String, CheckIn>>,
    habits: RefCell<HashMap<String, Habit>>,
    habit_events: RefCell<HashMap<String, HabitEvent>>,
    plans: RefCell<HashMap<String, Plan>>,
    plan_items: RefCell<HashMap<String, PlanItem>>,
    checkpoints: RefCell<HashMap<String, TaskCheckpoint>>,
    checklist_templates: RefCell<HashMap<String, ChecklistTemplate>>,
    checklist_runs: RefCell<HashMap<String, ChecklistRun>>,
    journal_entries: RefCell<HashMap<String, JournalEntry>>,
    timers: RefCell<HashMap<String, TimerDefinition>>,
    reminders: RefCell<HashMap<String, ReminderDefinition>>,
    alarms: RefCell<HashMap<String, AlarmDefinition>>,
    context_docs: RefCell<HashMap<String, ContextDocument>>,
    action_proposals: RefCell<HashMap<String, ActionProposal>>,
    import_runs: RefCell<HashMap<String, ImportRun>>,
}

impl MockStore {
    pub fn new() -> Self {
        Self {
            daily_logs: RefCell::new(HashMap::new()),
            check_ins: RefCell::new(HashMap::new()),
            habits: RefCell::new(HashMap::new()),
            habit_events: RefCell::new(HashMap::new()),
            plans: RefCell::new(HashMap::new()),
            plan_items: RefCell::new(HashMap::new()),
            checkpoints: RefCell::new(HashMap::new()),
            checklist_templates: RefCell::new(HashMap::new()),
            checklist_runs: RefCell::new(HashMap::new()),
            journal_entries: RefCell::new(HashMap::new()),
            timers: RefCell::new(HashMap::new()),
            reminders: RefCell::new(HashMap::new()),
            alarms: RefCell::new(HashMap::new()),
            context_docs: RefCell::new(HashMap::new()),
            action_proposals: RefCell::new(HashMap::new()),
            import_runs: RefCell::new(HashMap::new()),
        }
    }
}

impl Store for MockStore {
    type Error = CoreError;

    fn migrate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn health_check(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error> {
        self.daily_logs
            .borrow_mut()
            .insert(log.id.value().to_string(), log.clone());
        Ok(())
    }
    fn get_daily_log(&self, id: Id<DailyLog>) -> Result<Option<DailyLog>, Self::Error> {
        Ok(self
            .daily_logs
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn get_daily_log_by_date(&self, date: NaiveDate) -> Result<Option<DailyLog>, Self::Error> {
        Ok(self
            .daily_logs
            .borrow()
            .values()
            .find(|l| l.date == date)
            .cloned())
    }
    fn list_daily_logs(&self) -> Result<Vec<DailyLog>, Self::Error> {
        Ok(self.daily_logs.borrow().values().cloned().collect())
    }
    fn update_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error> {
        self.daily_logs
            .borrow_mut()
            .insert(log.id.value().to_string(), log.clone());
        Ok(())
    }
    fn delete_daily_log(&self, id: Id<DailyLog>) -> Result<(), Self::Error> {
        self.daily_logs.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }

    fn insert_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error> {
        self.check_ins
            .borrow_mut()
            .insert(ci.id.value().to_string(), ci.clone());
        Ok(())
    }
    fn get_check_in(&self, id: Id<CheckIn>) -> Result<Option<CheckIn>, Self::Error> {
        Ok(self
            .check_ins
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_check_ins_by_log(&self, log_id: Id<DailyLog>) -> Result<Vec<CheckIn>, Self::Error> {
        Ok(self
            .check_ins
            .borrow()
            .values()
            .filter(|ci| ci.daily_log_id == log_id)
            .cloned()
            .collect())
    }
    fn update_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error> {
        self.check_ins
            .borrow_mut()
            .insert(ci.id.value().to_string(), ci.clone());
        Ok(())
    }
    fn delete_check_in(&self, id: Id<CheckIn>) -> Result<(), Self::Error> {
        self.check_ins.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }

    fn insert_habit(&self, h: &Habit) -> Result<(), Self::Error> {
        self.habits
            .borrow_mut()
            .insert(h.id.value().to_string(), h.clone());
        Ok(())
    }
    fn get_habit(&self, id: Id<Habit>) -> Result<Option<Habit>, Self::Error> {
        Ok(self.habits.borrow().get(&id.value().to_string()).cloned())
    }
    fn list_habits(&self) -> Result<Vec<Habit>, Self::Error> {
        Ok(self.habits.borrow().values().cloned().collect())
    }
    fn update_habit(&self, h: &Habit) -> Result<(), Self::Error> {
        self.habits
            .borrow_mut()
            .insert(h.id.value().to_string(), h.clone());
        Ok(())
    }
    fn delete_habit(&self, id: Id<Habit>) -> Result<(), Self::Error> {
        self.habits.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }

    fn insert_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error> {
        self.habit_events
            .borrow_mut()
            .insert(he.id.value().to_string(), he.clone());
        Ok(())
    }
    fn get_habit_event(&self, id: Id<HabitEvent>) -> Result<Option<HabitEvent>, Self::Error> {
        Ok(self
            .habit_events
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_habit_events_by_habit(
        &self,
        habit_id: Id<Habit>,
    ) -> Result<Vec<HabitEvent>, Self::Error> {
        Ok(self
            .habit_events
            .borrow()
            .values()
            .filter(|e| e.habit_id == habit_id)
            .cloned()
            .collect())
    }
    fn update_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error> {
        self.habit_events
            .borrow_mut()
            .insert(he.id.value().to_string(), he.clone());
        Ok(())
    }
    fn delete_habit_event(&self, id: Id<HabitEvent>) -> Result<(), Self::Error> {
        self.habit_events
            .borrow_mut()
            .remove(&id.value().to_string());
        Ok(())
    }

    fn insert_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error> {
        self.reminders
            .borrow_mut()
            .insert(r.id.value().to_string(), r.clone());
        Ok(())
    }
    fn get_reminder(
        &self,
        id: Id<ReminderDefinition>,
    ) -> Result<Option<ReminderDefinition>, Self::Error> {
        Ok(self
            .reminders
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_reminders(&self) -> Result<Vec<ReminderDefinition>, Self::Error> {
        Ok(self.reminders.borrow().values().cloned().collect())
    }
    fn update_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error> {
        self.reminders
            .borrow_mut()
            .insert(r.id.value().to_string(), r.clone());
        Ok(())
    }
    fn delete_reminder(&self, id: Id<ReminderDefinition>) -> Result<(), Self::Error> {
        self.reminders.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }

    fn insert_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error> {
        self.alarms
            .borrow_mut()
            .insert(a.id.value().to_string(), a.clone());
        Ok(())
    }
    fn get_alarm(&self, id: Id<AlarmDefinition>) -> Result<Option<AlarmDefinition>, Self::Error> {
        Ok(self.alarms.borrow().get(&id.value().to_string()).cloned())
    }
    fn list_alarms(&self) -> Result<Vec<AlarmDefinition>, Self::Error> {
        Ok(self.alarms.borrow().values().cloned().collect())
    }
    fn update_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error> {
        self.alarms
            .borrow_mut()
            .insert(a.id.value().to_string(), a.clone());
        Ok(())
    }
    fn delete_alarm(&self, id: Id<AlarmDefinition>) -> Result<(), Self::Error> {
        self.alarms.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }

    fn insert_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error> {
        self.timers
            .borrow_mut()
            .insert(t.id.value().to_string(), t.clone());
        Ok(())
    }
    fn get_timer(&self, id: Id<TimerDefinition>) -> Result<Option<TimerDefinition>, Self::Error> {
        Ok(self.timers.borrow().get(&id.value().to_string()).cloned())
    }
    fn list_timers(&self) -> Result<Vec<TimerDefinition>, Self::Error> {
        Ok(self.timers.borrow().values().cloned().collect())
    }
    fn update_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error> {
        self.timers
            .borrow_mut()
            .insert(t.id.value().to_string(), t.clone());
        Ok(())
    }
    fn delete_timer(&self, id: Id<TimerDefinition>) -> Result<(), Self::Error> {
        self.timers.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }

    fn insert_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error> {
        self.context_docs
            .borrow_mut()
            .insert(cd.id.value().to_string(), cd.clone());
        Ok(())
    }
    fn get_context_document(
        &self,
        id: Id<ContextDocument>,
    ) -> Result<Option<ContextDocument>, Self::Error> {
        Ok(self
            .context_docs
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_context_documents(&self) -> Result<Vec<ContextDocument>, Self::Error> {
        Ok(self.context_docs.borrow().values().cloned().collect())
    }
    fn update_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error> {
        self.context_docs
            .borrow_mut()
            .insert(cd.id.value().to_string(), cd.clone());
        Ok(())
    }
    fn delete_context_document(&self, id: Id<ContextDocument>) -> Result<(), Self::Error> {
        self.context_docs
            .borrow_mut()
            .remove(&id.value().to_string());
        Ok(())
    }

    fn insert_plan(&self, p: &Plan) -> Result<(), Self::Error> {
        self.plans
            .borrow_mut()
            .insert(p.id.value().to_string(), p.clone());
        Ok(())
    }
    fn get_plan(&self, id: Id<Plan>) -> Result<Option<Plan>, Self::Error> {
        Ok(self.plans.borrow().get(&id.value().to_string()).cloned())
    }
    fn get_plan_by_daily_log(&self, log_id: Id<DailyLog>) -> Result<Option<Plan>, Self::Error> {
        Ok(self
            .plans
            .borrow()
            .values()
            .find(|p| p.daily_log_id == log_id)
            .cloned())
    }
    fn update_plan(&self, p: &Plan) -> Result<(), Self::Error> {
        self.plans
            .borrow_mut()
            .insert(p.id.value().to_string(), p.clone());
        Ok(())
    }
    fn delete_plan(&self, id: Id<Plan>) -> Result<(), Self::Error> {
        self.plans.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }

    fn insert_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error> {
        self.action_proposals
            .borrow_mut()
            .insert(ap.id.value().to_string(), ap.clone());
        Ok(())
    }
    fn get_action_proposal(
        &self,
        id: Id<ActionProposal>,
    ) -> Result<Option<ActionProposal>, Self::Error> {
        Ok(self
            .action_proposals
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_action_proposals(&self) -> Result<Vec<ActionProposal>, Self::Error> {
        Ok(self.action_proposals.borrow().values().cloned().collect())
    }
    fn list_action_proposals_by_status(
        &self,
        _status: ProposalStatus,
    ) -> Result<Vec<ActionProposal>, Self::Error> {
        Ok(vec![])
    }
    fn update_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error> {
        self.action_proposals
            .borrow_mut()
            .insert(ap.id.value().to_string(), ap.clone());
        Ok(())
    }
    fn delete_action_proposal(&self, id: Id<ActionProposal>) -> Result<(), Self::Error> {
        self.action_proposals
            .borrow_mut()
            .remove(&id.value().to_string());
        Ok(())
    }

    fn insert_plan_item(&self, item: &PlanItem) -> Result<(), Self::Error> {
        self.plan_items
            .borrow_mut()
            .insert(item.id.value().to_string(), item.clone());
        Ok(())
    }
    fn get_plan_item(&self, id: Id<PlanItem>) -> Result<Option<PlanItem>, Self::Error> {
        Ok(self
            .plan_items
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_plan_items_by_log(&self, log_id: Id<DailyLog>) -> Result<Vec<PlanItem>, Self::Error> {
        Ok(self
            .plan_items
            .borrow()
            .values()
            .filter(|i| i.daily_log_id == log_id)
            .cloned()
            .collect())
    }
    fn update_plan_item(&self, item: &PlanItem) -> Result<(), Self::Error> {
        self.plan_items
            .borrow_mut()
            .insert(item.id.value().to_string(), item.clone());
        Ok(())
    }
    fn delete_plan_item(&self, id: Id<PlanItem>) -> Result<(), Self::Error> {
        self.plan_items.borrow_mut().remove(&id.value().to_string());
        Ok(())
    }
    fn list_plan_items_by_status(
        &self,
        _status: PlanItemStatus,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        Ok(self.plan_items.borrow().values().cloned().collect())
    }
    fn list_plan_items_due_for_review(
        &self,
        _date: NaiveDate,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        Ok(self.plan_items.borrow().values().cloned().collect())
    }

    fn insert_task_checkpoint(&self, cp: &TaskCheckpoint) -> Result<(), Self::Error> {
        self.checkpoints
            .borrow_mut()
            .insert(cp.id.value().to_string(), cp.clone());
        Ok(())
    }
    fn get_task_checkpoint(
        &self,
        id: Id<TaskCheckpoint>,
    ) -> Result<Option<TaskCheckpoint>, Self::Error> {
        Ok(self
            .checkpoints
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_checkpoints_by_plan_item(
        &self,
        item_id: Id<PlanItem>,
    ) -> Result<Vec<TaskCheckpoint>, Self::Error> {
        Ok(self
            .checkpoints
            .borrow()
            .values()
            .filter(|cp| cp.plan_item_id == item_id)
            .cloned()
            .collect())
    }
    fn update_task_checkpoint(&self, cp: &TaskCheckpoint) -> Result<(), Self::Error> {
        self.checkpoints
            .borrow_mut()
            .insert(cp.id.value().to_string(), cp.clone());
        Ok(())
    }
    fn delete_task_checkpoint(&self, id: Id<TaskCheckpoint>) -> Result<(), Self::Error> {
        self.checkpoints
            .borrow_mut()
            .remove(&id.value().to_string());
        Ok(())
    }

    fn insert_checklist_template(&self, t: &ChecklistTemplate) -> Result<(), Self::Error> {
        self.checklist_templates
            .borrow_mut()
            .insert(t.id.value().to_string(), t.clone());
        Ok(())
    }
    fn get_checklist_template(
        &self,
        id: Id<ChecklistTemplate>,
    ) -> Result<Option<ChecklistTemplate>, Self::Error> {
        Ok(self
            .checklist_templates
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_checklist_templates_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<ChecklistTemplate>, Self::Error> {
        Ok(self
            .checklist_templates
            .borrow()
            .values()
            .filter(|t| t.category == category)
            .cloned()
            .collect())
    }
    fn list_checklist_templates(&self) -> Result<Vec<ChecklistTemplate>, Self::Error> {
        Ok(self
            .checklist_templates
            .borrow()
            .values()
            .cloned()
            .collect())
    }
    fn update_checklist_template(&self, t: &ChecklistTemplate) -> Result<(), Self::Error> {
        self.checklist_templates
            .borrow_mut()
            .insert(t.id.value().to_string(), t.clone());
        Ok(())
    }
    fn delete_checklist_template(&self, id: Id<ChecklistTemplate>) -> Result<(), Self::Error> {
        self.checklist_templates
            .borrow_mut()
            .remove(&id.value().to_string());
        Ok(())
    }

    fn insert_checklist_run(&self, run: &ChecklistRun) -> Result<(), Self::Error> {
        self.checklist_runs
            .borrow_mut()
            .insert(run.id.value().to_string(), run.clone());
        Ok(())
    }
    fn get_checklist_run(&self, id: Id<ChecklistRun>) -> Result<Option<ChecklistRun>, Self::Error> {
        Ok(self
            .checklist_runs
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_checklist_runs_by_log(
        &self,
        log_id: Id<DailyLog>,
    ) -> Result<Vec<ChecklistRun>, Self::Error> {
        Ok(self
            .checklist_runs
            .borrow()
            .values()
            .filter(|r| r.daily_log_id == log_id)
            .cloned()
            .collect())
    }
    fn update_checklist_run(&self, run: &ChecklistRun) -> Result<(), Self::Error> {
        self.checklist_runs
            .borrow_mut()
            .insert(run.id.value().to_string(), run.clone());
        Ok(())
    }
    fn delete_checklist_run(&self, id: Id<ChecklistRun>) -> Result<(), Self::Error> {
        self.checklist_runs
            .borrow_mut()
            .remove(&id.value().to_string());
        Ok(())
    }

    fn insert_journal_entry(&self, entry: &JournalEntry) -> Result<(), Self::Error> {
        self.journal_entries
            .borrow_mut()
            .insert(entry.id.value().to_string(), entry.clone());
        Ok(())
    }
    fn get_journal_entry(&self, id: Id<JournalEntry>) -> Result<Option<JournalEntry>, Self::Error> {
        Ok(self
            .journal_entries
            .borrow()
            .get(&id.value().to_string())
            .cloned())
    }
    fn list_journal_entries_by_log(
        &self,
        log_id: Id<DailyLog>,
    ) -> Result<Vec<JournalEntry>, Self::Error> {
        Ok(self
            .journal_entries
            .borrow()
            .values()
            .filter(|e| e.daily_log_id == log_id)
            .cloned()
            .collect())
    }
    fn update_journal_entry(&self, entry: &JournalEntry) -> Result<(), Self::Error> {
        self.journal_entries
            .borrow_mut()
            .insert(entry.id.value().to_string(), entry.clone());
        Ok(())
    }
    fn delete_journal_entry(&self, id: Id<JournalEntry>) -> Result<(), Self::Error> {
        self.journal_entries
            .borrow_mut()
            .remove(&id.value().to_string());
        Ok(())
    }

    fn insert_checkin_composite(
        &self,
        ci: &CheckIn,
        daily_log_update: Option<&DailyLog>,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.insert_check_in(ci)?;
        if let Some(log) = daily_log_update {
            self.update_daily_log(log)?;
        }
        self.insert_journal_entry(journal)
    }

    fn start_plan_item_composite(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_plan_item(item)?;
        self.insert_task_checkpoint(checkpoint)?;
        self.insert_journal_entry(journal)
    }

    fn done_plan_item_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_plan_item(item)?;
        self.insert_journal_entry(journal)
    }

    fn move_to_waiting_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_plan_item(item)?;
        self.insert_journal_entry(journal)
    }

    fn complete_checklist_composite(
        &self,
        run: &ChecklistRun,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_checklist_run(run)?;
        self.insert_journal_entry(journal)
    }

    fn insert_plan_item_with_checkpoint(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
    ) -> Result<(), Self::Error> {
        self.insert_plan_item(item)?;
        self.insert_task_checkpoint(checkpoint)?;
        Ok(())
    }

    fn answer_checkpoint_composite(
        &self,
        checkpoint: &TaskCheckpoint,
        next_checkpoint: Option<&TaskCheckpoint>,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_task_checkpoint(checkpoint)?;
        if let Some(next_cp) = next_checkpoint {
            self.insert_task_checkpoint(next_cp)?;
        }
        self.insert_journal_entry(journal)?;
        Ok(())
    }

    fn bundle_apply_composite(
        &self,
        _bundle: &crate::bundle::bundle_models::AdiyutantBundle,
        _mode: crate::bundle::bundle_models::ImportMode,
        import_run: &ImportRun,
    ) -> Result<(), Self::Error> {
        self.insert_import_run(import_run)
    }

    // ── ImportRun ──
    fn insert_import_run(&self, run: &ImportRun) -> Result<(), Self::Error> {
        self.import_runs
            .borrow_mut()
            .insert(run.id.value().to_string(), run.clone());
        Ok(())
    }
    fn list_import_runs(&self) -> Result<Vec<ImportRun>, Self::Error> {
        Ok(self.import_runs.borrow().values().cloned().collect())
    }

    // ── Slug-based lookups ──
    fn get_checklist_template_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ChecklistTemplate>, Self::Error> {
        Ok(self
            .checklist_templates
            .borrow()
            .values()
            .find(|t| t.slug.as_deref() == Some(slug))
            .cloned())
    }
    fn upsert_checklist_template(&self, template: &ChecklistTemplate) -> Result<(), Self::Error> {
        if let Some(ref slug) = template.slug {
            let existing = self
                .checklist_templates
                .borrow()
                .values()
                .find(|t| t.slug.as_deref() == Some(slug))
                .cloned();
            if let Some(existing_template) = existing {
                let mut updated = template.clone();
                updated.id = existing_template.id;
                self.checklist_templates
                    .borrow_mut()
                    .insert(updated.id.value().to_string(), updated);
            } else {
                self.insert_checklist_template(template)?;
            }
        } else {
            self.insert_checklist_template(template)?;
        }
        Ok(())
    }
    fn get_context_document_by_source_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ContextDocument>, Self::Error> {
        Ok(self
            .context_docs
            .borrow()
            .values()
            .find(|d| d.source_slug.as_deref() == Some(slug))
            .cloned())
    }
    fn upsert_context_document(&self, doc: &ContextDocument) -> Result<(), Self::Error> {
        if let Some(ref slug) = doc.source_slug {
            let existing = self
                .context_docs
                .borrow()
                .values()
                .find(|d| d.source_slug.as_deref() == Some(slug))
                .cloned();
            if let Some(existing_doc) = existing {
                let mut updated = doc.clone();
                updated.id = existing_doc.id;
                self.context_docs
                    .borrow_mut()
                    .insert(updated.id.value().to_string(), updated);
            } else {
                self.insert_context_document(doc)?;
            }
        } else {
            self.insert_context_document(doc)?;
        }
        Ok(())
    }
}
