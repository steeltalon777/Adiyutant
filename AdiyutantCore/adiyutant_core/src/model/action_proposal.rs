use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalSource {
    LocalRule,
    Agent,
    User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionProposal {
    pub id: Id<Self>,
    pub source: ProposalSource,
    pub proposal_type: String,
    pub payload_json: serde_json::Value,
    pub status: ProposalStatus,
    pub created_at: AdiyutantDateTime,
    pub applied_at: Option<AdiyutantDateTime>,
}

impl ActionProposal {
    pub fn new(
        source: ProposalSource,
        proposal_type: String,
        payload_json: serde_json::Value,
    ) -> Self {
        Self {
            id: Id::new(),
            source,
            proposal_type,
            payload_json,
            status: ProposalStatus::Pending,
            created_at: AdiyutantDateTime::now(),
            applied_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_creates_pending_proposal() {
        let p = ActionProposal::new(
            ProposalSource::Agent,
            "add_habit".into(),
            json!({"name": "speech"}),
        );
        assert_eq!(p.source, ProposalSource::Agent);
        assert_eq!(p.proposal_type, "add_habit");
        assert_eq!(p.status, ProposalStatus::Pending);
    }

    #[test]
    fn applied_at_is_none_by_default() {
        let p = ActionProposal::new(
            ProposalSource::User,
            "remind".into(),
            json!({"text": "drink water"}),
        );
        assert!(p.applied_at.is_none());
    }

    #[test]
    fn serde_round_trip_with_payload() {
        let payload = json!({"action": "add_habit", "name": "speech"});
        let p = ActionProposal::new(
            ProposalSource::LocalRule,
            "add_habit".into(),
            payload.clone(),
        );
        let json_str = serde_json::to_string(&p).unwrap();
        let deserialized: ActionProposal = serde_json::from_str(&json_str).unwrap();
        assert_eq!(p.id, deserialized.id);
        assert_eq!(deserialized.payload_json, payload);
    }
}
