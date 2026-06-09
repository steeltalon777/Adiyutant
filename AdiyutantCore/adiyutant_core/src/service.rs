use crate::current_activity::CurrentActivity;
use crate::datetime::AdiyutantDateTime;
use crate::dto::{
    self, CurrentActivityDto, DayPlanDto, PlanItemDto, StartupViewDto, TodayViewDto, WaitingTaskDto,
};
use crate::error::CoreError;
use crate::id::Id;
use crate::local_rule_gateway::LocalRuleAgentGateway;
use crate::model::daily_log::DailyLog;
use crate::model::day_plan::{EisenhowerQuadrant, PlanItem, PlanItemStatus, WaitingDecision};
use crate::model::habit_event::HabitEvent;
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
