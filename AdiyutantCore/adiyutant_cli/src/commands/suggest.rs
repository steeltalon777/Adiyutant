use adiyutant_core::local_rule_gateway::LocalRuleAgentGateway;
use adiyutant_core::store::Store;

/// `adiyutant suggest` — show local suggestions based on today's state.
pub fn cmd_suggest(store: &adiyutant_store::SqliteStore) {
    // Build TodayState directly from store (this is the only remaining direct store usage for suggest)
    let state = match build_raw_today_state(store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error building today's state: {e}");
            std::process::exit(1);
        }
    };

    let result = LocalRuleAgentGateway::evaluate(&state);

    if result.proposals.is_empty() {
        println!("💡 No suggestions — everything looks good!");
        return;
    }

    println!("💡 Suggestions based on today's state:");
    for proposal in &result.proposals {
        let reason = proposal
            .payload_json
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("no reason");
        let suggestion = proposal
            .payload_json
            .get("suggestion")
            .and_then(|v| v.as_str())
            .unwrap_or("no suggestion");

        println!("  • {}: {}", proposal.proposal_type, reason);
        println!("    → {suggestion}");
    }
}

/// Build a raw TodayState from the store (used by suggest command).
fn build_raw_today_state(
    store: &adiyutant_store::SqliteStore,
) -> Result<adiyutant_core::today_state::TodayState, adiyutant_core::error::CoreError> {
    use adiyutant_core::model::habit_event::HabitEvent;
    use adiyutant_core::today_state::TodayStateBuilder;

    let today = chrono::Utc::now().date_naive();
    let mut builder = TodayStateBuilder::new().date(today);

    if let Some(log) = store.get_daily_log_by_date(today)? {
        builder = builder.daily_log(log.clone());
        let check_ins = store.list_check_ins_by_log(log.id)?;
        builder = builder.check_ins(check_ins);
        if let Some(plan) = store.get_plan_by_daily_log(log.id)? {
            builder = builder.plan(plan);
        }
    }

    let habits = store.list_habits()?;
    builder = builder.habits(habits.clone());

    for habit in &habits {
        let events = store.list_habit_events_by_habit(habit.id)?;
        let today_events: Vec<HabitEvent> = events
            .into_iter()
            .filter(|e| e.date_time.inner().date_naive() == today)
            .collect();
        for ev in today_events {
            builder = builder.habit_event(ev);
        }
    }

    builder = builder.timers(store.list_timers()?);
    builder = builder.reminders(store.list_reminders()?);
    builder = builder.alarms(store.list_alarms()?);
    builder = builder.context_documents(store.list_context_documents()?);

    Ok(builder.build())
}
