mod commands;
mod common;

use clap::{Parser, Subcommand};

use crate::commands::checklist::{self, ChecklistCmd};
use crate::commands::context::{self, ContextCmd};
use crate::commands::daily::{self, CheckinArgs};
use crate::commands::habit::{self, HabitCmd};
use crate::commands::journal::{self, JournalCmd};
use crate::commands::plan::{self, PlanCmd};
use crate::commands::suggest;
use crate::commands::time::{self, AlarmCmd, ReminderCmd, TimerCmd};
use crate::commands::waiting::{self, WaitingCmd};
use crate::common::build_facade;

#[derive(Parser)]
#[command(name = "adiyutant")]
#[command(about = "Adiyutant — local-first organizer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show today's dashboard
    Today,
    /// Show raw daily log for today
    Log,
    /// Create a check-in
    Checkin(CheckinArgs),
    /// Show startup state / intent
    Startup,
    /// Show current activity
    Current,
    /// Manage checklists
    Checklist {
        #[command(subcommand)]
        cmd: ChecklistCmd,
    },
    /// Manage plans and plan items
    Plan {
        #[command(subcommand)]
        cmd: PlanCmd,
    },
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
    /// Journal entries
    Journal {
        #[command(subcommand)]
        cmd: JournalCmd,
    },
    /// Manage waiting tasks
    Waiting {
        #[command(subcommand)]
        cmd: WaitingCmd,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Today => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            daily::cmd_today(&facade);
        }
        Commands::Log => {
            let store = init_store_or_exit();
            daily::cmd_log(&store);
        }
        Commands::Checkin(args) => {
            let store = init_store_or_exit();
            daily::cmd_checkin(&store, args);
        }
        Commands::Startup => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            match facade.get_startup_state() {
                Ok(view) => {
                    println!("Startup intent: {}", view.intent);
                    println!("Reason: {}", view.reason);
                }
                Err(e) => {
                    eprintln!("Error getting startup state: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Current => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            match facade.get_current_activity() {
                Ok(activity) => {
                    println!("Date: {}", activity.date);
                    println!("Activity: {}", activity.activity_kind);
                    if !activity.active_task.is_empty() {
                        println!("Active task: {}", activity.active_task);
                    }
                    if !activity.next_checkpoint_at.is_empty() {
                        println!("Next checkpoint: {}", activity.next_checkpoint_at);
                    }
                    println!("→ {}", activity.recommended_prompt);
                }
                Err(e) => {
                    eprintln!("Error getting current activity: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Checklist { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            checklist::handle_checklist(&facade, cmd);
        }
        Commands::Plan { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            plan::handle_plan(&facade, cmd);
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
        Commands::Journal { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            journal::handle_journal(&facade, cmd);
        }
        Commands::Waiting { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            waiting::handle_waiting(&facade, cmd);
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
