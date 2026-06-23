use crate::id::Id;
use crate::model::action_proposal::ProposalStatus;
use crate::model::checklist_template::ChecklistTemplate;
use crate::model::day_plan::{PlanItem, PlanItemStatus};
use crate::model::import_run::ImportRun;
use crate::model::journal_entry::JournalEntry;
use crate::model::task_checkpoint::TaskCheckpoint;
use crate::model::*;
use chrono::NaiveDate;

// ──────────────────────────────────────────────
// Store trait — all domain entities
// ──────────────────────────────────────────────

/// Persistence contract for all domain entities.
///
/// Implemented by `SqliteStore` (adiyutant_store) and `NoopStore` (testing).
pub trait Store {
    type Error;

    // Bootstrap
    fn migrate(&self) -> Result<(), Self::Error>;
    fn health_check(&self) -> Result<(), Self::Error>;

    // ── DailyLog ──
    fn insert_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error>;
    fn get_daily_log(&self, id: Id<DailyLog>) -> Result<Option<DailyLog>, Self::Error>;
    fn get_daily_log_by_date(&self, date: NaiveDate) -> Result<Option<DailyLog>, Self::Error>;
    fn list_daily_logs(&self) -> Result<Vec<DailyLog>, Self::Error>;
    fn update_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error>;
    fn delete_daily_log(&self, id: Id<DailyLog>) -> Result<(), Self::Error>;

    // ── CheckIn ──
    fn insert_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error>;
    fn get_check_in(&self, id: Id<CheckIn>) -> Result<Option<CheckIn>, Self::Error>;
    fn list_check_ins_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<CheckIn>, Self::Error>;
    fn update_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error>;
    fn delete_check_in(&self, id: Id<CheckIn>) -> Result<(), Self::Error>;

    // ── Habit ──
    fn insert_habit(&self, h: &Habit) -> Result<(), Self::Error>;
    fn get_habit(&self, id: Id<Habit>) -> Result<Option<Habit>, Self::Error>;
    fn list_habits(&self) -> Result<Vec<Habit>, Self::Error>;
    fn update_habit(&self, h: &Habit) -> Result<(), Self::Error>;
    fn delete_habit(&self, id: Id<Habit>) -> Result<(), Self::Error>;

    // ── HabitEvent ──
    fn insert_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error>;
    fn get_habit_event(&self, id: Id<HabitEvent>) -> Result<Option<HabitEvent>, Self::Error>;
    fn list_habit_events_by_habit(
        &self,
        habit_id: Id<Habit>,
    ) -> Result<Vec<HabitEvent>, Self::Error>;
    fn update_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error>;
    fn delete_habit_event(&self, id: Id<HabitEvent>) -> Result<(), Self::Error>;

    // ── ReminderDefinition ──
    fn insert_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error>;
    fn get_reminder(
        &self,
        id: Id<ReminderDefinition>,
    ) -> Result<Option<ReminderDefinition>, Self::Error>;
    fn list_reminders(&self) -> Result<Vec<ReminderDefinition>, Self::Error>;
    fn update_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error>;
    fn delete_reminder(&self, id: Id<ReminderDefinition>) -> Result<(), Self::Error>;

    // ── AlarmDefinition ──
    fn insert_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error>;
    fn get_alarm(&self, id: Id<AlarmDefinition>) -> Result<Option<AlarmDefinition>, Self::Error>;
    fn list_alarms(&self) -> Result<Vec<AlarmDefinition>, Self::Error>;
    fn update_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error>;
    fn delete_alarm(&self, id: Id<AlarmDefinition>) -> Result<(), Self::Error>;

    // ── TimerDefinition ──
    fn insert_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error>;
    fn get_timer(&self, id: Id<TimerDefinition>) -> Result<Option<TimerDefinition>, Self::Error>;
    fn list_timers(&self) -> Result<Vec<TimerDefinition>, Self::Error>;
    fn update_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error>;
    fn delete_timer(&self, id: Id<TimerDefinition>) -> Result<(), Self::Error>;

    // ── ContextDocument ──
    fn insert_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error>;
    fn get_context_document(
        &self,
        id: Id<ContextDocument>,
    ) -> Result<Option<ContextDocument>, Self::Error>;
    fn list_context_documents(&self) -> Result<Vec<ContextDocument>, Self::Error>;
    fn update_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error>;
    fn delete_context_document(&self, id: Id<ContextDocument>) -> Result<(), Self::Error>;

    // ── Plan (legacy) ──
    fn insert_plan(&self, p: &Plan) -> Result<(), Self::Error>;
    fn get_plan(&self, id: Id<Plan>) -> Result<Option<Plan>, Self::Error>;
    fn get_plan_by_daily_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Option<Plan>, Self::Error>;
    fn update_plan(&self, p: &Plan) -> Result<(), Self::Error>;
    fn delete_plan(&self, id: Id<Plan>) -> Result<(), Self::Error>;

    // ── ActionProposal ──
    fn insert_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error>;
    fn get_action_proposal(
        &self,
        id: Id<ActionProposal>,
    ) -> Result<Option<ActionProposal>, Self::Error>;
    fn list_action_proposals(&self) -> Result<Vec<ActionProposal>, Self::Error>;
    fn list_action_proposals_by_status(
        &self,
        status: ProposalStatus,
    ) -> Result<Vec<ActionProposal>, Self::Error>;
    fn update_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error>;
    fn delete_action_proposal(&self, id: Id<ActionProposal>) -> Result<(), Self::Error>;

    // ── PlanItem (new) ──
    fn insert_plan_item(&self, item: &PlanItem) -> Result<(), Self::Error>;
    fn get_plan_item(&self, id: Id<PlanItem>) -> Result<Option<PlanItem>, Self::Error>;
    fn list_plan_items_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<PlanItem>, Self::Error>;
    fn update_plan_item(&self, item: &PlanItem) -> Result<(), Self::Error>;
    fn delete_plan_item(&self, id: Id<PlanItem>) -> Result<(), Self::Error>;
    fn list_plan_items_by_status(
        &self,
        status: PlanItemStatus,
    ) -> Result<Vec<PlanItem>, Self::Error>;
    fn list_plan_items_due_for_review(&self, date: NaiveDate)
    -> Result<Vec<PlanItem>, Self::Error>;

    // ── TaskCheckpoint ──
    fn insert_task_checkpoint(&self, cp: &TaskCheckpoint) -> Result<(), Self::Error>;
    fn get_task_checkpoint(
        &self,
        id: Id<TaskCheckpoint>,
    ) -> Result<Option<TaskCheckpoint>, Self::Error>;
    fn list_checkpoints_by_plan_item(
        &self,
        plan_item_id: Id<PlanItem>,
    ) -> Result<Vec<TaskCheckpoint>, Self::Error>;
    fn update_task_checkpoint(&self, cp: &TaskCheckpoint) -> Result<(), Self::Error>;
    fn delete_task_checkpoint(&self, id: Id<TaskCheckpoint>) -> Result<(), Self::Error>;

    // ── ChecklistTemplate ──
    fn insert_checklist_template(&self, template: &ChecklistTemplate) -> Result<(), Self::Error>;
    fn get_checklist_template(
        &self,
        id: Id<ChecklistTemplate>,
    ) -> Result<Option<ChecklistTemplate>, Self::Error>;
    fn list_checklist_templates_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<ChecklistTemplate>, Self::Error>;
    fn list_checklist_templates(&self) -> Result<Vec<ChecklistTemplate>, Self::Error>;
    fn update_checklist_template(&self, template: &ChecklistTemplate) -> Result<(), Self::Error>;
    fn delete_checklist_template(&self, id: Id<ChecklistTemplate>) -> Result<(), Self::Error>;

    // ── ChecklistRun ──
    fn insert_checklist_run(&self, run: &ChecklistRun) -> Result<(), Self::Error>;
    fn get_checklist_run(&self, id: Id<ChecklistRun>) -> Result<Option<ChecklistRun>, Self::Error>;
    fn list_checklist_runs_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<ChecklistRun>, Self::Error>;
    fn update_checklist_run(&self, run: &ChecklistRun) -> Result<(), Self::Error>;
    fn delete_checklist_run(&self, id: Id<ChecklistRun>) -> Result<(), Self::Error>;

    // ── JournalEntry ──
    fn insert_journal_entry(&self, entry: &JournalEntry) -> Result<(), Self::Error>;
    fn get_journal_entry(&self, id: Id<JournalEntry>) -> Result<Option<JournalEntry>, Self::Error>;
    fn list_journal_entries_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<JournalEntry>, Self::Error>;
    fn update_journal_entry(&self, entry: &JournalEntry) -> Result<(), Self::Error>;
    fn delete_journal_entry(&self, id: Id<JournalEntry>) -> Result<(), Self::Error>;

    // ── Composite (transactional) operations ──
    /// Create a check-in with optional daily-log update and journal entry (atomic).
    fn insert_checkin_composite(
        &self,
        ci: &CheckIn,
        daily_log_update: Option<&DailyLog>,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error>;
    /// Start a plan item: update status, insert checkpoint, create journal entry (atomic).
    fn start_plan_item_composite(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error>;
    /// Complete a plan item: update status, create journal entry (atomic).
    fn done_plan_item_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error>;
    /// Move to waiting: update status, create journal entry (atomic).
    fn move_to_waiting_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error>;
    /// Complete a checklist run: update run + insert journal entry (atomic).
    fn complete_checklist_composite(
        &self,
        run: &ChecklistRun,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error>;

    /// Insert a plan item with its initial StartCheck checkpoint (atomic).
    /// Used by `add_plan_item()` to ensure the item and its first checkpoint
    /// are either both created, or neither.
    fn insert_plan_item_with_checkpoint(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
    ) -> Result<(), Self::Error>;

    /// Answer a checkpoint, optionally create the next checkpoint, and journal the event (atomic).
    /// Used by `answer_checkpoint()` to ensure checkpoint update, next checkpoint creation,
    /// and journal entry are either all committed or all rolled back.
    fn answer_checkpoint_composite(
        &self,
        checkpoint: &TaskCheckpoint,
        next_checkpoint: Option<&TaskCheckpoint>,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error>;

    // ── ImportRun ──
    fn insert_import_run(&self, run: &ImportRun) -> Result<(), Self::Error>;
    fn list_import_runs(&self) -> Result<Vec<ImportRun>, Self::Error>;

    // ── Slug-based lookups (for bundle merge) ──
    fn get_checklist_template_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ChecklistTemplate>, Self::Error>;
    fn upsert_checklist_template(&self, template: &ChecklistTemplate) -> Result<(), Self::Error>;
    fn get_context_document_by_source_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ContextDocument>, Self::Error>;
    fn upsert_context_document(&self, doc: &ContextDocument) -> Result<(), Self::Error>;

    /// Apply a bundle's supported sections atomically within a single transaction.
    fn bundle_apply_composite(
        &self,
        bundle: &crate::bundle::bundle_models::AdiyutantBundle,
        mode: crate::bundle::bundle_models::ImportMode,
        import_run: &ImportRun,
    ) -> Result<(), Self::Error>;
}
