use adiyutant_core::service::AdiyutantCoreService;

/// CLI subcommands for journal.
#[derive(clap::Subcommand, Debug)]
pub enum JournalCmd {
    /// Show today's journal entries
    Show,
    /// Add a journal entry
    Add {
        /// Entry type: CheckInCreated, ChecklistCompleted, TaskStarted, TaskDone, Note, Shutdown
        #[arg(long)]
        r#type: String,
        /// Summary text
        summary: String,
    },
}

pub fn handle_journal(facade: &AdiyutantCoreService, cmd: &JournalCmd) {
    match cmd {
        JournalCmd::Show => match facade.get_journal() {
            Ok(entries) => {
                if entries.is_empty() {
                    println!("📝 No journal entries for today.");
                    return;
                }
                println!("📝 Today's journal:");
                for e in &entries {
                    println!("  • [{}] {}", e.entry_type, e.summary);
                    println!("    Time: {}", e.timestamp);
                }
            }
            Err(e) => {
                eprintln!("Error reading journal: {e}");
                std::process::exit(1);
            }
        },
        JournalCmd::Add { r#type, summary } => match facade.add_journal_entry(r#type, summary) {
            Ok(entry) => {
                println!("✅ Journal entry added:");
                println!("  Type:    {}", entry.entry_type);
                println!("  Summary: {}", entry.summary);
            }
            Err(e) => {
                eprintln!("Error adding journal entry: {e}");
                std::process::exit(1);
            }
        },
    }
}
