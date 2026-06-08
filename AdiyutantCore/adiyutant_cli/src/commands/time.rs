use adiyutant_core::model::alarm::AlarmDefinition;
use adiyutant_core::model::reminder::ReminderDefinition;
use adiyutant_core::model::timer::{TimerDefinition, TimerMode};
use adiyutant_store::Store;
use chrono::NaiveTime;

#[derive(clap::Subcommand, Debug)]
pub enum TimerCmd {
    /// Start a timer (save definition)
    Start {
        /// Timer name
        name: String,
        /// Duration in minutes
        #[arg(short, long, default_value = "25")]
        minutes: u64,
        /// Timer mode: focus, rest, custom
        #[arg(long, default_value = "focus")]
        mode: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum ReminderCmd {
    /// Add a reminder
    Add {
        /// Reminder title
        title: String,
        /// Schedule rule (e.g. "21:00", "every 2h")
        #[arg(short, long)]
        at: String,
    },
    /// List reminders
    List,
}

#[derive(clap::Subcommand, Debug)]
pub enum AlarmCmd {
    /// Add an alarm
    Add {
        /// Alarm title
        title: String,
        /// Time in HH:MM format
        #[arg(short, long)]
        at: String,
    },
    /// List alarms
    List,
}

fn parse_timer_mode(s: &str) -> TimerMode {
    match s.to_lowercase().as_str() {
        "focus" => TimerMode::Focus,
        "rest" => TimerMode::Rest,
        _ => TimerMode::Custom,
    }
}

// ── Timer ──

pub fn handle_timer(store: &adiyutant_store::SqliteStore, cmd: &TimerCmd) {
    match cmd {
        TimerCmd::Start {
            name,
            minutes,
            mode,
        } => {
            cmd_timer_start(store, name, *minutes, mode);
        }
    }
}

fn cmd_timer_start(store: &adiyutant_store::SqliteStore, name: &str, minutes: u64, mode_str: &str) {
    let seconds = minutes * 60;
    let mode = parse_timer_mode(mode_str);
    let timer = TimerDefinition::new(name.to_string(), seconds, mode);

    match store.insert_timer(&timer) {
        Ok(()) => {
            println!("⏱️ Timer \"{name}\" started: {minutes} minutes ({mode_str})");
        }
        Err(e) => {
            eprintln!("Error saving timer: {e}");
            std::process::exit(1);
        }
    }
}

// ── Reminder ──

pub fn handle_reminder(store: &adiyutant_store::SqliteStore, cmd: &ReminderCmd) {
    match cmd {
        ReminderCmd::Add { title, at } => {
            cmd_reminder_add(store, title, at);
        }
        ReminderCmd::List => {
            cmd_reminder_list(store);
        }
    }
}

fn cmd_reminder_add(store: &adiyutant_store::SqliteStore, title: &str, at: &str) {
    let reminder = ReminderDefinition::new(title.to_string(), at.to_string());

    match store.insert_reminder(&reminder) {
        Ok(()) => {
            println!("🔔 Reminder \"{title}\" added (rule: {at})");
        }
        Err(e) => {
            eprintln!("Error saving reminder: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_reminder_list(store: &adiyutant_store::SqliteStore) {
    match store.list_reminders() {
        Ok(reminders) => {
            if reminders.is_empty() {
                println!("No reminders.");
                return;
            }
            println!("🔔 Reminders ({}):", reminders.len());
            for r in &reminders {
                let status = if r.enabled { "enabled" } else { "disabled" };
                println!("  • \"{}\" — rule: {} ({status})", r.title, r.schedule_rule);
            }
        }
        Err(e) => {
            eprintln!("Error listing reminders: {e}");
            std::process::exit(1);
        }
    }
}

// ── Alarm ──

pub fn handle_alarm(store: &adiyutant_store::SqliteStore, cmd: &AlarmCmd) {
    match cmd {
        AlarmCmd::Add { title, at } => {
            cmd_alarm_add(store, title, at);
        }
        AlarmCmd::List => {
            cmd_alarm_list(store);
        }
    }
}

fn cmd_alarm_add(store: &adiyutant_store::SqliteStore, title: &str, at: &str) {
    // Parse HH:MM
    let time = match NaiveTime::parse_from_str(at, "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Invalid time format \"{at}\". Expected HH:MM (e.g. 07:00).");
            std::process::exit(1);
        }
    };

    let alarm = AlarmDefinition::new(title.to_string(), time);
    match store.insert_alarm(&alarm) {
        Ok(()) => {
            println!("⏰ Alarm \"{title}\" set for {at}:00");
        }
        Err(e) => {
            eprintln!("Error saving alarm: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_alarm_list(store: &adiyutant_store::SqliteStore) {
    match store.list_alarms() {
        Ok(alarms) => {
            if alarms.is_empty() {
                println!("No alarms.");
                return;
            }
            println!("⏰ Alarms ({}):", alarms.len());
            for a in &alarms {
                let status = if a.enabled { "enabled" } else { "disabled" };
                println!(
                    "  • \"{}\" at {} ({status})",
                    a.title,
                    a.time.format("%H:%M")
                );
            }
        }
        Err(e) => {
            eprintln!("Error listing alarms: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adiyutant_core::model::timer::TimerMode;
    use chrono::Timelike;

    #[test]
    fn parse_timer_mode_focus() {
        assert_eq!(parse_timer_mode("focus"), TimerMode::Focus);
    }

    #[test]
    fn parse_timer_mode_rest() {
        assert_eq!(parse_timer_mode("rest"), TimerMode::Rest);
    }

    #[test]
    fn parse_timer_mode_custom() {
        assert_eq!(parse_timer_mode("pomodoro"), TimerMode::Custom);
    }

    #[test]
    fn parse_timer_mode_case_insensitive() {
        assert_eq!(parse_timer_mode("FOCUS"), TimerMode::Focus);
    }

    #[test]
    fn parse_alarm_time_valid() {
        let time = NaiveTime::parse_from_str("07:00", "%H:%M").unwrap();
        assert_eq!(time.hour(), 7);
        assert_eq!(time.minute(), 0);
    }

    #[test]
    fn parse_alarm_time_invalid() {
        let result = NaiveTime::parse_from_str("25:00", "%H:%M");
        assert!(result.is_err());
    }
}
