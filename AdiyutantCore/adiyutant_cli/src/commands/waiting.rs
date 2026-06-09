use adiyutant_core::service::AdiyutantCoreService;

/// CLI subcommands for waiting task review.
#[derive(clap::Subcommand, Debug)]
pub enum WaitingCmd {
    /// List waiting items due for review
    List,
    /// Review a waiting item
    Review {
        /// Item ID (UUID)
        id: String,
        /// Decision: keep, resume, delete, archive
        #[arg(long)]
        decision: String,
    },
}

pub fn handle_waiting(facade: &AdiyutantCoreService, cmd: &WaitingCmd) {
    match cmd {
        WaitingCmd::List => match facade.list_waiting_tasks() {
            Ok(tasks) => {
                if tasks.is_empty() {
                    println!("No waiting tasks due for review.");
                } else {
                    println!("⏸ Waiting tasks due for review:");
                    for task in &tasks {
                        println!("  • {} (ID: {})", task.title, task.id);
                        println!("    Review due: {}", task.review_due);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error listing waiting tasks: {e}");
                std::process::exit(1);
            }
        },
        WaitingCmd::Review { id, decision } => match facade.review_waiting(id, decision) {
            Ok(dto) => {
                let action = match decision.as_str() {
                    "keep" => "kept in waiting",
                    "resume" => "resumed to plan",
                    "delete" => "deleted",
                    "archive" => "archived",
                    _ => "reviewed",
                };
                println!("✅ Item {} ({}) — {}", dto.title, dto.id, action);
            }
            Err(e) => {
                eprintln!("Error reviewing waiting item: {e}");
                std::process::exit(1);
            }
        },
    }
}
