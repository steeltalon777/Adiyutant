use adiyutant_core::service::AdiyutantCoreService;

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

// ── Timer ──

pub fn handle_timer(facade: &AdiyutantCoreService, cmd: &TimerCmd) {
    match cmd {
        TimerCmd::Start {
            name,
            minutes,
            mode: _,
        } => {
            let seconds = minutes * 60;
            match facade.add_timer(name, seconds) {
                Ok(id) => println!("⏱️ Timer \"{name}\" saved ({minutes} min, ID: {id})"),
                Err(e) => {
                    eprintln!("Error saving timer: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

// ── Reminder ──

pub fn handle_reminder(facade: &AdiyutantCoreService, cmd: &ReminderCmd) {
    match cmd {
        ReminderCmd::Add { title, at } => match facade.add_reminder(title, at) {
            Ok(id) => println!("🔔 Reminder \"{title}\" added (rule: {at}, ID: {id})"),
            Err(e) => {
                eprintln!("Error adding reminder: {e}");
                std::process::exit(1);
            }
        },
        ReminderCmd::List => match facade.list_reminders() {
            Ok(reminders) => {
                if reminders.is_empty() {
                    println!("No reminders.");
                    return;
                }
                println!("🔔 Reminders ({}):", reminders.len());
                for (id, title, rule) in &reminders {
                    println!("  • \"{title}\" — rule: {rule} ({id})");
                }
            }
            Err(e) => {
                eprintln!("Error listing reminders: {e}");
                std::process::exit(1);
            }
        },
    }
}

// ── Alarm ──

pub fn handle_alarm(facade: &AdiyutantCoreService, cmd: &AlarmCmd) {
    match cmd {
        AlarmCmd::Add { title, at } => match facade.add_alarm(title, at) {
            Ok(id) => println!("⏰ Alarm \"{title}\" set for {at} (ID: {id})"),
            Err(e) => {
                eprintln!("Error adding alarm: {e}");
                std::process::exit(1);
            }
        },
        AlarmCmd::List => match facade.list_alarms() {
            Ok(alarms) => {
                if alarms.is_empty() {
                    println!("No alarms.");
                    return;
                }
                println!("⏰ Alarms ({}):", alarms.len());
                for (id, title, time) in &alarms {
                    println!("  • \"{title}\" at {time} ({id})");
                }
            }
            Err(e) => {
                eprintln!("Error listing alarms: {e}");
                std::process::exit(1);
            }
        },
    }
}
