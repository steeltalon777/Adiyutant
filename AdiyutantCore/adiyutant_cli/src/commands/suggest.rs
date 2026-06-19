use adiyutant_core::service::AdiyutantCoreService;

/// `adiyutant suggest` — show local suggestions via facade.
pub fn cmd_suggest(facade: &AdiyutantCoreService) {
    match facade.get_suggestions() {
        Ok(suggestions) => {
            if suggestions.is_empty() {
                println!("💡 No suggestions — everything looks good!");
                return;
            }
            println!("💡 Suggestions based on today's state:");
            for s in &suggestions {
                println!("  • [{}] {}: {}", s.proposal_type, s.reason, s.suggestion);
            }
        }
        Err(e) => {
            eprintln!("Error getting suggestions: {e}");
            std::process::exit(1);
        }
    }
}
