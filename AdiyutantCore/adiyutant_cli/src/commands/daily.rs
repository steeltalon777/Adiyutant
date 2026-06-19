use adiyutant_core::dto::TodayViewDto;
use adiyutant_core::service::AdiyutantCoreService;

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
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckinTypeArg::Morning => "morning",
            CheckinTypeArg::Day => "day",
            CheckinTypeArg::Evening => "evening",
            CheckinTypeArg::Shutdown => "shutdown",
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

/// `adiyutant log` — show raw daily log (direct store access acceptable for debug).
pub fn cmd_log(store: &adiyutant_store::SqliteStore) {
    let today = chrono::Utc::now().date_naive();
    use adiyutant_core::store::Store;
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

/// `adiyutant checkin <type> <text>` — create a check-in via facade.
pub fn cmd_checkin(facade: &AdiyutantCoreService, args: &CheckinArgs) {
    // Parse structured data from text: sleep=N, energy=N, mood=N
    let (text, sleep_score, energy, mood) = parse_metrics(&args.text);

    match facade.create_checkin(args.checkin_type.as_str(), &text, sleep_score, energy, mood) {
        Ok(_dto) => {
            let type_name = args.checkin_type.as_str();
            println!("✅ {type_name} check-in recorded.");
            if sleep_score.is_some() || energy.is_some() || mood.is_some() {
                println!("   Numeric values extracted and saved.");
            }
        }
        Err(e) => {
            eprintln!("Error recording check-in: {e}");
            std::process::exit(1);
        }
    }
}

/// Extract sleep=, energy=, mood= from the check-in text.
fn parse_metrics(text: &str) -> (String, Option<u8>, Option<u8>, Option<u8>) {
    let mut clean = text.to_string();
    let mut sleep_score: Option<u8> = None;
    let mut energy: Option<u8> = None;
    let mut mood: Option<u8> = None;

    for word in text.split_whitespace() {
        if let Some(rest) = word.strip_prefix("sleep=") {
            sleep_score = rest.parse::<u8>().ok();
            clean = clean.replace(word, "").trim().to_string();
        } else if let Some(rest) = word.strip_prefix("energy=") {
            energy = rest.parse::<u8>().ok();
            clean = clean.replace(word, "").trim().to_string();
        } else if let Some(rest) = word.strip_prefix("mood=") {
            mood = rest.parse::<u8>().ok();
            clean = clean.replace(word, "").trim().to_string();
        }
    }

    (clean, sleep_score, energy, mood)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkin_type_arg_as_str() {
        assert_eq!(CheckinTypeArg::Morning.as_str(), "morning");
        assert_eq!(CheckinTypeArg::Day.as_str(), "day");
        assert_eq!(CheckinTypeArg::Evening.as_str(), "evening");
        assert_eq!(CheckinTypeArg::Shutdown.as_str(), "shutdown");
    }

    #[test]
    fn parse_metrics_extracts_values() {
        let (text, sleep, energy, mood) = parse_metrics("gm sleep=7 energy=6 mood=8");
        assert_eq!(text, "gm");
        assert_eq!(sleep, Some(7));
        assert_eq!(energy, Some(6));
        assert_eq!(mood, Some(8));
    }

    #[test]
    fn parse_metrics_no_metrics() {
        let (text, sleep, energy, mood) = parse_metrics("hello world");
        assert_eq!(text, "hello world");
        assert!(sleep.is_none());
        assert!(energy.is_none());
        assert!(mood.is_none());
    }
}
