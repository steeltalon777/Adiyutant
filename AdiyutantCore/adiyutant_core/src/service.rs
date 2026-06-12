use crate::current_activity::CurrentActivity;
use crate::datetime::AdiyutantDateTime;
use crate::dto::{
    self, ChecklistRunDto, ChecklistTemplateDto, CurrentActivityDto, DayPlanDto, JournalEntryDto,
    NotificationInstructionDto, PlanItemDto, StartupViewDto, TodayViewDto, WaitingTaskDto,
};
use crate::error::CoreError;
use crate::id::Id;
use crate::local_rule_gateway::LocalRuleAgentGateway;
use crate::model::checklist_run::{ChecklistAnswer, ChecklistRun};
use crate::model::checklist_template::{ChecklistItem, ChecklistTemplate};

use crate::model::daily_log::DailyLog;
use crate::model::day_plan::{EisenhowerQuadrant, PlanItem, PlanItemStatus, WaitingDecision};
use crate::model::habit::Habit;
use crate::model::habit_event::HabitEvent;
use crate::model::journal_entry::{JournalEntry, JournalEntryType};
use crate::model::task_checkpoint::{
    CheckpointKind, CheckpointResponse, CheckpointStatus, TaskCheckpoint,
};
use crate::startup::StartupState;
use crate::store::Store;
use crate::time_provider::{RealTimeProvider, TimeProvider};
use crate::today_state::{TodayState, TodayStateBuilder};

/// Application-layer facade / use-case boundary.
///
/// All public methods return stable DTOs. Consumers (CLI, Android)
/// must never access Store or domain internals directly.
pub struct AdiyutantCoreService {
    store: Box<dyn Store<Error = CoreError>>,
    time: Box<dyn TimeProvider>,
}

impl AdiyutantCoreService {
    pub fn new(store: Box<dyn Store<Error = CoreError>>) -> Self {
        Self {
            store,
            time: Box::new(RealTimeProvider),
        }
    }

    pub fn with_time(
        store: Box<dyn Store<Error = CoreError>>,
        time: Box<dyn TimeProvider>,
    ) -> Self {
        Self { store, time }
    }

    /// Build a `TodayState` snapshot from the store.
    fn build_today_state(&self) -> Result<TodayState, CoreError> {
        let today = self.time.today();
        let mut builder = TodayStateBuilder::new().date(today);

        // DailyLog
        if let Some(log) = self.store.get_daily_log_by_date(today)? {
            builder = builder.daily_log(log.clone());

            // Check-ins for this log
            let check_ins = self.store.list_check_ins_by_log(log.id)?;
            builder = builder.check_ins(check_ins);

            // Plan for this log
            if let Some(plan) = self.store.get_plan_by_daily_log(log.id)? {
                builder = builder.plan(plan);
            }
        }

        // PlanItems (new)
        if let Some(log) = self.store.get_daily_log_by_date(today)? {
            let plan_items = self.store.list_plan_items_by_log(log.id)?;
            builder = builder.plan_items(plan_items);
        }

        // Habits
        let habits = self.store.list_habits()?;
        builder = builder.habits(habits.clone());

        // Habit events filtered to today
        for habit in &habits {
            let events = self.store.list_habit_events_by_habit(habit.id)?;
            let today_events: Vec<HabitEvent> = events
                .into_iter()
                .filter(|e| e.date_time.inner().date_naive() == today)
                .collect();
            for ev in today_events {
                builder = builder.habit_event(ev);
            }
        }

        // Timers, reminders, alarms, context docs
        builder = builder.timers(self.store.list_timers()?);
        builder = builder.reminders(self.store.list_reminders()?);
        builder = builder.alarms(self.store.list_alarms()?);
        builder = builder.context_documents(self.store.list_context_documents()?);

        Ok(builder.build())
    }

    /// Full today's dashboard view.
    pub fn get_today(&self) -> Result<TodayViewDto, CoreError> {
        let state = self.build_today_state()?;
        let legacy_item_count = state.plan.as_ref().map_or(0, |p| p.items.len());
        let new_item_count = state.plan_items.len();

        Ok(TodayViewDto {
            date: dto::naive_date_to_string(state.date),
            has_daily_log: state.daily_log.is_some(),
            mode: state
                .daily_log
                .as_ref()
                .and_then(|l| l.mode.as_deref())
                .unwrap_or("none")
                .to_string(),
            sleep_score: dto::option_u8_string(
                &state.daily_log.as_ref().and_then(|l| l.sleep_score),
            ),
            energy: dto::option_u8_string(&state.daily_log.as_ref().and_then(|l| l.energy)),
            mood: dto::option_u8_string(&state.daily_log.as_ref().and_then(|l| l.mood)),
            check_in_count: state.check_ins.len(),
            has_morning_check_in: state.morning_check_in().is_some(),
            has_evening_check_in: state.evening_check_in().is_some(),
            habit_count: state.habits.len(),
            habit_events_done: state.habit_event_count(),
            has_plan: state.has_plan(),
            plan_title: state
                .plan
                .as_ref()
                .map(|p| p.title.clone())
                .or_else(|| {
                    if !state.plan_items.is_empty() {
                        Some(format!("Plan for {}", state.date.format("%Y-%m-%d")))
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
            plan_item_count: legacy_item_count + new_item_count,
            plan_items: {
                let mut items: Vec<dto::PlanItemViewDto> = Vec::new();
                // New plan_items
                for pi in &state.plan_items {
                    items.push(dto::PlanItemViewDto {
                        description: pi.title.clone(),
                        status: pi.status.as_str().to_string(),
                        estimated_minutes: String::new(),
                        notes: dto::option_string(&pi.description),
                    });
                }
                // Legacy plan items
                if let Some(plan) = &state.plan {
                    for item in &plan.items {
                        items.push(dto::PlanItemViewDto {
                            description: item.description.clone(),
                            status: format!("{:?}", item.status),
                            estimated_minutes: item
                                .estimated_minutes
                                .map(|m| m.to_string())
                                .unwrap_or_default(),
                            notes: dto::option_string(&item.notes),
                        });
                    }
                }
                items
            },
            suggestion_count: {
                let rule_result = LocalRuleAgentGateway::evaluate(&state);
                rule_result.proposals.len()
            },
        })
    }

    /// What to show / suggest on launch.
    pub fn get_startup_state(&self) -> Result<StartupViewDto, CoreError> {
        let state = self.build_today_state()?;
        let startup = StartupState::determine_at_hour(&state, self.time.hour());
        Ok(StartupViewDto {
            intent: startup.intent.as_str().to_string(),
            reason: startup.reason,
        })
    }

    /// What the user is (or should be) doing right now.
    pub fn get_current_activity(&self) -> Result<CurrentActivityDto, CoreError> {
        let state = self.build_today_state()?;
        let activity = CurrentActivity::determine_at_hour(&state, self.time.hour());
        Ok(CurrentActivityDto {
            date: dto::naive_date_to_string(state.date),
            activity_kind: activity.kind.as_str().to_string(),
            active_task: activity.active_task.unwrap_or_default(),
            active_block: activity.active_block.unwrap_or_default(),
            next_checkpoint_at: activity
                .next_checkpoint
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_default(),
            recommended_prompt: activity.recommended_prompt,
        })
    }

    // ── Planning Domain ────────────────────────

    /// Get or create today's DailyLog.
    fn get_or_create_today_log(&self) -> Result<Id<DailyLog>, CoreError> {
        let today = self.time.today();
        if let Some(log) = self.store.get_daily_log_by_date(today)? {
            Ok(log.id)
        } else {
            let log = DailyLog::new(today);
            self.store.insert_daily_log(&log)?;
            Ok(log.id)
        }
    }

    /// Add a plan item for today.
    pub fn add_plan_item(
        &self,
        title: &str,
        quadrant: Option<&str>,
        priority: Option<u8>,
    ) -> Result<PlanItemDto, CoreError> {
        let daily_log_id = self.get_or_create_today_log()?;
        let mut item = PlanItem::new(daily_log_id, title.to_string());

        if let Some(q) = quadrant {
            item.quadrant = EisenhowerQuadrant::from_str(q)
                .ok_or_else(|| CoreError::InvalidInput(format!("unknown quadrant: {q}")))?;
        }
        if let Some(p) = priority {
            item.priority = p;
        }

        self.store.insert_plan_item(&item)?;

        // Auto-create a StartCheck checkpoint
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::StartCheck);
        let _ = self.store.insert_task_checkpoint(&cp);

        Ok(plan_item_to_dto(&item))
    }

    /// List today's plan items.
    pub fn list_plan_items(&self) -> Result<DayPlanDto, CoreError> {
        let today = self.time.today();
        let daily_log_id = match self.store.get_daily_log_by_date(today)? {
            Some(log) => log.id,
            None => {
                return Ok(DayPlanDto {
                    id: String::new(),
                    date: dto::naive_date_to_string(today),
                    items: vec![],
                    item_count: 0,
                });
            }
        };

        let items = self.store.list_plan_items_by_log(daily_log_id)?;
        let item_dtos: Vec<PlanItemDto> = items.iter().map(plan_item_to_dto).collect();

        Ok(DayPlanDto {
            id: String::new(),
            date: dto::naive_date_to_string(today),
            items: item_dtos,
            item_count: items.len(),
        })
    }

    /// Start a plan item (set status to Started).
    pub fn start_plan_item(&self, item_id: &str) -> Result<PlanItemDto, CoreError> {
        let id = parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| CoreError::NotFound(format!("plan_item {item_id}")))?;

        item.status = PlanItemStatus::Started;
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        let cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        let entry = JournalEntry::new(
            item.daily_log_id,
            JournalEntryType::TaskStarted,
            format!("Started: {}", item.title),
        );

        // Atomic: update_plan_item + insert_task_checkpoint + insert_journal_entry
        self.store
            .start_plan_item_composite(&item, &cp, &entry)?;
        Ok(plan_item_to_dto(&item))
    }

    /// Mark a plan item as Done.
    pub fn done_plan_item(&self, item_id: &str) -> Result<PlanItemDto, CoreError> {
        let id = parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| CoreError::NotFound(format!("plan_item {item_id}")))?;

        item.status = PlanItemStatus::Done;
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        let entry = JournalEntry::new(
            item.daily_log_id,
            JournalEntryType::TaskDone,
            format!("Done: {}", item.title),
        );

        // Atomic: update_plan_item + insert_journal_entry
        self.store.done_plan_item_composite(&item, &entry)?;
        Ok(plan_item_to_dto(&item))
    }

    /// Move a plan item to waiting status.
    pub fn move_to_waiting(
        &self,
        item_id: &str,
        review_days: Option<u64>,
    ) -> Result<PlanItemDto, CoreError> {
        let id = parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| CoreError::NotFound(format!("plan_item {item_id}")))?;

        item.status = PlanItemStatus::Waiting;
        item.waiting_review_at =
            Some(self.time.today() + chrono::Duration::days(review_days.unwrap_or(7) as i64));
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        let entry = JournalEntry::new(
            item.daily_log_id,
            JournalEntryType::TaskMoved,
            format!("Moved to waiting: {}", item.title),
        );

        // Atomic: update_plan_item + insert_journal_entry
        self.store.move_to_waiting_composite(&item, &entry)?;
        Ok(plan_item_to_dto(&item))
    }

    /// List waiting items due for review.
    pub fn list_waiting_tasks(&self) -> Result<Vec<WaitingTaskDto>, CoreError> {
        let today = self.time.today();
        let items = self.store.list_plan_items_due_for_review(today)?;
        Ok(items
            .iter()
            .map(|item| WaitingTaskDto {
                id: item.id.value().to_string(),
                title: item.title.clone(),
                waiting_since: item.updated_at.inner().to_string(),
                review_due: item
                    .waiting_review_at
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
            })
            .collect())
    }

    /// Review a waiting item.
    pub fn review_waiting(&self, item_id: &str, decision: &str) -> Result<PlanItemDto, CoreError> {
        let decision = WaitingDecision::from_str(decision).ok_or_else(|| {
            CoreError::InvalidInput(format!(
                "unknown decision: {decision} (use: keep, resume, delete, archive)"
            ))
        })?;

        let id = parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| CoreError::NotFound(format!("plan_item {item_id}")))?;

        match decision {
            WaitingDecision::Keep => {
                // Keep waiting, extend review by 7 days
                item.waiting_review_at = Some(self.time.today() + chrono::Duration::days(7));
            }
            WaitingDecision::Resume => {
                item.status = PlanItemStatus::Planned;
                item.waiting_review_at = None;
            }
            WaitingDecision::Delete => {
                self.store.delete_plan_item(id)?;
                return Ok(PlanItemDto {
                    id: item_id.to_string(),
                    title: item.title,
                    description: dto::option_string(&item.description),
                    quadrant: item.quadrant.as_str().to_string(),
                    planned_start: String::new(),
                    planned_end: String::new(),
                    status: "Deleted".to_string(),
                    priority: item.priority,
                    source: item.source.as_str().to_string(),
                    created_at: String::new(),
                    updated_at: String::new(),
                });
            }
            WaitingDecision::Archive => {
                item.status = PlanItemStatus::Archived;
                item.waiting_review_at = None;
            }
        }
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        self.store.update_plan_item(&item)?;
        Ok(plan_item_to_dto(&item))
    }

    // ── Checkpoint Domain ──────────────────────

    /// Answer a checkpoint: transition Pending or Shown → Answered with response.
    /// Auto-creates the next checkpoint in the sequence via `calculate_next_checkpoint`.
    pub fn answer_checkpoint(
        &self,
        checkpoint_id: &str,
        response: &str,
    ) -> Result<PlanItemDto, CoreError> {
        let id = parse_id::<TaskCheckpoint>(checkpoint_id)?;
        let mut cp = self
            .store
            .get_task_checkpoint(id)?
            .ok_or_else(|| CoreError::NotFound(format!("checkpoint {checkpoint_id}")))?;

        let resp = CheckpointResponse::from_str(response).ok_or_else(|| {
            CoreError::InvalidInput(format!("unknown response: {response}"))
        })?;

        match cp.status {
            CheckpointStatus::Pending | CheckpointStatus::Shown => {
                cp.status = CheckpointStatus::Answered;
                cp.response = Some(resp);
                cp.answered_at = Some(AdiyutantDateTime::from_utc(self.time.now_utc()));
            }
            CheckpointStatus::Answered | CheckpointStatus::Dismissed => {
                return Err(CoreError::InvalidInput(
                    "checkpoint already answered or dismissed".into(),
                ));
            }
        }

        self.store.update_task_checkpoint(&cp)?;

        let item = self
            .store
            .get_plan_item(cp.plan_item_id)?
            .ok_or_else(|| CoreError::NotFound("plan_item for checkpoint".into()))?;

        // Auto-create next checkpoint in the sequence
        if let Some(next_kind) = Self::calculate_next_checkpoint(&item, &cp.kind, &resp) {
            let next_cp = TaskCheckpoint::new(item.id, next_kind);
            let _ = self.store.insert_task_checkpoint(&next_cp);
        }

        Ok(plan_item_to_dto(&item))
    }

    /// Dismiss a checkpoint: transition Pending or Shown → Dismissed.
    pub fn dismiss_checkpoint(&self, checkpoint_id: &str) -> Result<(), CoreError> {
        let id = parse_id::<TaskCheckpoint>(checkpoint_id)?;
        let mut cp = self
            .store
            .get_task_checkpoint(id)?
            .ok_or_else(|| CoreError::NotFound(format!("checkpoint {checkpoint_id}")))?;

        match cp.status {
            CheckpointStatus::Pending | CheckpointStatus::Shown => {
                cp.status = CheckpointStatus::Dismissed;
            }
            CheckpointStatus::Answered | CheckpointStatus::Dismissed => {
                return Err(CoreError::InvalidInput(
                    "checkpoint already answered or dismissed".into(),
                ));
            }
        }

        self.store.update_task_checkpoint(&cp)
    }

    /// Determine the next checkpoint kind after answering one.
    fn calculate_next_checkpoint(
        _plan_item: &PlanItem,
        last_kind: &CheckpointKind,
        last_response: &CheckpointResponse,
    ) -> Option<CheckpointKind> {
        match (last_kind, last_response) {
            (CheckpointKind::StartCheck, CheckpointResponse::Started) => {
                Some(CheckpointKind::ProgressCheck)
            }
            (CheckpointKind::ProgressCheck, CheckpointResponse::Done) => {
                Some(CheckpointKind::FinishCheck)
            }
            (CheckpointKind::ProgressCheck, CheckpointResponse::Move) => {
                Some(CheckpointKind::RescheduleCheck)
            }
            (CheckpointKind::FinishCheck, CheckpointResponse::Done) => None,
            (CheckpointKind::RescheduleCheck, CheckpointResponse::KeepWaiting) => {
                Some(CheckpointKind::RelevanceReview)
            }
            _ => None,
        }
    }

    pub fn get_pending_checkpoint_notification(
        &self,
    ) -> Result<Option<NotificationInstructionDto>, CoreError> {
        let today = self.time.today();
        let daily_log_id = match self.store.get_daily_log_by_date(today)? {
            Some(log) => log.id,
            None => return Ok(None),
        };

        let plan_items = self.store.list_plan_items_by_log(daily_log_id)?;
        for item in &plan_items {
            let checkpoints = self.store.list_checkpoints_by_plan_item(item.id)?;
            for cp in checkpoints {
                if matches!(cp.status, CheckpointStatus::Pending) {
                    return Ok(Some(NotificationInstructionDto {
                        source: "Checkpoint".into(),
                        title: format!("Checkpoint: {}", cp.kind.as_str()),
                        body: format!("Time to check in on task: {}", item.title),
                        action_id: cp.id.value().to_string(),
                    }));
                }
            }
        }
        Ok(None)
    }

    // ── Checklist Domain ───────────────────────

    /// List all available checklist templates.
    pub fn list_checklist_templates(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<ChecklistTemplateDto>, CoreError> {
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
    fn seed_default_checklist(&self, category: &str) -> Result<ChecklistTemplate, CoreError> {
        use crate::model::checklist_template::{
            ChecklistItem, ChecklistItemKind, ChecklistTemplate,
        };

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
            _ => return Err(CoreError::NotFound(format!("unknown category: {category}"))),
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
    /// Auto-seeds a default template if none exists for the requested category.
    pub fn run_checklist(&self, category: &str) -> Result<ChecklistRunDto, CoreError> {
        let templates = self.store.list_checklist_templates_by_category(category)?;
        let template = if let Some(t) = templates.first() {
            t.clone()
        } else {
            // Auto-seed a default template for the requested category
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
    ) -> Result<ChecklistRunDto, CoreError> {
        let run_uuid = parse_id::<ChecklistRun>(run_id)?;
        let mut run = self
            .store
            .get_checklist_run(run_uuid)?
            .ok_or_else(|| CoreError::NotFound(format!("checklist_run {run_id}")))?;

        if run.completed_at.is_some() {
            return Err(CoreError::InvalidInput(
                "checklist run already completed".into(),
            ));
        }

        let item_uuid = parse_id::<ChecklistItem>(item_id)?;
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
    pub fn complete_checklist_run(&self, run_id: &str) -> Result<ChecklistRunDto, CoreError> {
        let uuid = parse_id::<ChecklistRun>(run_id)?;
        let mut run = self
            .store
            .get_checklist_run(uuid)?
            .ok_or_else(|| CoreError::NotFound(format!("checklist_run {run_id}")))?;

        if run.completed_at.is_some() {
            return Err(CoreError::InvalidInput(
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

    // ── Journal Domain ─────────────────────────

    /// List journal entries for today.
    pub fn get_journal(&self) -> Result<Vec<JournalEntryDto>, CoreError> {
        let today = self.time.today();
        let entries = if let Some(log) = self.store.get_daily_log_by_date(today)? {
            self.store.list_journal_entries_by_log(log.id)?
        } else {
            vec![]
        };
        Ok(entries
            .iter()
            .map(|e| JournalEntryDto {
                id: e.id.value().to_string(),
                entry_type: e.entry_type.as_str().to_string(),
                summary: e.summary.clone(),
                timestamp: e.timestamp.inner().to_rfc3339(),
            })
            .collect())
    }

    /// Add a journal entry for today.
    pub fn add_journal_entry(
        &self,
        entry_type: &str,
        summary: &str,
    ) -> Result<JournalEntryDto, CoreError> {
        let entry_type = JournalEntryType::from_str(entry_type)
            .ok_or_else(|| CoreError::InvalidInput(format!("unknown entry type: {entry_type}")))?;
        let daily_log_id = self.get_or_create_today_log()?;
        let entry = JournalEntry::new(daily_log_id, entry_type, summary.to_string());
        self.store.insert_journal_entry(&entry)?;
        Ok(JournalEntryDto {
            id: entry.id.value().to_string(),
            entry_type: entry.entry_type.as_str().to_string(),
            summary: entry.summary,
            timestamp: entry.timestamp.inner().to_rfc3339(),
        })
    }

    // ── Check-In ───────────────────────────────

    /// Create a check-in and optionally update daily log metrics (transactional).
    pub fn create_checkin(
        &self,
        checkin_type: &str,
        text: &str,
        sleep_score: Option<u8>,
        energy: Option<u8>,
        mood: Option<u8>,
    ) -> Result<String, CoreError> {
        use crate::model::check_in::{CheckIn, CheckInType};
        let daily_log_id = self.get_or_create_today_log()?;
        let ci_type = match checkin_type {
            "morning" => CheckInType::Morning,
            "day" => CheckInType::Day,
            "evening" => CheckInType::Evening,
            "shutdown" => CheckInType::Shutdown,
            _ => {
                return Err(CoreError::InvalidInput(format!(
                    "unknown checkin type: {checkin_type}"
                )));
            }
        };
        let ci = CheckIn::new(daily_log_id, ci_type, text.to_string());

        // Build optional daily log update
        let daily_log_update = if sleep_score.is_some() || energy.is_some() || mood.is_some() {
            self.store.get_daily_log(daily_log_id)?
                .map(|mut log| {
                    if let Some(s) = sleep_score { log.sleep_score = Some(s); }
                    if let Some(e) = energy { log.energy = Some(e); }
                    if let Some(m) = mood { log.mood = Some(m); }
                    log
                })
        } else {
            None
        };

        let entry = JournalEntry::new(
            daily_log_id,
            JournalEntryType::CheckInCreated,
            format!("{checkin_type} check-in: {text}"),
        );

        // Atomic: insert_check_in + update_daily_log + insert_journal_entry
        self.store
            .insert_checkin_composite(&ci, daily_log_update.as_ref(), &entry)?;
        Ok(ci.id.value().to_string())
    }

    // ── Habits ─────────────────────────────────

    pub fn add_habit(&self, name: &str) -> Result<String, CoreError> {
        use crate::model::habit::Habit;
        let habit = Habit::new(name.to_string());
        self.store.insert_habit(&habit)?;
        Ok(habit.id.value().to_string())
    }

    pub fn list_habits(&self) -> Result<Vec<(String, String, bool)>, CoreError> {
        let habits = self.store.list_habits()?;
        Ok(habits
            .iter()
            .map(|h| (h.id.value().to_string(), h.name.clone(), h.is_active))
            .collect())
    }

    pub fn mark_habit_done(
        &self,
        habit_id: &str,
        level: Option<&str>,
    ) -> Result<String, CoreError> {
        use crate::model::habit_event::{HabitEvent, HabitEventLevel, HabitEventStatus};
        let id = parse_id::<Habit>(habit_id)?;
        self.store
            .get_habit(id)?
            .ok_or_else(|| CoreError::NotFound(habit_id.to_string()))?;
        let event_level = level
            .and_then(|l| match l {
                "min" => Some(HabitEventLevel::Min),
                "light" => Some(HabitEventLevel::Light),
                "base" => Some(HabitEventLevel::Base),
                "full" => Some(HabitEventLevel::Full),
                _ => None,
            })
            .unwrap_or(HabitEventLevel::Base);
        let event = HabitEvent::new(id, HabitEventStatus::Done, event_level);
        self.store.insert_habit_event(&event)?;
        Ok(event.id.value().to_string())
    }

    pub fn skip_habit(&self, habit_id: &str) -> Result<String, CoreError> {
        use crate::model::habit_event::{HabitEvent, HabitEventLevel, HabitEventStatus};
        let id = parse_id::<Habit>(habit_id)?;
        self.store
            .get_habit(id)?
            .ok_or_else(|| CoreError::NotFound(habit_id.to_string()))?;
        let event = HabitEvent::new(id, HabitEventStatus::Skipped, HabitEventLevel::Min);
        self.store.insert_habit_event(&event)?;
        Ok(event.id.value().to_string())
    }

    // ── Timers ─────────────────────────────────

    pub fn add_timer(&self, title: &str, duration_seconds: u64) -> Result<String, CoreError> {
        use crate::model::timer::{TimerDefinition, TimerMode};
        let timer = TimerDefinition::new(title.to_string(), duration_seconds, TimerMode::Focus);
        self.store.insert_timer(&timer)?;
        Ok(timer.id.value().to_string())
    }

    pub fn list_timers(&self) -> Result<Vec<(String, String, u64)>, CoreError> {
        let timers = self.store.list_timers()?;
        Ok(timers
            .iter()
            .map(|t| {
                (
                    t.id.value().to_string(),
                    t.title.clone(),
                    t.duration_seconds,
                )
            })
            .collect())
    }

    // ── Reminders ──────────────────────────────

    pub fn add_reminder(&self, title: &str, schedule_rule: &str) -> Result<String, CoreError> {
        use crate::model::reminder::ReminderDefinition;
        let reminder = ReminderDefinition::new(title.to_string(), schedule_rule.to_string());
        self.store.insert_reminder(&reminder)?;
        Ok(reminder.id.value().to_string())
    }

    pub fn list_reminders(&self) -> Result<Vec<(String, String, String)>, CoreError> {
        let reminders = self.store.list_reminders()?;
        Ok(reminders
            .iter()
            .map(|r| {
                (
                    r.id.value().to_string(),
                    r.title.clone(),
                    r.schedule_rule.clone(),
                )
            })
            .collect())
    }

    // ── Alarms ─────────────────────────────────

    pub fn add_alarm(&self, title: &str, time: &str) -> Result<String, CoreError> {
        use crate::model::alarm::AlarmDefinition;
        let parsed_time = chrono::NaiveTime::parse_from_str(time, "%H:%M")
            .map_err(|e| CoreError::InvalidInput(format!("invalid time: {e}")))?;
        let alarm = AlarmDefinition::new(title.to_string(), parsed_time);
        self.store.insert_alarm(&alarm)?;
        Ok(alarm.id.value().to_string())
    }

    pub fn list_alarms(&self) -> Result<Vec<(String, String, String)>, CoreError> {
        let alarms = self.store.list_alarms()?;
        Ok(alarms
            .iter()
            .map(|a| {
                (
                    a.id.value().to_string(),
                    a.title.clone(),
                    a.time.format("%H:%M").to_string(),
                )
            })
            .collect())
    }

    // ── Context Documents ──────────────────────

    pub fn add_context_document(
        &self,
        doc_type: &str,
        title: &str,
        content: &str,
    ) -> Result<String, CoreError> {
        use crate::model::context_document::{ContextDocument, ContextDocumentType};
        let dt: ContextDocumentType = serde_json::from_value(serde_json::json!(doc_type))
            .map_err(|_| CoreError::InvalidInput(format!("unknown doc_type: {doc_type}")))?;
        let doc = ContextDocument::new(dt, title.to_string(), content.to_string());
        self.store.insert_context_document(&doc)?;
        Ok(doc.id.value().to_string())
    }

    pub fn list_context_documents(&self) -> Result<Vec<(String, String, String)>, CoreError> {
        let docs = self.store.list_context_documents()?;
        Ok(docs
            .iter()
            .map(|d| {
                (
                    d.id.value().to_string(),
                    format!("{:?}", d.doc_type),
                    d.title.clone(),
                )
            })
            .collect())
    }

    // ── Suggestions ────────────────────────────

    pub fn get_suggestions(&self) -> Result<Vec<String>, CoreError> {
        let state = self.build_today_state()?;
        let result = LocalRuleAgentGateway::evaluate(&state);
        Ok(result
            .proposals
            .iter()
            .map(|p| {
                let reason = p
                    .payload_json
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let suggestion = p
                    .payload_json
                    .get("suggestion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("{}: {} → {}", p.proposal_type, reason, suggestion)
            })
            .collect())
    }
}

// ── helpers ────────────────────────────────────

fn plan_item_to_dto(item: &PlanItem) -> PlanItemDto {
    PlanItemDto {
        id: item.id.value().to_string(),
        title: item.title.clone(),
        description: dto::option_string(&item.description),
        quadrant: item.quadrant.as_str().to_string(),
        planned_start: item
            .planned_start
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_default(),
        planned_end: item
            .planned_end
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_default(),
        status: item.status.as_str().to_string(),
        priority: item.priority,
        source: item.source.as_str().to_string(),
        created_at: item.created_at.inner().to_rfc3339(),
        updated_at: item.updated_at.inner().to_rfc3339(),
    }
}

fn parse_item_id(s: &str) -> Result<Id<PlanItem>, CoreError> {
    let uuid = uuid::Uuid::parse_str(s)
        .map_err(|e| CoreError::InvalidInput(format!("invalid item id: {e}")))?;
    Ok(Id::from_uuid(uuid))
}

fn parse_id<T>(s: &str) -> Result<Id<T>, CoreError> {
    let uuid = uuid::Uuid::parse_str(s)
        .map_err(|e| CoreError::InvalidInput(format!("invalid id: {e}")))?;
    Ok(Id::from_uuid(uuid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::action_proposal::ActionProposal;
    use crate::model::action_proposal::ProposalStatus;
    use crate::model::alarm::AlarmDefinition;
    use crate::model::check_in::CheckIn;
    use crate::model::check_in::CheckInType;
    use crate::model::checklist_run::ChecklistRun;
    use crate::model::checklist_template::ChecklistTemplate;
    use crate::model::context_document::ContextDocument;
    use crate::model::daily_log::DailyLog;
    use crate::model::habit::Habit;
    use crate::model::habit_event::HabitEvent;
    use crate::model::plan::Plan;
    use crate::model::reminder::ReminderDefinition;
    use crate::model::task_checkpoint::TaskCheckpoint;
    use crate::model::timer::TimerDefinition;
    use crate::time_provider::FakeTimeProvider;
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockStore {
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
    }

    impl MockStore {
        fn new() -> Self {
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
        fn get_alarm(
            &self,
            id: Id<AlarmDefinition>,
        ) -> Result<Option<AlarmDefinition>, Self::Error> {
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
        fn get_timer(
            &self,
            id: Id<TimerDefinition>,
        ) -> Result<Option<TimerDefinition>, Self::Error> {
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
        fn list_plan_items_by_log(
            &self,
            log_id: Id<DailyLog>,
        ) -> Result<Vec<PlanItem>, Self::Error> {
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
        fn get_checklist_run(
            &self,
            id: Id<ChecklistRun>,
        ) -> Result<Option<ChecklistRun>, Self::Error> {
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
        fn get_journal_entry(
            &self,
            id: Id<JournalEntry>,
        ) -> Result<Option<JournalEntry>, Self::Error> {
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
    }

    // ── TimeProvider integration tests ──────────

    #[test]
    fn service_with_time_returns_fixed_hour() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 14);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
        assert_eq!(service.time.hour(), 14);
        assert_eq!(
            service.time.today(),
            NaiveDate::from_ymd_opt(2026, 6, 10).unwrap()
        );
    }

    #[test]
    fn startup_state_uses_time_provider() {
        let store = MockStore::new();
        let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
        store.insert_daily_log(&log).unwrap();
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".to_string());
        store.insert_check_in(&ci).unwrap();
        let mut plan = Plan::new(log.id, "My Plan".into());
        plan.add_item("Task 1".into());
        store.insert_plan(&plan).unwrap();

        let time = FakeTimeProvider::new(2026, 6, 10, 22);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
        let view = service.get_startup_state().unwrap();
        assert_eq!(view.intent, "suggest_evening_shutdown");
    }

    #[test]
    fn current_activity_uses_time_provider() {
        let store = MockStore::new();
        let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
        store.insert_daily_log(&log).unwrap();
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".to_string());
        store.insert_check_in(&ci).unwrap();
        let mut plan = Plan::new(log.id, "My Plan".into());
        plan.add_item("Task 1".into());
        plan.items[0].status = crate::model::plan::PlanItemStatus::Done;
        store.insert_plan(&plan).unwrap();

        let time = FakeTimeProvider::new(2026, 6, 10, 22);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
        let activity = service.get_current_activity().unwrap();
        assert_eq!(activity.activity_kind, "shutdown");
    }

    // ── Checkpoint tests ───────────────────────

    #[test]
    fn calculate_next_checkpoint_chain() {
        let item = PlanItem::new(Id::<DailyLog>::new(), "test".into());

        // StartCheck + Started → ProgressCheck
        let next = AdiyutantCoreService::calculate_next_checkpoint(
            &item,
            &CheckpointKind::StartCheck,
            &CheckpointResponse::Started,
        );
        assert_eq!(next, Some(CheckpointKind::ProgressCheck));

        // ProgressCheck + Done → FinishCheck
        let next = AdiyutantCoreService::calculate_next_checkpoint(
            &item,
            &CheckpointKind::ProgressCheck,
            &CheckpointResponse::Done,
        );
        assert_eq!(next, Some(CheckpointKind::FinishCheck));

        // ProgressCheck + Move → RescheduleCheck
        let next = AdiyutantCoreService::calculate_next_checkpoint(
            &item,
            &CheckpointKind::ProgressCheck,
            &CheckpointResponse::Move,
        );
        assert_eq!(next, Some(CheckpointKind::RescheduleCheck));

        // FinishCheck + Done → None
        let next = AdiyutantCoreService::calculate_next_checkpoint(
            &item,
            &CheckpointKind::FinishCheck,
            &CheckpointResponse::Done,
        );
        assert_eq!(next, None);

        // RescheduleCheck + KeepWaiting → RelevanceReview
        let next = AdiyutantCoreService::calculate_next_checkpoint(
            &item,
            &CheckpointKind::RescheduleCheck,
            &CheckpointResponse::KeepWaiting,
        );
        assert_eq!(next, Some(CheckpointKind::RelevanceReview));

        // Unknown combo → None
        let next = AdiyutantCoreService::calculate_next_checkpoint(
            &item,
            &CheckpointKind::StartCheck,
            &CheckpointResponse::Done,
        );
        assert_eq!(next, None);
    }

    #[test]
    fn answer_checkpoint_state_machine() {
        let store = MockStore::new();
        let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
        store.insert_daily_log(&log).unwrap();
        let item = PlanItem::new(log.id, "test".into());
        store.insert_plan_item(&item).unwrap();
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::StartCheck);
        let cp_id = cp.id.value().to_string();
        store.insert_task_checkpoint(&cp).unwrap();

        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        // First call: Pending → Answered (directly, with response)
        let result = service.answer_checkpoint(&cp_id, "Started").unwrap();
        assert_eq!(result.title, "test");

        let cp = service
            .store
            .get_task_checkpoint(parse_id(&cp_id).unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(cp.status, CheckpointStatus::Answered);
        assert_eq!(cp.response, Some(CheckpointResponse::Started));
        assert!(cp.answered_at.is_some());

        // Second call: Answered → error
        let err = service.answer_checkpoint(&cp_id, "Started").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn get_pending_checkpoint_notification_returns_first_pending() {
        let store = MockStore::new();
        let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
        store.insert_daily_log(&log).unwrap();
        let item = PlanItem::new(log.id, "My Task".into());
        store.insert_plan_item(&item).unwrap();
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        store.insert_task_checkpoint(&cp).unwrap();

        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        let notification = service.get_pending_checkpoint_notification().unwrap();
        assert!(notification.is_some());
        let note = notification.unwrap();
        assert_eq!(note.source, "Checkpoint");
        assert!(note.title.contains("ProgressCheck"));
        assert!(note.body.contains("My Task"));
    }

    #[test]
    fn get_pending_checkpoint_notification_none_when_no_log() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
        let notification = service.get_pending_checkpoint_notification().unwrap();
        assert!(notification.is_none());
    }

    // ── Checklist tests ────────────────────────

    #[test]
    fn complete_checklist_run_adds_journal_entry() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        let log_id = service.get_or_create_today_log().unwrap();
        let mut template = ChecklistTemplate::new("Test".into(), "morning".into());
        service.store.insert_checklist_template(&template).unwrap();
        template = service
            .store
            .get_checklist_template(template.id)
            .unwrap()
            .unwrap();

        let run = ChecklistRun::new(template.id, log_id);
        let run_id = run.id.value().to_string();
        service.store.insert_checklist_run(&run).unwrap();

        let result = service.complete_checklist_run(&run_id).unwrap();
        assert!(
            !result.completed_at.is_empty(),
            "completed_at should be set"
        );
        assert_eq!(result.template_title, "Test");

        let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
        assert_eq!(journal.len(), 1);
        assert_eq!(journal[0].entry_type, JournalEntryType::ChecklistCompleted);
    }

    #[test]
    fn answer_checklist_item_on_completed_run_errors() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        let log_id = service.get_or_create_today_log().unwrap();
        let template = ChecklistTemplate::new("Test".into(), "morning".into());
        service.store.insert_checklist_template(&template).unwrap();
        let mut run = ChecklistRun::new(template.id, log_id);
        run.completed_at = Some(AdiyutantDateTime::from_utc(service.time.now_utc()));
        let run_id = run.id.value().to_string();
        service.store.insert_checklist_run(&run).unwrap();

        let err = service
            .answer_checklist_item(&run_id, "any", "yes", None)
            .unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    // ── Journal auto-event tests ────────────────

    #[test]
    fn start_plan_item_creates_journal_entry() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        let log_id = service.get_or_create_today_log().unwrap();
        let item = PlanItem::new(log_id, "Test Task".into());
        let item_id = item.id.value().to_string();
        service.store.insert_plan_item(&item).unwrap();

        service.start_plan_item(&item_id).unwrap();
        let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
        assert!(
            journal
                .iter()
                .any(|e| e.entry_type == JournalEntryType::TaskStarted)
        );
    }

    #[test]
    fn done_plan_item_creates_journal_entry() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        let log_id = service.get_or_create_today_log().unwrap();
        let item = PlanItem::new(log_id, "Test Task".into());
        let item_id = item.id.value().to_string();
        service.store.insert_plan_item(&item).unwrap();

        service.done_plan_item(&item_id).unwrap();
        let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
        assert!(
            journal
                .iter()
                .any(|e| e.entry_type == JournalEntryType::TaskDone)
        );
    }

    #[test]
    fn move_to_waiting_creates_journal_entry() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        let log_id = service.get_or_create_today_log().unwrap();
        let item = PlanItem::new(log_id, "Test Task".into());
        let item_id = item.id.value().to_string();
        service.store.insert_plan_item(&item).unwrap();

        service.move_to_waiting(&item_id, None).unwrap();
        let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
        assert!(
            journal
                .iter()
                .any(|e| e.entry_type == JournalEntryType::TaskMoved)
        );
    }

    // ── Integration tests ──

    #[test]
    fn get_today_works_with_fake_time() {
        let store = MockStore::new();
        let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
        store.insert_daily_log(&log).unwrap();

        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
        let today = service.get_today().unwrap();
        assert_eq!(today.date, "2026-06-10");
    }

    #[test]
    fn new_service_uses_real_time_provider() {
        let store = MockStore::new();
        let service = AdiyutantCoreService::new(Box::new(store));
        let _hour = service.time.hour();
    }

    #[test]
    fn add_plan_item_creates_start_check_checkpoint() {
        let store = MockStore::new();
        let time = FakeTimeProvider::new(2026, 6, 10, 10);
        let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

        let dto = service.add_plan_item("Test", None, None).unwrap();
        assert_eq!(dto.title, "Test");

        let log_id = service.get_or_create_today_log().unwrap();
        let items = service.store.list_plan_items_by_log(log_id).unwrap();
        assert_eq!(items.len(), 1);
        let checkpoints = service
            .store
            .list_checkpoints_by_plan_item(items[0].id)
            .unwrap();
        assert!(!checkpoints.is_empty());
        assert_eq!(checkpoints[0].kind, CheckpointKind::StartCheck);
    }
}
