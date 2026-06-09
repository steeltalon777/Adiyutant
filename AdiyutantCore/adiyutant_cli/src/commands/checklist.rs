use adiyutant_core::service::AdiyutantCoreService;

/// CLI subcommands for checklists.
#[derive(clap::Subcommand, Debug)]
pub enum ChecklistCmd {
    /// List available checklist templates
    List {
        /// Optional category filter: morning, day, evening, shutdown, recovery
        #[arg(long)]
        category: Option<String>,
    },
    /// Run a checklist by category (e.g. morning, evening, shutdown)
    Run {
        /// Category name: morning, day, evening, shutdown, recovery
        category: String,
    },
}

pub fn handle_checklist(facade: &AdiyutantCoreService, cmd: &ChecklistCmd) {
    match cmd {
        ChecklistCmd::List { category } => {
            match facade.list_checklist_templates(category.as_deref()) {
                Ok(templates) => {
                    if templates.is_empty() {
                        println!("No checklist templates found.");
                        return;
                    }
                    println!("📋 Checklist templates:");
                    for t in &templates {
                        println!(
                            "  • {} [{}] — {} items {}",
                            t.title,
                            t.category,
                            t.item_count,
                            if t.is_active { "" } else { "(inactive)" }
                        );
                        println!("    ID: {}", t.id);
                    }
                }
                Err(e) => {
                    eprintln!("Error listing checklists: {e}");
                    std::process::exit(1);
                }
            }
        }
        ChecklistCmd::Run { category } => match facade.run_checklist(category) {
            Ok(run) => {
                println!("✅ Checklist run started:");
                println!("  Template: {}", run.template_title);
                println!("  Started:  {}", run.started_at);
                println!("  Run ID:   {}", run.id);
            }
            Err(e) => {
                eprintln!("Error running checklist: {e}");
                std::process::exit(1);
            }
        },
    }
}
