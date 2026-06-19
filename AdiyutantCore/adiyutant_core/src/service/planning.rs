use super::*;
use crate::datetime::AdiyutantDateTime;
use crate::dto::{DayPlanDto, PlanItemDto, WaitingTaskDto};
use crate::id::Id;
use crate::model::daily_log::DailyLog;
use crate::model::day_plan::{EisenhowerQuadrant, PlanItem, PlanItemStatus, WaitingDecision};
use crate::model::journal_entry::{JournalEntry, JournalEntryType};
use crate::model::task_checkpoint::{CheckpointKind, TaskCheckpoint};

impl AdiyutantCoreService {
    /// Get or create today's DailyLog.
    pub(crate) fn get_or_create_today_log(&self) -> Result<Id<DailyLog>, crate::error::CoreError> {
        let today = self.time.today();
        if let Some(log) = self.store.get_daily_log_by_date(today)? {
            Ok(log.id)
        } else {
            let log = DailyLog::new(today);
            self.store.insert_daily_log(&log)?;
            Ok(log.id)
        }
    }

    /// Add a plan item for today.
    pub fn add_plan_item(
        &self,
        title: &str,
        quadrant: Option<&str>,
        priority: Option<u8>,
    ) -> Result<PlanItemDto, crate::error::CoreError> {
        let daily_log_id = self.get_or_create_today_log()?;
        let mut item = PlanItem::new(daily_log_id, title.to_string());

        if let Some(q) = quadrant {
            item.quadrant = EisenhowerQuadrant::from_str(q).ok_or_else(|| {
                crate::error::CoreError::InvalidInput(format!("unknown quadrant: {q}"))
            })?;
        }
        if let Some(p) = priority {
            item.priority = p;
        }

        // Auto-create a StartCheck checkpoint (atomic with plan_item insert)
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::StartCheck);
        self.store.insert_plan_item_with_checkpoint(&item, &cp)?;

        Ok(crate::service::helpers::plan_item_to_dto(&item))
    }

    /// List today's plan items.
    pub fn list_plan_items(&self) -> Result<DayPlanDto, crate::error::CoreError> {
        let today = self.time.today();
        let daily_log_id = match self.store.get_daily_log_by_date(today)? {
            Some(log) => log.id,
            None => {
                return Ok(DayPlanDto {
                    id: String::new(),
                    date: crate::dto::naive_date_to_string(today),
                    items: vec![],
                    item_count: 0,
                });
            }
        };

        let items = self.store.list_plan_items_by_log(daily_log_id)?;
        let item_dtos: Vec<PlanItemDto> = items
            .iter()
            .map(crate::service::helpers::plan_item_to_dto)
            .collect();

        Ok(DayPlanDto {
            id: String::new(),
            date: crate::dto::naive_date_to_string(today),
            items: item_dtos,
            item_count: items.len(),
        })
    }

    /// Start a plan item (set status to Started).
    pub fn start_plan_item(&self, item_id: &str) -> Result<PlanItemDto, crate::error::CoreError> {
        let id = crate::service::helpers::parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| crate::error::CoreError::NotFound(format!("plan_item {item_id}")))?;

        item.status = PlanItemStatus::Started;
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        let cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        let entry = JournalEntry::new(
            item.daily_log_id,
            JournalEntryType::TaskStarted,
            format!("Started: {}", item.title),
        );

        // Atomic: update_plan_item + insert_task_checkpoint + insert_journal_entry
        self.store.start_plan_item_composite(&item, &cp, &entry)?;
        Ok(crate::service::helpers::plan_item_to_dto(&item))
    }

    /// Mark a plan item as Done.
    pub fn done_plan_item(&self, item_id: &str) -> Result<PlanItemDto, crate::error::CoreError> {
        let id = crate::service::helpers::parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| crate::error::CoreError::NotFound(format!("plan_item {item_id}")))?;

        item.status = PlanItemStatus::Done;
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        let entry = JournalEntry::new(
            item.daily_log_id,
            JournalEntryType::TaskDone,
            format!("Done: {}", item.title),
        );

        // Atomic: update_plan_item + insert_journal_entry
        self.store.done_plan_item_composite(&item, &entry)?;
        Ok(crate::service::helpers::plan_item_to_dto(&item))
    }

    /// Move a plan item to waiting status.
    pub fn move_to_waiting(
        &self,
        item_id: &str,
        review_days: Option<u64>,
    ) -> Result<PlanItemDto, crate::error::CoreError> {
        let id = crate::service::helpers::parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| crate::error::CoreError::NotFound(format!("plan_item {item_id}")))?;

        item.status = PlanItemStatus::Waiting;
        item.waiting_review_at =
            Some(self.time.today() + chrono::Duration::days(review_days.unwrap_or(7) as i64));
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        let entry = JournalEntry::new(
            item.daily_log_id,
            JournalEntryType::TaskMoved,
            format!("Moved to waiting: {}", item.title),
        );

        // Atomic: update_plan_item + insert_journal_entry
        self.store.move_to_waiting_composite(&item, &entry)?;
        Ok(crate::service::helpers::plan_item_to_dto(&item))
    }

    /// List waiting items due for review.
    pub fn list_waiting_tasks(&self) -> Result<Vec<WaitingTaskDto>, crate::error::CoreError> {
        let today = self.time.today();
        let items = self.store.list_plan_items_due_for_review(today)?;
        Ok(items
            .iter()
            .map(|item| WaitingTaskDto {
                id: item.id.value().to_string(),
                title: item.title.clone(),
                waiting_since: item.updated_at.inner().to_string(),
                review_due: item
                    .waiting_review_at
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
            })
            .collect())
    }

    /// Review a waiting item.
    pub fn review_waiting(
        &self,
        item_id: &str,
        decision: &str,
    ) -> Result<PlanItemDto, crate::error::CoreError> {
        let decision = WaitingDecision::from_str(decision).ok_or_else(|| {
            crate::error::CoreError::InvalidInput(format!(
                "unknown decision: {decision} (use: keep, resume, delete, archive)"
            ))
        })?;

        let id = crate::service::helpers::parse_item_id(item_id)?;
        let mut item = self
            .store
            .get_plan_item(id)?
            .ok_or_else(|| crate::error::CoreError::NotFound(format!("plan_item {item_id}")))?;

        match decision {
            WaitingDecision::Keep => {
                // Keep waiting, extend review by 7 days
                item.waiting_review_at = Some(self.time.today() + chrono::Duration::days(7));
            }
            WaitingDecision::Resume => {
                item.status = PlanItemStatus::Planned;
                item.waiting_review_at = None;
            }
            WaitingDecision::Delete => {
                self.store.delete_plan_item(id)?;
                return Ok(PlanItemDto {
                    id: item_id.to_string(),
                    title: item.title,
                    description: crate::dto::option_string(&item.description),
                    quadrant: item.quadrant.as_str().to_string(),
                    planned_start: String::new(),
                    planned_end: String::new(),
                    status: "Deleted".to_string(),
                    priority: item.priority,
                    source: item.source.as_str().to_string(),
                    created_at: String::new(),
                    updated_at: String::new(),
                });
            }
            WaitingDecision::Archive => {
                item.status = PlanItemStatus::Archived;
                item.waiting_review_at = None;
            }
        }
        item.updated_at = AdiyutantDateTime::from_utc(self.time.now_utc());

        self.store.update_plan_item(&item)?;
        Ok(crate::service::helpers::plan_item_to_dto(&item))
    }
}
