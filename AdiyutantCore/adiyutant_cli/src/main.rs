mod commands;
mod common;

use clap::{Parser, Subcommand};

use crate::commands::context::{self, ContextCmd};
use crate::commands::daily::{self, CheckinArgs};
use crate::commands::habit::{self, HabitCmd};
use crate::commands::suggest;
use crate::commands::time::{self, AlarmCmd, ReminderCmd, TimerCmd};

#[derive(Parser)]
#[command(name = "adiyutant")]
#[command(about = "Adiyutant — local-first organizer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show today's state
    Today,
    /// Show daily log for today
    Log,
    /// Create a check-in
    Checkin(CheckinArgs),
    /// Manage habits
    Habit {
        #[command(subcommand)]
        cmd: HabitCmd,
    },
    /// Timers
    Timer {
        #[command(subcommand)]
        cmd: TimerCmd,
    },
    /// Reminders
    Reminder {
        #[command(subcommand)]
        cmd: ReminderCmd,
    },
    /// Alarms
    Alarm {
        #[command(subcommand)]
        cmd: AlarmCmd,
    },
    /// Context documents
    Context {
        #[command(subcommand)]
        cmd: ContextCmd,
    },
    /// Show local suggestions
    Suggest,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Today => {
            let store = init_store_or_exit();
            daily::cmd_today(&store);
        }
        Commands::Log => {
            let store = init_store_or_exit();
            daily::cmd_log(&store);
        }
        Commands::Checkin(args) => {
            let store = init_store_or_exit();
            daily::cmd_checkin(&store, args);
        }
        Commands::Habit { cmd } => {
            let store = init_store_or_exit();
            habit::handle(&store, cmd);
        }
        Commands::Timer { cmd } => {
            let store = init_store_or_exit();
            time::handle_timer(&store, cmd);
        }
        Commands::Reminder { cmd } => {
            let store = init_store_or_exit();
            time::handle_reminder(&store, cmd);
        }
        Commands::Alarm { cmd } => {
            let store = init_store_or_exit();
            time::handle_alarm(&store, cmd);
        }
        Commands::Context { cmd } => {
            let store = init_store_or_exit();
            context::handle(&store, cmd);
        }
        Commands::Suggest => {
            let store = init_store_or_exit();
            suggest::cmd_suggest(&store);
        }
    }
}

fn init_store_or_exit() -> adiyutant_store::SqliteStore {
    match crate::common::init_store() {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Error initializing store: {e}");
            std::process::exit(1);
        }
    }
}
