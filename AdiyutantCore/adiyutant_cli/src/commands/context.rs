use adiyutant_core::model::context_document::{ContextDocument, ContextDocumentType};
use adiyutant_core::store::Store;

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum ContextTypeArg {
    Core,
    Goals,
    Rules,
    Routine,
    Health,
    Work,
    Custom,
}

impl ContextTypeArg {
    fn to_domain(&self) -> ContextDocumentType {
        match self {
            ContextTypeArg::Core => ContextDocumentType::Core,
            ContextTypeArg::Goals => ContextDocumentType::Goals,
            ContextTypeArg::Rules => ContextDocumentType::Rules,
            ContextTypeArg::Routine => ContextDocumentType::Routine,
            ContextTypeArg::Health => ContextDocumentType::Health,
            ContextTypeArg::Work => ContextDocumentType::Work,
            ContextTypeArg::Custom => ContextDocumentType::Custom,
        }
    }
}

fn doc_type_display(dt: &ContextDocumentType) -> &'static str {
    match dt {
        ContextDocumentType::Core => "core",
        ContextDocumentType::Goals => "goals",
        ContextDocumentType::Rules => "rules",
        ContextDocumentType::Routine => "routine",
        ContextDocumentType::Health => "health",
        ContextDocumentType::Work => "work",
        ContextDocumentType::Custom => "custom",
    }
}

#[derive(clap::Subcommand, Debug)]
pub enum ContextCmd {
    /// Add a context document
    Add {
        /// Document type: core, goals, rules, routine, health, work, custom
        #[arg(value_enum)]
        doc_type: ContextTypeArg,
        /// Document title
        title: String,
        /// Document content (markdown)
        content: String,
    },
    /// List context documents
    List,
    /// Show a context document by title (first match)
    Show {
        /// Document title to show
        title: String,
    },
}

pub fn handle(store: &adiyutant_store::SqliteStore, cmd: &ContextCmd) {
    match cmd {
        ContextCmd::Add {
            doc_type,
            title,
            content,
        } => cmd_add(store, doc_type, title, content),
        ContextCmd::List => cmd_list(store),
        ContextCmd::Show { title } => cmd_show(store, title),
    }
}

fn cmd_add(
    store: &adiyutant_store::SqliteStore,
    doc_type: &ContextTypeArg,
    title: &str,
    content: &str,
) {
    let doc = ContextDocument::new(doc_type.to_domain(), title.to_string(), content.to_string());

    match store.insert_context_document(&doc) {
        Ok(()) => {
            let type_name = format!("{:?}", doc_type).to_lowercase();
            println!("📄 Context document \"{title}\" added (type: {type_name}, v1)");
        }
        Err(e) => {
            eprintln!("Error saving context document: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_list(store: &adiyutant_store::SqliteStore) {
    match store.list_context_documents() {
        Ok(docs) => {
            if docs.is_empty() {
                println!("No context documents. Use `adiyutant context add ...` to create one.");
                return;
            }
            println!("📚 Context documents ({}):", docs.len());
            for d in &docs {
                let type_name = doc_type_display(&d.doc_type);
                println!("  • [{type_name}] {} (v{})", d.title, d.version);
            }
        }
        Err(e) => {
            eprintln!("Error listing context documents: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_show(store: &adiyutant_store::SqliteStore, title: &str) {
    match store.list_context_documents() {
        Ok(docs) => {
            let doc = docs.into_iter().find(|d| d.title == title);
            match doc {
                Some(d) => {
                    let type_name = doc_type_display(&d.doc_type);
                    println!("── {title} ({type_name}, v{}) ──", d.version);
                    println!("{}", d.content_markdown);
                }
                None => {
                    eprintln!("Context document \"{title}\" not found.");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading context documents: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adiyutant_core::model::context_document::ContextDocumentType as DomainType;

    #[test]
    fn context_type_arg_to_domain() {
        assert_eq!(ContextTypeArg::Core.to_domain(), DomainType::Core);
        assert_eq!(ContextTypeArg::Goals.to_domain(), DomainType::Goals);
        assert_eq!(ContextTypeArg::Rules.to_domain(), DomainType::Rules);
        assert_eq!(ContextTypeArg::Routine.to_domain(), DomainType::Routine);
        assert_eq!(ContextTypeArg::Health.to_domain(), DomainType::Health);
        assert_eq!(ContextTypeArg::Work.to_domain(), DomainType::Work);
        assert_eq!(ContextTypeArg::Custom.to_domain(), DomainType::Custom);
    }
}
