use crate::current_activity::CurrentActivity;
use crate::datetime::AdiyutantDateTime;
use crate::dto::{
    self, ChecklistRunDto, ChecklistTemplateDto, CurrentActivityDto, DayPlanDto, JournalEntryDto,
    PlanItemDto, StartupViewDto, TodayViewDto, WaitingTaskDto,
};
use crate::error::CoreError;
use crate::id::Id;
use crate::local_rule_gateway::LocalRuleAgentGateway;
use crate::model::checklist_run::ChecklistRun;
use crate::model::checklist_template::ChecklistTemplate;

use crate::model::daily_log::DailyLog;
use crate::model::day_plan::{EisenhowerQuadrant, PlanItem, PlanItemStatus, WaitingDecision};
use crate::model::habit::Habit;
use crate::model::habit_event::HabitEvent;
use crate::model::journal_entry::{JournalEntry, JournalEntryType};
use crate::model::task_checkpoint::{CheckpointKind, TaskCheckpoint};
use crate::startup::StartupState;
use crate::store::Store;
use crate::today_state::{TodayState, TodayStateBuilder};

/// Application-layer facade / use-case boundary.
///
/// All public methods return stable DTOs. Consumers (CLI, Android)
/// must never access Store or domain internals directly.
pub struct AdiyutantCoreService {
    store: Box<dyn Store<Error = CoreError>>,
}

impl AdiyutantCoreService {
    pub fn new(store: Box<dyn Store<Error = CoreError>>) -> Self {
        Self { store }
    }

    /// Build a `TodayState` snapshot from the store.
    fn build_today_state(&self) -> Result<TodayState, CoreError> {
        let today = chrono::Utc::now().date_naive();
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
                .unwrap_or_default(),
            plan_item_count: state.plan.as_ref().map_or(0, |p| p.items.len()),
            plan_items: state
                .plan
                .as_ref()
                .map(|p| {
                    p.items
                        .iter()
                        .map(|item| dto::PlanItemViewDto {
                            description: item.description.clone(),
                            status: format!("{:?}", item.status),
                            estimated_minutes: item
                                .estimated_minutes
                                .map(|m| m.to_string())
                                .unwrap_or_default(),
                            notes: dto::option_string(&item.notes),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            suggestion_count: {
                let rule_result = LocalRuleAgentGateway::evaluate(&state);
                rule_result.proposals.len()
            },
        })
    }

    /// What to show / suggest on launch.
    pub fn get_startup_state(&self) -> Result<StartupViewDto, CoreError> {
        let state = self.build_today_state()?;
        let startup = StartupState::determine(&state);
        Ok(StartupViewDto {
            intent: startup.intent.as_str().to_string(),
            reason: startup.reason,
        })
    }

    /// What the user is (or should be) doing right now.
    pub fn get_current_activity(&self) -> Result<CurrentActivityDto, CoreError> {
        let state = self.build_today_state()?;
        let activity = CurrentActivity::determine(&state);
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
        let today = chrono::Utc::now().date_naive();
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
        let today = chrono::Utc::now().date_naive();
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
        item.updated_at = AdiyutantDateTime::now();

        // Create ProgressCheck checkpoint
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        let _ = self.store.insert_task_checkpoint(&cp);

        self.store.update_plan_item(&item)?;
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
        item.updated_at = crate::datetime::AdiyutantDateTime::now();

        self.store.update_plan_item(&item)?;
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
        item.waiting_review_at = Some(
            chrono::Utc::now().date_naive()
                + chrono::Duration::days(review_days.unwrap_or(7) as i64),
        );
        item.updated_at = crate::datetime::AdiyutantDateTime::now();

        self.store.update_plan_item(&item)?;
        Ok(plan_item_to_dto(&item))
    }

    /// List waiting items due for review.
    pub fn list_waiting_tasks(&self) -> Result<Vec<WaitingTaskDto>, CoreError> {
        let today = chrono::Utc::now().date_naive();
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
                item.waiting_review_at =
                    Some(chrono::Utc::now().date_naive() + chrono::Duration::days(7));
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
        item.updated_at = AdiyutantDateTime::now();

        self.store.update_plan_item(&item)?;
        Ok(plan_item_to_dto(&item))
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

    // ── Journal Domain ─────────────────────────

    /// List journal entries for today.
    pub fn get_journal(&self) -> Result<Vec<JournalEntryDto>, CoreError> {
        let today = chrono::Utc::now().date_naive();
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

    /// Create a check-in and optionally update daily log metrics.
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
        self.store.insert_check_in(&ci)?;

        if (sleep_score.is_some() || energy.is_some() || mood.is_some())
            && let Some(log) = self.store.get_daily_log(daily_log_id)?
        {
            let mut updated = log.clone();
            if let Some(s) = sleep_score {
                updated.sleep_score = Some(s);
            }
            if let Some(e) = energy {
                updated.energy = Some(e);
            }
            if let Some(m) = mood {
                updated.mood = Some(m);
            }
            self.store.update_daily_log(&updated)?;
        }

        // Auto-journal entry
        let entry = JournalEntry::new(
            daily_log_id,
            JournalEntryType::CheckInCreated,
            format!("{checkin_type} check-in: {text}"),
        );
        let _ = self.store.insert_journal_entry(&entry);

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
