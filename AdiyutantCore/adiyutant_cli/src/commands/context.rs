use adiyutant_core::service::AdiyutantCoreService;

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum ContextTypeArg {
    Core,
    Goals,
    Rules,
    Routine,
    Health,
    Work,
    Custom,
    LifeCore,
    RecoveryProtocol,
    Tone,
    PlanningPreferences,
}

impl ContextTypeArg {
    fn as_json_str(&self) -> &'static str {
        match self {
            ContextTypeArg::Core => "core",
            ContextTypeArg::Goals => "goals",
            ContextTypeArg::Rules => "rules",
            ContextTypeArg::Routine => "routine",
            ContextTypeArg::Health => "health",
            ContextTypeArg::Work => "work",
            ContextTypeArg::Custom => "custom",
            ContextTypeArg::LifeCore => "life_core",
            ContextTypeArg::RecoveryProtocol => "recovery_protocol",
            ContextTypeArg::Tone => "tone",
            ContextTypeArg::PlanningPreferences => "planning_preferences",
        }
    }
}

#[derive(clap::Subcommand, Debug)]
pub enum ContextCmd {
    /// Add a context document
    Add {
        /// Document type
        #[arg(value_enum)]
        doc_type: ContextTypeArg,
        /// Title
        title: String,
        /// Content (markdown)
        content: String,
    },
    /// List context documents
    List,
    /// Show a context document by title
    Show {
        /// Title to search for
        title: String,
    },
}

pub fn handle(facade: &AdiyutantCoreService, cmd: &ContextCmd) {
    match cmd {
        ContextCmd::Add {
            doc_type,
            title,
            content,
        } => match facade.add_context_document(doc_type.as_json_str(), title, content) {
            Ok(dto) => println!("✅ Context document added: {} (ID: {})", dto.title, dto.id),
            Err(e) => {
                eprintln!("Error adding context document: {e}");
                std::process::exit(1);
            }
        },
        ContextCmd::List => match facade.list_context_documents() {
            Ok(docs) => {
                if docs.is_empty() {
                    println!("No context documents.");
                    return;
                }
                println!("📄 Context documents:");
                for d in &docs {
                    println!("  • [{}] {} ({})", d.doc_type, d.title, d.id);
                }
            }
            Err(e) => {
                eprintln!("Error listing context documents: {e}");
                std::process::exit(1);
            }
        },
        ContextCmd::Show { title } => {
            // Use raw store for detailed view (still acceptable for debug-ish command)
            let store = crate::common::init_store().unwrap_or_else(|e| {
                eprintln!("Error initializing store: {e}");
                std::process::exit(1);
            });
            use adiyutant_core::store::Store;
            match store.list_context_documents() {
                Ok(docs) => {
                    let found: Vec<_> = docs
                        .iter()
                        .filter(|d| d.title.to_lowercase().contains(&title.to_lowercase()))
                        .collect();
                    if found.is_empty() {
                        println!("No context document matching \"{title}\".");
                    } else {
                        for doc in found {
                            println!("── {:?}: {} (v{}) ──", doc.doc_type, doc.title, doc.version);
                            println!("{}", doc.content_markdown);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading context: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_type_arg_as_json_str() {
        assert_eq!(ContextTypeArg::Core.as_json_str(), "core");
        assert_eq!(ContextTypeArg::LifeCore.as_json_str(), "life_core");
        assert_eq!(
            ContextTypeArg::RecoveryProtocol.as_json_str(),
            "recovery_protocol"
        );
    }
}
