use super::*;
use crate::dto::SuggestionDto;
use crate::local_rule_gateway::LocalRuleAgentGateway;

impl AdiyutantCoreService {
    pub fn get_suggestions(&self) -> Result<Vec<SuggestionDto>, crate::error::CoreError> {
        let state = self.build_today_state()?;
        let result = LocalRuleAgentGateway::evaluate(&state);
        Ok(result
            .proposals
            .iter()
            .map(|p| SuggestionDto {
                proposal_type: p.proposal_type.clone(),
                reason: p
                    .payload_json
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                suggestion: p
                    .payload_json
                    .get("suggestion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }
}
