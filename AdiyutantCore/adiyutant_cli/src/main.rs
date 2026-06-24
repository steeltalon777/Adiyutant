mod commands;
mod common;

use clap::{Parser, Subcommand};

use crate::commands::checklist::{self, ChecklistCmd};
use crate::commands::context::{self, ContextCmd};
use crate::commands::daily::{self, CheckinArgs};
use crate::commands::export_cmd::{self, ExportCmd};
use crate::commands::habit::{self, HabitCmd};
use crate::commands::import_cmd::{self, ImportCmd};
use crate::commands::journal::{self, JournalCmd};
use crate::commands::plan::{self, PlanCmd};
use crate::commands::project as project_cmd;
use crate::commands::roadmap as roadmap_cmd;
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
    /// Manage projects
    #[command(subcommand)]
    Project(ProjectCmd),
    /// Manage roadmaps
    #[command(subcommand)]
    Roadmap(RoadmapCmd),
    /// Bundle import commands
    Import {
        #[command(subcommand)]
        cmd: ImportCmd,
    },
    /// Export commands
    Export {
        #[command(subcommand)]
        cmd: ExportCmd,
    },
}

#[derive(clap::Subcommand)]
enum ProjectCmd {
    /// List all projects
    List,
    /// Show project details
    Show { slug: String },
}

#[derive(clap::Subcommand)]
enum RoadmapCmd {
    /// List roadmaps, optionally filtered by project
    List {
        /// Project slug to filter by
        project_slug: Option<String>,
    },
    /// Show roadmap with phases and items
    Show { selector: String },
    /// List roadmap items
    Items { selector: String },
    /// Take a roadmap item into a day plan
    TakeItem {
        /// Item slug or selector (phase/item or roadmap/phase/item)
        selector: String,
        /// Target date: "today" or "tomorrow"
        #[arg(long, default_value = "today")]
        date: String,
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
            let facade = build_facade(store);
            daily::cmd_checkin(&facade, args);
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
            let facade = build_facade(store);
            habit::handle(&facade, cmd);
        }
        Commands::Timer { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            time::handle_timer(&facade, cmd);
        }
        Commands::Reminder { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            time::handle_reminder(&facade, cmd);
        }
        Commands::Alarm { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            time::handle_alarm(&facade, cmd);
        }
        Commands::Context { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            context::handle(&facade, cmd);
        }
        Commands::Suggest => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            suggest::cmd_suggest(&facade);
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
        Commands::Import { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            import_cmd::handle_import(&facade, cmd);
        }
        Commands::Export { cmd } => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            export_cmd::handle_export(&facade, cmd);
        }
        Commands::Project(cmd) => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            match cmd {
                ProjectCmd::List => project_cmd::handle_list(&facade),
                ProjectCmd::Show { slug } => project_cmd::handle_show(&facade, slug),
            }
        }
        Commands::Roadmap(cmd) => {
            let store = init_store_or_exit();
            let facade = build_facade(store);
            match cmd {
                RoadmapCmd::List { project_slug } => {
                    roadmap_cmd::handle_list(&facade, project_slug)
                }
                RoadmapCmd::Show { selector } => roadmap_cmd::handle_show(&facade, selector),
                RoadmapCmd::Items { selector } => roadmap_cmd::handle_items(&facade, selector),
                RoadmapCmd::TakeItem { selector, date } => {
                    roadmap_cmd::handle_take_item(&facade, selector, date)
                }
            }
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
