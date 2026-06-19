use adiyutant_core::service::AdiyutantCoreService;

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
        /// Habit name (matched case-insensitively) or ID
        name: String,
        /// Level: min, light, base, full
        #[arg(long, default_value = "base")]
        level: LevelArg,
    },
    /// Skip a habit
    Skip {
        /// Habit name or ID
        name: String,
    },
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum LevelArg {
    Min,
    Light,
    Base,
    Full,
}

impl LevelArg {
    fn as_str(&self) -> &'static str {
        match self {
            LevelArg::Min => "min",
            LevelArg::Light => "light",
            LevelArg::Base => "base",
            LevelArg::Full => "full",
        }
    }
}

pub fn handle(facade: &AdiyutantCoreService, cmd: &HabitCmd) {
    match cmd {
        HabitCmd::Add { name } => match facade.add_habit(name) {
            Ok(dto) => println!("✅ Habit added: {} (ID: {})", dto.name, dto.id),
            Err(e) => {
                eprintln!("Error adding habit: {e}");
                std::process::exit(1);
            }
        },
        HabitCmd::List => match facade.list_habits() {
            Ok(habits) => {
                if habits.is_empty() {
                    println!("No habits tracked.");
                } else {
                    println!("📋 Habits:");
                    for h in &habits {
                        let status = if h.is_active { "" } else { " (inactive)" };
                        println!("  • {}{status} — {}", h.name, h.id);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error listing habits: {e}");
                std::process::exit(1);
            }
        },
        HabitCmd::Done { name, level } => {
            match facade.mark_habit_done(name, Some(level.as_str())) {
                Ok(dto) => println!(
                    "✅ Habit done: {} at {} (event ID: {})",
                    dto.habit_name, dto.date_time, dto.id
                ),
                Err(e) => {
                    eprintln!("Error marking habit done: {e}");
                    std::process::exit(1);
                }
            }
        }
        HabitCmd::Skip { name } => match facade.skip_habit(name) {
            Ok(dto) => println!("⏭ Habit skipped: {} (event ID: {})", dto.habit_name, dto.id),
            Err(e) => {
                eprintln!("Error skipping habit: {e}");
                std::process::exit(1);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_arg_as_str() {
        assert_eq!(LevelArg::Min.as_str(), "min");
        assert_eq!(LevelArg::Base.as_str(), "base");
    }
}
