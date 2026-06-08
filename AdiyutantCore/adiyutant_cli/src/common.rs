use std::path::PathBuf;

use adiyutant_core::model::habit_event::HabitEvent;
use adiyutant_core::today_state::{TodayState, TodayStateBuilder};
use adiyutant_store::{SqliteStore, Store};

/// Determine the SQLite database path.
///
/// Priority:
/// 1. `ADIYUTANT_DB_PATH` environment variable
/// 2. `~/.adiyutant/adiyutant.db` (fallback)
pub fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("ADIYUTANT_DB_PATH") {
        return PathBuf::from(p);
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".adiyutant").join("adiyutant.db")
}

/// Initialize the `SqliteStore`: create directory, open DB, run migration.
pub fn init_store() -> Result<SqliteStore, adiyutant_core::error::CoreError> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| adiyutant_core::error::CoreError::Storage(e.to_string()))?;
    }
    let store = SqliteStore::new(path.to_str().unwrap_or("adiyutant.db"))?;
    store.migrate()?;
    Ok(store)
}

/// Collect all today's data from the store and build a `TodayState`.
///
/// This is shared between `today` and `suggest` commands.
pub fn build_today_state(
    store: &SqliteStore,
) -> Result<TodayState, adiyutant_core::error::CoreError> {
    let today = chrono::Utc::now().date_naive();

    let mut builder = TodayStateBuilder::new().date(today);

    // DailyLog
    if let Some(log) = store.get_daily_log_by_date(today)? {
        builder = builder.daily_log(log.clone());

        // Check-ins for this log
        let check_ins = store.list_check_ins_by_log(log.id)?;
        builder = builder.check_ins(check_ins);

        // Plan for this log
        if let Some(plan) = store.get_plan_by_daily_log(log.id)? {
            builder = builder.plan(plan);
        }
    }

    // Habits
    let habits = store.list_habits()?;
    builder = builder.habits(habits.clone());

    // Habit events for each habit — filter to today only
    for habit in &habits {
        let events = store.list_habit_events_by_habit(habit.id)?;
        let today_events: Vec<HabitEvent> = events
            .into_iter()
            .filter(|e| {
                // Compare by date via chrono::NaiveDate
                let event_date = e.date_time.inner().date_naive();
                event_date == today
            })
            .collect();
        for ev in today_events {
            builder = builder.habit_event(ev);
        }
    }

    // Timers, reminders, alarms
    let timers = store.list_timers()?;
    builder = builder.timers(timers);

    let reminders = store.list_reminders()?;
    builder = builder.reminders(reminders);

    let alarms = store.list_alarms()?;
    builder = builder.alarms(alarms);

    // Context documents
    let context_docs = store.list_context_documents()?;
    builder = builder.context_documents(context_docs);

    Ok(builder.build())
}

/// Print a human-readable summary of `TodayState`.
pub fn print_today_state(state: &TodayState) {
    let date_str = state.date.format("%Y-%m-%d").to_string();
    println!("── Today: {date_str} ──");

    // Mode / sleep / energy / mood
    let mode = state
        .daily_log
        .as_ref()
        .and_then(|l| l.mode.as_deref())
        .unwrap_or("none");
    let sleep = state
        .daily_log
        .as_ref()
        .and_then(|l| l.sleep_score.map(|s| s.to_string()))
        .unwrap_or_else(|| "-".to_string());
    let energy = state
        .daily_log
        .as_ref()
        .and_then(|l| l.energy.map(|e| e.to_string()))
        .unwrap_or_else(|| "-".to_string());
    let mood = state
        .daily_log
        .as_ref()
        .and_then(|l| l.mood.map(|m| m.to_string()))
        .unwrap_or_else(|| "-".to_string());

    println!("Mode: {mode}");
    println!("Sleep: {sleep} | Energy: {energy} | Mood: {mood}");

    // Check-ins
    let ci_count = state.check_ins.len();
    let has_morning = state.morning_check_in().is_some();
    let has_evening = state.evening_check_in().is_some();
    println!(
        "Check-ins: {ci_count} (morning: {}, evening: {})",
        if has_morning { "yes" } else { "no" },
        if has_evening { "yes" } else { "no" }
    );

    // Habits
    let habit_count = state.habits.len();
    let done_count = state.habit_event_count();
    println!("Habits: {habit_count} tracked, {done_count} done today");

    // Plan
    if let Some(plan) = &state.plan {
        println!("Plan: {} ({} items)", plan.title, plan.items.len());
        for item in &plan.items {
            let status = if item.status == adiyutant_core::model::plan::PlanItemStatus::Done {
                "✓"
            } else {
                "•"
            };
            println!("  {status} {desc}", desc = item.description);
        }
    } else {
        println!("Plan: no plan");
    }

    // Alarms / Reminders / Timers
    println!(
        "Alarms: {} | Reminders: {} | Timers: {}",
        state.alarms.len(),
        state.reminders.len(),
        state.timers.len()
    );

    // Context documents
    println!("Context documents: {}", state.context_documents.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_path_uses_env_var() {
        // SAFETY: test-only env var modification
        unsafe {
            std::env::set_var("ADIYUTANT_DB_PATH", "/tmp/test.db");
        }
        let path = db_path();
        assert_eq!(path, PathBuf::from("/tmp/test.db"));
        // SAFETY: test-only env var removal
        unsafe {
            std::env::remove_var("ADIYUTANT_DB_PATH");
        }
    }

    #[test]
    fn db_path_falls_back_to_home() {
        // SAFETY: test-only env var removal
        unsafe {
            std::env::remove_var("ADIYUTANT_DB_PATH");
        }
        let path = db_path();
        // Should contain ".adiyutant/adiyutant.db"
        assert!(path.to_string_lossy().contains(".adiyutant/adiyutant.db"));
    }
}
