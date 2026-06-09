use crate::model::action_proposal::{ActionProposal, ProposalSource};
use crate::today_state::TodayState;
use serde_json::json;

/// Local rule engine without LLM.
///
/// Analyses a `TodayState` and generates `ActionProposal` items
/// based on simple human-readable rules.
#[derive(Debug)]
pub struct LocalRuleAgentGateway;

/// Result of evaluating all rules against a `TodayState`.
#[derive(Debug)]
pub struct RuleResult {
    pub proposals: Vec<ActionProposal>,
}

impl LocalRuleAgentGateway {
    /// Evaluate all rules against the given `TodayState`.
    ///
    /// Returns a list of `ActionProposal` items, each with
    /// `ProposalSource::LocalRule`.
    pub fn evaluate(state: &TodayState) -> RuleResult {
        let mut proposals = Vec::new();
        let mut push = |pt: &str, payload: serde_json::Value| {
            proposals.push(ActionProposal::new(
                ProposalSource::LocalRule,
                pt.to_string(),
                payload,
            ));
        };

        // Rule 1: No morning check-in → suggest one
        if state.morning_check_in().is_none() {
            push(
                "morning_checkin",
                json!({
                    "reason": "No morning check-in yet today",
                    "suggestion": "Do a morning check-in to start the day"
                }),
            );
        }

        // Rule 2: Low sleep score → suggest recovery mode
        if let Some(score) = state.daily_log.as_ref().and_then(|log| log.sleep_score)
            && score < 6
        {
            push(
                "set_recovery_mode",
                json!({
                    "reason": "Low sleep score detected",
                    "sleep_score": score,
                    "suggestion": "Consider setting day mode to recovery"
                }),
            );
        }

        // Rule 3: No plan for today → suggest creating one
        if !state.has_plan() {
            push(
                "create_plan",
                json!({
                    "reason": "No plan for today yet",
                    "suggestion": "Create a plan to organize the day"
                }),
            );
        }

        // Rule 4: Active habits exist but no events logged today → suggest doing one
        if !state.habits.is_empty() && state.habit_events.is_empty() {
            push(
                "do_habit",
                json!({
                    "reason": "Active habits exist but no events logged today",
                    "habit_count": state.habits.len(),
                    "suggestion": "Log at least one habit event today"
                }),
            );
        }

        // Rule 5: Evening approaching with pending plan items → suggest evening review
        if state.evening_check_in().is_none() && state.pending_plan_items() > 0 {
            push(
                "evening_checkin",
                json!({
                    "reason": "Evening approaching with pending plan items",
                    "pending_items": state.pending_plan_items(),
                    "suggestion": "Review the day and do an evening check-in"
                }),
            );
        }

        // Rule 6: Late hour and no shutdown → suggest shutdown
        if state.evening_check_in().is_none() {
            let now = chrono::Utc::now();
            let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(0);
            if !(6..20).contains(&hour) {
                push(
                    "shutdown",
                    json!({
                        "reason": "It's late and no shutdown check-in done",
                        "suggestion": "Do a shutdown check-in to close the day"
                    }),
                );
            }
        }

        RuleResult { proposals }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::check_in::CheckIn;
    use crate::model::check_in::CheckInType;
    use crate::model::daily_log::DailyLog;
    use crate::model::habit::Habit;
    use crate::today_state::TodayStateBuilder;

    #[test]
    fn no_morning_checkin_proposes_morning_checkin() {
        let state = TodayStateBuilder::new()
            .date(chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap())
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(
            result
                .proposals
                .iter()
                .any(|p| p.proposal_type == "morning_checkin")
        );
    }

    #[test]
    fn low_sleep_proposes_recovery_mode() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let mut log = DailyLog::new(date);
        log.sleep_score = Some(4);
        let state = TodayStateBuilder::new().date(date).daily_log(log).build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(
            result
                .proposals
                .iter()
                .any(|p| p.proposal_type == "set_recovery_mode")
        );
    }

    #[test]
    fn no_plan_proposes_create_plan() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let state = TodayStateBuilder::new().date(date).build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(
            result
                .proposals
                .iter()
                .any(|p| p.proposal_type == "create_plan")
        );
    }

    #[test]
    fn habits_without_events_proposes_do_habit() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let habit = Habit::new("read".into());
        let state = TodayStateBuilder::new().date(date).habit(habit).build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(
            result
                .proposals
                .iter()
                .any(|p| p.proposal_type == "do_habit")
        );
    }

    #[test]
    fn morning_checkin_present_skips_morning_proposal() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log = DailyLog::new(date);
        let checkin = CheckIn::new(log.id, CheckInType::Morning, "good morning".into());
        let state = TodayStateBuilder::new()
            .date(date)
            .daily_log(log)
            .check_in(checkin)
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(
            !result
                .proposals
                .iter()
                .any(|p| p.proposal_type == "morning_checkin")
        );
    }

    #[test]
    fn normal_sleep_does_not_propose_recovery() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let mut log = DailyLog::new(date);
        log.sleep_score = Some(8);
        let state = TodayStateBuilder::new().date(date).daily_log(log).build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(
            !result
                .proposals
                .iter()
                .any(|p| p.proposal_type == "set_recovery_mode")
        );
    }

    #[test]
    fn all_proposals_have_local_rule_source() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let state = TodayStateBuilder::new().date(date).build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        for p in &result.proposals {
            assert_eq!(
                p.source,
                ProposalSource::LocalRule,
                "proposal {} should be LocalRule",
                p.proposal_type
            );
        }
    }

    #[test]
    fn habits_with_events_skips_do_habit() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let habit = Habit::new("read".into());
        let he = crate::model::habit_event::HabitEvent::new(
            habit.id,
            crate::model::habit_event::HabitEventStatus::Done,
            crate::model::habit_event::HabitEventLevel::Base,
        );
        let state = TodayStateBuilder::new()
            .date(date)
            .habit(habit)
            .habit_event(he)
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(
            !result
                .proposals
                .iter()
                .any(|p| p.proposal_type == "do_habit")
        );
    }
}
