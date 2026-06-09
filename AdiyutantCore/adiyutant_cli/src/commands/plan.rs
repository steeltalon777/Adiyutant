use adiyutant_core::service::AdiyutantCoreService;

/// CLI subcommands for planning.
#[derive(clap::Subcommand, Debug)]
pub enum PlanCmd {
    /// Add a plan item
    AddItem {
        /// Title of the task
        title: String,
        /// Eisenhower quadrant: important-urgent, important-not-urgent, not-important-urgent, not-important-not-urgent
        #[arg(long, default_value = "important-not-urgent")]
        quadrant: String,
        /// Priority (1-10, default 5)
        #[arg(long, default_value_t = 5)]
        priority: u8,
    },
    /// List today's plan
    List,
    /// Start a plan item
    StartItem {
        /// Item ID (UUID)
        id: String,
    },
    /// Mark a plan item as done
    DoneItem {
        /// Item ID (UUID)
        id: String,
    },
    /// Move a plan item to waiting
    MoveToWaiting {
        /// Item ID (UUID)
        id: String,
        /// Days until next review (default 7)
        #[arg(long, default_value_t = 7)]
        review_days: u64,
    },
}

pub fn handle_plan(facade: &AdiyutantCoreService, cmd: &PlanCmd) {
    match cmd {
        PlanCmd::AddItem {
            title,
            quadrant,
            priority,
        } => match facade.add_plan_item(title, Some(quadrant), Some(*priority)) {
            Ok(dto) => {
                println!("✅ Plan item added:");
                println!("  ID:        {}", dto.id);
                println!("  Title:     {}", dto.title);
                println!("  Quadrant:  {}", dto.quadrant);
                println!("  Status:    {}", dto.status);
            }
            Err(e) => {
                eprintln!("Error adding plan item: {e}");
                std::process::exit(1);
            }
        },
        PlanCmd::List => match facade.list_plan_items() {
            Ok(plan) => {
                println!("📋 Plan for {}", plan.date);
                if plan.items.is_empty() {
                    println!("  No items. Use `adiyutant plan add-item` to add one.");
                } else {
                    println!("  Items: {}", plan.item_count);
                    for item in &plan.items {
                        println!(
                            "  {} [{:>3}] {} — {}",
                            if item.status == "Done" { "✓" } else { "•" },
                            item.priority,
                            item.title,
                            item.status,
                        );
                        println!("         ID: {}", item.id);
                        println!("         Quadrant: {}", item.quadrant);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error listing plan: {e}");
                std::process::exit(1);
            }
        },
        PlanCmd::StartItem { id } => match facade.start_plan_item(id) {
            Ok(dto) => {
                println!("▶ Started: {} (status: {})", dto.title, dto.status);
            }
            Err(e) => {
                eprintln!("Error starting item: {e}");
                std::process::exit(1);
            }
        },
        PlanCmd::DoneItem { id } => match facade.done_plan_item(id) {
            Ok(dto) => {
                println!("✅ Done: {} (status: {})", dto.title, dto.status);
            }
            Err(e) => {
                eprintln!("Error marking item done: {e}");
                std::process::exit(1);
            }
        },
        PlanCmd::MoveToWaiting { id, review_days } => {
            match facade.move_to_waiting(id, Some(*review_days)) {
                Ok(dto) => {
                    println!("⏸ Moved to waiting: {}", dto.title);
                }
                Err(e) => {
                    eprintln!("Error moving to waiting: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
