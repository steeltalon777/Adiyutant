use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::day_plan::PlanItem;

/// A checkpoint associated with a plan item during its lifecycle.
#[derive(Debug, Clone)]
pub struct TaskCheckpoint {
    pub id: Id<TaskCheckpoint>,
    pub plan_item_id: Id<PlanItem>,
    pub kind: CheckpointKind,
    pub status: CheckpointStatus,
    pub response: Option<CheckpointResponse>,
    pub created_at: AdiyutantDateTime,
    pub answered_at: Option<AdiyutantDateTime>,
}

impl TaskCheckpoint {
    pub fn new(plan_item_id: Id<PlanItem>, kind: CheckpointKind) -> Self {
        Self {
            id: Id::new(),
            plan_item_id,
            kind,
            status: CheckpointStatus::Pending,
            response: None,
            created_at: AdiyutantDateTime::now(),
            answered_at: None,
        }
    }
}

/// What type of checkpoint to present to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointKind {
    StartCheck,
    ProgressCheck,
    FinishCheck,
    RescheduleCheck,
    RelevanceReview,
    ShutdownPrompt,
}

impl CheckpointKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckpointKind::StartCheck => "StartCheck",
            CheckpointKind::ProgressCheck => "ProgressCheck",
            CheckpointKind::FinishCheck => "FinishCheck",
            CheckpointKind::RescheduleCheck => "RescheduleCheck",
            CheckpointKind::RelevanceReview => "RelevanceReview",
            CheckpointKind::ShutdownPrompt => "ShutdownPrompt",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "StartCheck" => Some(CheckpointKind::StartCheck),
            "ProgressCheck" => Some(CheckpointKind::ProgressCheck),
            "FinishCheck" => Some(CheckpointKind::FinishCheck),
            "RescheduleCheck" => Some(CheckpointKind::RescheduleCheck),
            "RelevanceReview" => Some(CheckpointKind::RelevanceReview),
            "ShutdownPrompt" => Some(CheckpointKind::ShutdownPrompt),
            _ => None,
        }
    }
}

/// Display status of a checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointStatus {
    Pending,
    Shown,
    Answered,
    Dismissed,
}

impl CheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckpointStatus::Pending => "Pending",
            CheckpointStatus::Shown => "Shown",
            CheckpointStatus::Answered => "Answered",
            CheckpointStatus::Dismissed => "Dismissed",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Pending" => Some(CheckpointStatus::Pending),
            "Shown" => Some(CheckpointStatus::Shown),
            "Answered" => Some(CheckpointStatus::Answered),
            "Dismissed" => Some(CheckpointStatus::Dismissed),
            _ => None,
        }
    }
}

/// User's response to a checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointResponse {
    Started,
    Done,
    Snooze,
    DoingOther,
    Move,
    Skip,
    Cancel,
    KeepWaiting,
    Delete,
}

impl CheckpointResponse {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckpointResponse::Started => "Started",
            CheckpointResponse::Done => "Done",
            CheckpointResponse::Snooze => "Snooze",
            CheckpointResponse::DoingOther => "DoingOther",
            CheckpointResponse::Move => "Move",
            CheckpointResponse::Skip => "Skip",
            CheckpointResponse::Cancel => "Cancel",
            CheckpointResponse::KeepWaiting => "KeepWaiting",
            CheckpointResponse::Delete => "Delete",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Started" => Some(CheckpointResponse::Started),
            "Done" => Some(CheckpointResponse::Done),
            "Snooze" => Some(CheckpointResponse::Snooze),
            "DoingOther" => Some(CheckpointResponse::DoingOther),
            "Move" => Some(CheckpointResponse::Move),
            "Skip" => Some(CheckpointResponse::Skip),
            "Cancel" => Some(CheckpointResponse::Cancel),
            "KeepWaiting" => Some(CheckpointResponse::KeepWaiting),
            "Delete" => Some(CheckpointResponse::Delete),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_checkpoint_new_is_pending() {
        let item_id = Id::<PlanItem>::new();
        let cp = TaskCheckpoint::new(item_id, CheckpointKind::ProgressCheck);
        assert_eq!(cp.status, CheckpointStatus::Pending);
        assert!(cp.response.is_none());
        assert!(cp.answered_at.is_none());
    }

    #[test]
    fn checkpoint_kind_round_trip() {
        for k in &[
            CheckpointKind::StartCheck,
            CheckpointKind::ProgressCheck,
            CheckpointKind::FinishCheck,
            CheckpointKind::RescheduleCheck,
            CheckpointKind::RelevanceReview,
            CheckpointKind::ShutdownPrompt,
        ] {
            let s = k.as_str();
            let back = CheckpointKind::from_str(s).unwrap();
            assert_eq!(*k, back);
        }
    }

    #[test]
    fn checkpoint_response_round_trip() {
        for r in &[
            CheckpointResponse::Started,
            CheckpointResponse::Done,
            CheckpointResponse::Snooze,
            CheckpointResponse::DoingOther,
            CheckpointResponse::Move,
            CheckpointResponse::Skip,
            CheckpointResponse::Cancel,
            CheckpointResponse::KeepWaiting,
            CheckpointResponse::Delete,
        ] {
            let s = r.as_str();
            let back = CheckpointResponse::from_str(s).unwrap();
            assert_eq!(*r, back);
        }
    }
}
