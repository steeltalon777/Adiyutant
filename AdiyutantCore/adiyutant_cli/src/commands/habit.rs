use adiyutant_core::model::habit::Habit;
use adiyutant_core::model::habit_event::{HabitEvent, HabitEventLevel, HabitEventStatus};
use adiyutant_core::store::Store;

#[derive(clap::Subcommand, Debug)]
pub enum HabitCmd {
    /// Add a new habit
    Add {
        /// Habit name
        name: String,
    },
    /// List all habits
    List,
    /// Mark a habit as done
    Done {
        /// Habit name (first match)
        name: String,
        /// Level: min, light, base, full
        #[arg(short, long, default_value = "base")]
        level: LevelArg,
    },
    /// Skip a habit today
    Skip {
        /// Habit name (first match)
        name: String,
    },
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum LevelArg {
    Min,
    Light,
    Base,
    Full,
}

impl LevelArg {
    fn to_domain(&self) -> HabitEventLevel {
        match self {
            LevelArg::Min => HabitEventLevel::Min,
            LevelArg::Light => HabitEventLevel::Light,
            LevelArg::Base => HabitEventLevel::Base,
            LevelArg::Full => HabitEventLevel::Full,
        }
    }
}

/// Find a habit by name (first match).
fn find_habit_by_name(store: &adiyutant_store::SqliteStore, name: &str) -> Option<Habit> {
    store
        .list_habits()
        .ok()?
        .into_iter()
        .find(|h| h.name == name)
}

/// Handle habit subcommands.
pub fn handle(store: &adiyutant_store::SqliteStore, cmd: &HabitCmd) {
    match cmd {
        HabitCmd::Add { name } => cmd_add(store, name),
        HabitCmd::List => cmd_list(store),
        HabitCmd::Done { name, level } => cmd_done(store, name, level),
        HabitCmd::Skip { name } => cmd_skip(store, name),
    }
}

fn cmd_add(store: &adiyutant_store::SqliteStore, name: &str) {
    let habit = Habit::new(name.to_string());
    match store.insert_habit(&habit) {
        Ok(()) => println!("✅ Habit \"{name}\" created."),
        Err(e) => {
            eprintln!("Error creating habit: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_list(store: &adiyutant_store::SqliteStore) {
    match store.list_habits() {
        Ok(habits) => {
            if habits.is_empty() {
                println!("No habits yet. Use `adiyutant habit add <name>` to create one.");
                return;
            }
            println!("📋 Habits ({}):", habits.len());
            for h in &habits {
                let status = if h.is_active { "active" } else { "inactive" };
                println!("  • {} — {status}", h.name);
            }
        }
        Err(e) => {
            eprintln!("Error listing habits: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_done(store: &adiyutant_store::SqliteStore, name: &str, level: &LevelArg) {
    let habit = match find_habit_by_name(store, name) {
        Some(h) => h,
        None => {
            eprintln!("Habit \"{name}\" not found.");
            std::process::exit(1);
        }
    };

    let event = HabitEvent::new(habit.id, HabitEventStatus::Done, level.to_domain());
    match store.insert_habit_event(&event) {
        Ok(()) => {
            let level_name = format!("{:?}", level).to_lowercase();
            println!("✅ {name} done ({level_name})");
        }
        Err(e) => {
            eprintln!("Error recording habit event: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_skip(store: &adiyutant_store::SqliteStore, name: &str) {
    let habit = match find_habit_by_name(store, name) {
        Some(h) => h,
        None => {
            eprintln!("Habit \"{name}\" not found.");
            std::process::exit(1);
        }
    };

    let event = HabitEvent::new(habit.id, HabitEventStatus::Skipped, HabitEventLevel::Base);
    match store.insert_habit_event(&event) {
        Ok(()) => {
            println!("⏭️  {name} skipped");
        }
        Err(e) => {
            eprintln!("Error recording habit event: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adiyutant_core::model::habit_event::HabitEventLevel as DomainLevel;

    #[test]
    fn level_arg_to_domain() {
        assert_eq!(LevelArg::Min.to_domain(), DomainLevel::Min);
        assert_eq!(LevelArg::Light.to_domain(), DomainLevel::Light);
        assert_eq!(LevelArg::Base.to_domain(), DomainLevel::Base);
        assert_eq!(LevelArg::Full.to_domain(), DomainLevel::Full);
    }
}
