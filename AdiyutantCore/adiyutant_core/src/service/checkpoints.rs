use super::*;
use crate::datetime::AdiyutantDateTime;
use crate::dto::{NotificationInstructionDto, PlanItemDto};
use crate::model::day_plan::PlanItem;
use crate::model::journal_entry::{JournalEntry, JournalEntryType};
use crate::model::task_checkpoint::{
    CheckpointKind, CheckpointResponse, CheckpointStatus, TaskCheckpoint,
};

impl AdiyutantCoreService {
    /// Answer a checkpoint: transition Pending or Shown → Answered with response.
    /// Auto-creates the next checkpoint in the sequence via `calculate_next_checkpoint`.
    pub fn answer_checkpoint(
        &self,
        checkpoint_id: &str,
        response: &str,
    ) -> Result<PlanItemDto, crate::error::CoreError> {
        let id = crate::service::helpers::parse_id::<TaskCheckpoint>(checkpoint_id)?;
        let mut cp = self.store.get_task_checkpoint(id)?.ok_or_else(|| {
            crate::error::CoreError::NotFound(format!("checkpoint {checkpoint_id}"))
        })?;

        let resp = CheckpointResponse::from_str(response).ok_or_else(|| {
            crate::error::CoreError::InvalidInput(format!("unknown response: {response}"))
        })?;

        match cp.status {
            CheckpointStatus::Pending | CheckpointStatus::Shown => {
                cp.status = CheckpointStatus::Answered;
                cp.response = Some(resp);
                cp.answered_at = Some(AdiyutantDateTime::from_utc(self.time.now_utc()));
            }
            CheckpointStatus::Answered | CheckpointStatus::Dismissed => {
                return Err(crate::error::CoreError::InvalidInput(
                    "checkpoint already answered or dismissed".into(),
                ));
            }
        }

        let item = self
            .store
            .get_plan_item(cp.plan_item_id)?
            .ok_or_else(|| crate::error::CoreError::NotFound("plan_item for checkpoint".into()))?;

        // Auto-create next checkpoint in the sequence
        let next_cp =
            if let Some(next_kind) = Self::calculate_next_checkpoint(&item, &cp.kind, &resp) {
                Some(TaskCheckpoint::new(item.id, next_kind))
            } else {
                None
            };

        // Journal auto-event: NudgeAnswered
        let entry = JournalEntry::new(
            item.daily_log_id,
            JournalEntryType::NudgeAnswered,
            format!("Checkpoint answered: {} → {:?}", cp.kind.as_str(), resp),
        );

        // Atomic: update_task_checkpoint + optionally insert_next_checkpoint + insert_journal_entry
        self.store
            .answer_checkpoint_composite(&cp, next_cp.as_ref(), &entry)?;

        Ok(crate::service::helpers::plan_item_to_dto(&item))
    }

    /// Dismiss a checkpoint: transition Pending or Shown → Dismissed.
    pub fn dismiss_checkpoint(&self, checkpoint_id: &str) -> Result<(), crate::error::CoreError> {
        let id = crate::service::helpers::parse_id::<TaskCheckpoint>(checkpoint_id)?;
        let mut cp = self.store.get_task_checkpoint(id)?.ok_or_else(|| {
            crate::error::CoreError::NotFound(format!("checkpoint {checkpoint_id}"))
        })?;

        match cp.status {
            CheckpointStatus::Pending | CheckpointStatus::Shown => {
                cp.status = CheckpointStatus::Dismissed;
            }
            CheckpointStatus::Answered | CheckpointStatus::Dismissed => {
                return Err(crate::error::CoreError::InvalidInput(
                    "checkpoint already answered or dismissed".into(),
                ));
            }
        }

        self.store.update_task_checkpoint(&cp)
    }

    /// Determine the next checkpoint kind after answering one.
    pub(crate) fn calculate_next_checkpoint(
        _plan_item: &PlanItem,
        last_kind: &CheckpointKind,
        last_response: &CheckpointResponse,
    ) -> Option<CheckpointKind> {
        match (last_kind, last_response) {
            (CheckpointKind::StartCheck, CheckpointResponse::Started) => {
                Some(CheckpointKind::ProgressCheck)
            }
            (CheckpointKind::ProgressCheck, CheckpointResponse::Done) => {
                Some(CheckpointKind::FinishCheck)
            }
            (CheckpointKind::ProgressCheck, CheckpointResponse::Move) => {
                Some(CheckpointKind::RescheduleCheck)
            }
            (CheckpointKind::FinishCheck, CheckpointResponse::Done) => None,
            (CheckpointKind::RescheduleCheck, CheckpointResponse::KeepWaiting) => {
                Some(CheckpointKind::RelevanceReview)
            }
            _ => None,
        }
    }

    pub fn get_pending_checkpoint_notification(
        &self,
    ) -> Result<Option<NotificationInstructionDto>, crate::error::CoreError> {
        let today = self.time.today();
        let daily_log_id = match self.store.get_daily_log_by_date(today)? {
            Some(log) => log.id,
            None => return Ok(None),
        };

        let plan_items = self.store.list_plan_items_by_log(daily_log_id)?;
        for item in &plan_items {
            let checkpoints = self.store.list_checkpoints_by_plan_item(item.id)?;
            for cp in checkpoints {
                if matches!(cp.status, CheckpointStatus::Pending) {
                    return Ok(Some(NotificationInstructionDto {
                        source: "Checkpoint".into(),
                        title: format!("Checkpoint: {}", cp.kind.as_str()),
                        body: format!("Time to check in on task: {}", item.title),
                        action_id: cp.id.value().to_string(),
                    }));
                }
            }
        }
        Ok(None)
    }
}
