use adiyutant_core::dto::TodayViewDto;
use adiyutant_core::model::check_in::{CheckIn, CheckInType};
use adiyutant_core::model::daily_log::DailyLog;
use adiyutant_core::service::AdiyutantCoreService;
use adiyutant_core::store::Store;

/// Arguments for the `checkin` subcommand.
#[derive(clap::Args, Debug, Clone)]
pub struct CheckinArgs {
    /// Check-in type: morning, day, evening, shutdown
    #[arg(value_enum)]
    pub checkin_type: CheckinTypeArg,

    /// Free-form text for the check-in
    pub text: String,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum CheckinTypeArg {
    Morning,
    Day,
    Evening,
    Shutdown,
}

impl CheckinTypeArg {
    fn to_domain(&self) -> CheckInType {
        match self {
            CheckinTypeArg::Morning => CheckInType::Morning,
            CheckinTypeArg::Day => CheckInType::Day,
            CheckinTypeArg::Evening => CheckInType::Evening,
            CheckinTypeArg::Shutdown => CheckInType::Shutdown,
        }
    }
}

/// Print a human-readable summary of `TodayViewDto`.
pub fn print_today_view(today: &TodayViewDto) {
    println!("── Today: {} ──", today.date);
    println!("Mode: {}", today.mode);
    println!(
        "Sleep: {} | Energy: {} | Mood: {}",
        today.sleep_score, today.energy, today.mood
    );
    println!(
        "Check-ins: {} (morning: {}, evening: {})",
        today.check_in_count,
        if today.has_morning_check_in {
            "yes"
        } else {
            "no"
        },
        if today.has_evening_check_in {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "Habits: {} tracked, {} done today",
        today.habit_count, today.habit_events_done
    );
    if today.has_plan {
        println!(
            "Plan: {} ({} items)",
            today.plan_title, today.plan_item_count
        );
        for item in &today.plan_items {
            println!("  • {} [{}]", item.description, item.status);
        }
    } else {
        println!("Plan: no plan");
    }
    println!("Suggestions: {}", today.suggestion_count);
}

/// `adiyutant today` — show today's state via facade.
pub fn cmd_today(facade: &AdiyutantCoreService) {
    match facade.get_today() {
        Ok(view) => print_today_view(&view),
        Err(e) => {
            eprintln!("Error reading today's state: {e}");
            std::process::exit(1);
        }
    }
}

/// `adiyutant log` — show raw daily log.
pub fn cmd_log(store: &adiyutant_store::SqliteStore) {
    let today = chrono::Utc::now().date_naive();

    match store.get_daily_log_by_date(today) {
        Ok(Some(log)) => {
            println!("Date: {}", log.date);
            println!("Mode: {}", log.mode.as_deref().unwrap_or("none"));
            println!(
                "Sleep: {} | Energy: {} | Mood: {}",
                log.sleep_score
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or("-"),
                log.energy.map(|e| e.to_string()).as_deref().unwrap_or("-"),
                log.mood.map(|m| m.to_string()).as_deref().unwrap_or("-"),
            );
            println!("Notes: {}", log.raw_notes.as_deref().unwrap_or("<none>"));
        }
        Ok(None) => {
            println!("No daily log for today. Use `adiyutant checkin morning` to start.");
        }
        Err(e) => {
            eprintln!("Error reading daily log: {e}");
            std::process::exit(1);
        }
    }
}

/// `adiyutant checkin <type> <text>` — create a check-in.
pub fn cmd_checkin(store: &adiyutant_store::SqliteStore, args: &CheckinArgs) {
    let today = chrono::Utc::now().date_naive();

    // Get or create DailyLog for today
    let daily_log = match store.get_daily_log_by_date(today) {
        Ok(Some(log)) => log,
        Ok(None) => {
            let new_log = DailyLog::new(today);
            if let Err(e) = store.insert_daily_log(&new_log) {
                eprintln!("Error creating daily log: {e}");
                std::process::exit(1);
            }
            new_log
        }
        Err(e) => {
            eprintln!("Error reading daily log: {e}");
            std::process::exit(1);
        }
    };

    // Parse text for structured data hints
    let mut energy: Option<u8> = None;
    let mut mood: Option<u8> = None;
    let mut sleep_score: Option<u8> = None;

    for word in args.text.split_whitespace() {
        if let Some(rest) = word.strip_prefix("sleep=") {
            sleep_score = rest.parse::<u8>().ok();
        } else if let Some(rest) = word.strip_prefix("energy=") {
            energy = rest.parse::<u8>().ok();
        } else if let Some(rest) = word.strip_prefix("mood=") {
            mood = rest.parse::<u8>().ok();
        }
    }

    let ci = CheckIn::new(
        daily_log.id,
        args.checkin_type.to_domain(),
        args.text.clone(),
    );
    if let Err(e) = store.insert_check_in(&ci) {
        eprintln!("Error recording check-in: {e}");
        std::process::exit(1);
    }

    // Update daily log with any parsed values
    let mut needs_update = false;
    if (energy.is_some() || mood.is_some() || sleep_score.is_some())
        && let Some(log) = store.get_daily_log_by_date(today).ok().flatten()
    {
        let mut updated = log.clone();
        if let Some(e) = energy {
            updated.energy = Some(e);
        }
        if let Some(m) = mood {
            updated.mood = Some(m);
        }
        if let Some(s) = sleep_score {
            updated.sleep_score = Some(s);
        }
        if updated.energy != log.energy
            || updated.mood != log.mood
            || updated.sleep_score != log.sleep_score
        {
            let _ = store.update_daily_log(&updated);
            needs_update = true;
        }
    }

    let type_name = format!("{:?}", args.checkin_type).to_lowercase();
    println!("✅ {type_name} check-in recorded.");
    if needs_update {
        println!("   Numeric values extracted and saved.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adiyutant_core::model::check_in::CheckInType;

    #[test]
    fn checkin_type_arg_to_domain() {
        assert_eq!(CheckinTypeArg::Morning.to_domain(), CheckInType::Morning);
        assert_eq!(CheckinTypeArg::Day.to_domain(), CheckInType::Day);
        assert_eq!(CheckinTypeArg::Evening.to_domain(), CheckInType::Evening);
        assert_eq!(CheckinTypeArg::Shutdown.to_domain(), CheckInType::Shutdown);
    }
}
