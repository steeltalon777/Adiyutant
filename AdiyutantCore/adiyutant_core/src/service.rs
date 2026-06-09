use crate::current_activity::CurrentActivity;
use crate::dto::{self, CurrentActivityDto, StartupViewDto, TodayViewDto};
use crate::error::CoreError;
use crate::local_rule_gateway::LocalRuleAgentGateway;
use crate::model::habit_event::HabitEvent;
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
}
