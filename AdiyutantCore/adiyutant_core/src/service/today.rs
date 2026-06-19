use super::*;
use crate::current_activity::CurrentActivity;
use crate::dto::{CurrentActivityDto, StartupViewDto, TodayViewDto};
use crate::local_rule_gateway::LocalRuleAgentGateway;
use crate::startup::StartupState;
use crate::today_state::{TodayState, TodayStateBuilder};

impl AdiyutantCoreService {
    /// Build a `TodayState` snapshot from the store.
    pub(crate) fn build_today_state(&self) -> Result<TodayState, crate::error::CoreError> {
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
            let today_events: Vec<crate::model::habit_event::HabitEvent> = events
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
    pub fn get_today(&self) -> Result<TodayViewDto, crate::error::CoreError> {
        let state = self.build_today_state()?;
        let legacy_item_count = state.plan.as_ref().map_or(0, |p| p.items.len());
        let new_item_count = state.plan_items.len();

        Ok(TodayViewDto {
            date: crate::dto::naive_date_to_string(state.date),
            has_daily_log: state.daily_log.is_some(),
            mode: state
                .daily_log
                .as_ref()
                .and_then(|l| l.mode.as_deref())
                .unwrap_or("none")
                .to_string(),
            sleep_score: crate::dto::option_u8_string(
                &state.daily_log.as_ref().and_then(|l| l.sleep_score),
            ),
            energy: crate::dto::option_u8_string(&state.daily_log.as_ref().and_then(|l| l.energy)),
            mood: crate::dto::option_u8_string(&state.daily_log.as_ref().and_then(|l| l.mood)),
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
                let mut items: Vec<crate::dto::PlanItemViewDto> = Vec::new();
                // New plan_items
                for pi in &state.plan_items {
                    items.push(crate::dto::PlanItemViewDto {
                        description: pi.title.clone(),
                        status: pi.status.as_str().to_string(),
                        estimated_minutes: String::new(),
                        notes: crate::dto::option_string(&pi.description),
                    });
                }
                // Legacy plan items
                if let Some(plan) = &state.plan {
                    for item in &plan.items {
                        items.push(crate::dto::PlanItemViewDto {
                            description: item.description.clone(),
                            status: format!("{:?}", item.status),
                            estimated_minutes: item
                                .estimated_minutes
                                .map(|m| m.to_string())
                                .unwrap_or_default(),
                            notes: crate::dto::option_string(&item.notes),
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
    pub fn get_startup_state(&self) -> Result<StartupViewDto, crate::error::CoreError> {
        let state = self.build_today_state()?;
        let startup = StartupState::determine_at_hour(&state, self.time.hour());
        Ok(StartupViewDto {
            intent: startup.intent.as_str().to_string(),
            reason: startup.reason,
        })
    }

    /// What the user is (or should be) doing right now.
    pub fn get_current_activity(&self) -> Result<CurrentActivityDto, crate::error::CoreError> {
        let state = self.build_today_state()?;
        let activity = CurrentActivity::determine_at_hour(&state, self.time.hour());
        Ok(CurrentActivityDto {
            date: crate::dto::naive_date_to_string(state.date),
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
