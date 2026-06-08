use adiyutant_core::local_rule_gateway::LocalRuleAgentGateway;

use crate::common;

/// `adiyutant suggest` — show local suggestions based on today's state.
pub fn cmd_suggest(store: &adiyutant_store::SqliteStore) {
    let state = match common::build_today_state(store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error building today's state: {e}");
            std::process::exit(1);
        }
    };

    let result = LocalRuleAgentGateway::evaluate(&state);

    if result.proposals.is_empty() {
        println!("💡 No suggestions — everything looks good!");
        return;
    }

    println!("💡 Suggestions based on today's state:");
    for proposal in &result.proposals {
        let reason = proposal
            .payload_json
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("no reason");
        let suggestion = proposal
            .payload_json
            .get("suggestion")
            .and_then(|v| v.as_str())
            .unwrap_or("no suggestion");

        println!("  • {}: {}", proposal.proposal_type, reason);
        println!("    → {suggestion}");
    }
}
