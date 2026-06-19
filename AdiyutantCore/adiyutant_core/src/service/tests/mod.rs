mod mock_store;

use super::helpers::parse_id;
use super::*;
use crate::datetime::AdiyutantDateTime;
use crate::error::CoreError;
use crate::id::Id;
use crate::model::check_in::{CheckIn, CheckInType};
use crate::model::checklist_run::ChecklistRun;
use crate::model::checklist_template::ChecklistTemplate;
use crate::model::daily_log::DailyLog;
use crate::model::day_plan::PlanItem;

use crate::model::journal_entry::JournalEntryType;
use crate::model::plan::Plan;
use crate::model::plan::PlanItemStatus;
use crate::model::task_checkpoint::{
    CheckpointKind, CheckpointResponse, CheckpointStatus, TaskCheckpoint,
};
use crate::time_provider::FakeTimeProvider;
use chrono::NaiveDate;
use mock_store::MockStore;

// ── TimeProvider integration tests ──────────

#[test]
fn service_with_time_returns_fixed_hour() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 14);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
    assert_eq!(service.time.hour(), 14);
    assert_eq!(
        service.time.today(),
        NaiveDate::from_ymd_opt(2026, 6, 10).unwrap()
    );
}

#[test]
fn startup_state_uses_time_provider() {
    let store = MockStore::new();
    let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    store.insert_daily_log(&log).unwrap();
    let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".to_string());
    store.insert_check_in(&ci).unwrap();
    let mut plan = Plan::new(log.id, "My Plan".into());
    plan.add_item("Task 1".into());
    store.insert_plan(&plan).unwrap();

    let time = FakeTimeProvider::new(2026, 6, 10, 22);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
    let view = service.get_startup_state().unwrap();
    assert_eq!(view.intent, "suggest_evening_shutdown");
}

#[test]
fn current_activity_uses_time_provider() {
    let store = MockStore::new();
    let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    store.insert_daily_log(&log).unwrap();
    let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".to_string());
    store.insert_check_in(&ci).unwrap();
    let mut plan = Plan::new(log.id, "My Plan".into());
    plan.add_item("Task 1".into());
    plan.items[0].status = PlanItemStatus::Done;
    store.insert_plan(&plan).unwrap();

    let time = FakeTimeProvider::new(2026, 6, 10, 22);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
    let activity = service.get_current_activity().unwrap();
    assert_eq!(activity.activity_kind, "shutdown");
}

// ── Checkpoint tests ───────────────────────

#[test]
fn calculate_next_checkpoint_chain() {
    let item = PlanItem::new(Id::<DailyLog>::new(), "test".into());

    // StartCheck + Started → ProgressCheck
    let next = AdiyutantCoreService::calculate_next_checkpoint(
        &item,
        &CheckpointKind::StartCheck,
        &CheckpointResponse::Started,
    );
    assert_eq!(next, Some(CheckpointKind::ProgressCheck));

    // ProgressCheck + Done → FinishCheck
    let next = AdiyutantCoreService::calculate_next_checkpoint(
        &item,
        &CheckpointKind::ProgressCheck,
        &CheckpointResponse::Done,
    );
    assert_eq!(next, Some(CheckpointKind::FinishCheck));

    // ProgressCheck + Move → RescheduleCheck
    let next = AdiyutantCoreService::calculate_next_checkpoint(
        &item,
        &CheckpointKind::ProgressCheck,
        &CheckpointResponse::Move,
    );
    assert_eq!(next, Some(CheckpointKind::RescheduleCheck));

    // FinishCheck + Done → None
    let next = AdiyutantCoreService::calculate_next_checkpoint(
        &item,
        &CheckpointKind::FinishCheck,
        &CheckpointResponse::Done,
    );
    assert_eq!(next, None);

    // RescheduleCheck + KeepWaiting → RelevanceReview
    let next = AdiyutantCoreService::calculate_next_checkpoint(
        &item,
        &CheckpointKind::RescheduleCheck,
        &CheckpointResponse::KeepWaiting,
    );
    assert_eq!(next, Some(CheckpointKind::RelevanceReview));

    // Unknown combo → None
    let next = AdiyutantCoreService::calculate_next_checkpoint(
        &item,
        &CheckpointKind::StartCheck,
        &CheckpointResponse::Done,
    );
    assert_eq!(next, None);
}

#[test]
fn answer_checkpoint_state_machine() {
    let store = MockStore::new();
    let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    store.insert_daily_log(&log).unwrap();
    let item = PlanItem::new(log.id, "test".into());
    store.insert_plan_item(&item).unwrap();
    let cp = TaskCheckpoint::new(item.id, CheckpointKind::StartCheck);
    let cp_id = cp.id.value().to_string();
    store.insert_task_checkpoint(&cp).unwrap();

    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    // First call: Pending → Answered (directly, with response)
    let result = service.answer_checkpoint(&cp_id, "Started").unwrap();
    assert_eq!(result.title, "test");

    let cp = service
        .store
        .get_task_checkpoint(parse_id(&cp_id).unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(cp.status, CheckpointStatus::Answered);
    assert_eq!(cp.response, Some(CheckpointResponse::Started));
    assert!(cp.answered_at.is_some());

    // Second call: Answered → error
    let err = service.answer_checkpoint(&cp_id, "Started").unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput(_)));
}

#[test]
fn get_pending_checkpoint_notification_returns_first_pending() {
    let store = MockStore::new();
    let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    store.insert_daily_log(&log).unwrap();
    let item = PlanItem::new(log.id, "My Task".into());
    store.insert_plan_item(&item).unwrap();
    let cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
    store.insert_task_checkpoint(&cp).unwrap();

    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    let notification = service.get_pending_checkpoint_notification().unwrap();
    assert!(notification.is_some());
    let note = notification.unwrap();
    assert_eq!(note.source, "Checkpoint");
    assert!(note.title.contains("ProgressCheck"));
    assert!(note.body.contains("My Task"));
}

#[test]
fn get_pending_checkpoint_notification_none_when_no_log() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
    let notification = service.get_pending_checkpoint_notification().unwrap();
    assert!(notification.is_none());
}

// ── Checklist tests ────────────────────────

#[test]
fn complete_checklist_run_adds_journal_entry() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    let log_id = service.get_or_create_today_log().unwrap();
    let mut template = ChecklistTemplate::new("Test".into(), "morning".into());
    service.store.insert_checklist_template(&template).unwrap();
    template = service
        .store
        .get_checklist_template(template.id)
        .unwrap()
        .unwrap();

    let run = ChecklistRun::new(template.id, log_id);
    let run_id = run.id.value().to_string();
    service.store.insert_checklist_run(&run).unwrap();

    let result = service.complete_checklist_run(&run_id).unwrap();
    assert!(
        !result.completed_at.is_empty(),
        "completed_at should be set"
    );
    assert_eq!(result.template_title, "Test");

    let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].entry_type, JournalEntryType::ChecklistCompleted);
}

#[test]
fn answer_checklist_item_on_completed_run_errors() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    let log_id = service.get_or_create_today_log().unwrap();
    let template = ChecklistTemplate::new("Test".into(), "morning".into());
    service.store.insert_checklist_template(&template).unwrap();
    let mut run = ChecklistRun::new(template.id, log_id);
    run.completed_at = Some(AdiyutantDateTime::from_utc(service.time.now_utc()));
    let run_id = run.id.value().to_string();
    service.store.insert_checklist_run(&run).unwrap();

    let err = service
        .answer_checklist_item(&run_id, "any", "yes", None)
        .unwrap_err();
    assert!(matches!(err, CoreError::InvalidInput(_)));
}

// ── Journal auto-event tests ────────────────

#[test]
fn start_plan_item_creates_journal_entry() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    let log_id = service.get_or_create_today_log().unwrap();
    let item = PlanItem::new(log_id, "Test Task".into());
    let item_id = item.id.value().to_string();
    service.store.insert_plan_item(&item).unwrap();

    service.start_plan_item(&item_id).unwrap();
    let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
    assert!(
        journal
            .iter()
            .any(|e| e.entry_type == JournalEntryType::TaskStarted)
    );
}

#[test]
fn done_plan_item_creates_journal_entry() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    let log_id = service.get_or_create_today_log().unwrap();
    let item = PlanItem::new(log_id, "Test Task".into());
    let item_id = item.id.value().to_string();
    service.store.insert_plan_item(&item).unwrap();

    service.done_plan_item(&item_id).unwrap();
    let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
    assert!(
        journal
            .iter()
            .any(|e| e.entry_type == JournalEntryType::TaskDone)
    );
}

#[test]
fn move_to_waiting_creates_journal_entry() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    let log_id = service.get_or_create_today_log().unwrap();
    let item = PlanItem::new(log_id, "Test Task".into());
    let item_id = item.id.value().to_string();
    service.store.insert_plan_item(&item).unwrap();

    service.move_to_waiting(&item_id, None).unwrap();
    let journal = service.store.list_journal_entries_by_log(log_id).unwrap();
    assert!(
        journal
            .iter()
            .any(|e| e.entry_type == JournalEntryType::TaskMoved)
    );
}

// ── Integration tests ──

#[test]
fn get_today_works_with_fake_time() {
    let store = MockStore::new();
    let log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    store.insert_daily_log(&log).unwrap();

    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));
    let today = service.get_today().unwrap();
    assert_eq!(today.date, "2026-06-10");
}

#[test]
fn new_service_uses_real_time_provider() {
    let store = MockStore::new();
    let service = AdiyutantCoreService::new(Box::new(store));
    let _hour = service.time.hour();
}

#[test]
fn add_plan_item_creates_start_check_checkpoint() {
    let store = MockStore::new();
    let time = FakeTimeProvider::new(2026, 6, 10, 10);
    let service = AdiyutantCoreService::with_time(Box::new(store), Box::new(time));

    let dto = service.add_plan_item("Test", None, None).unwrap();
    assert_eq!(dto.title, "Test");

    let log_id = service.get_or_create_today_log().unwrap();
    let items = service.store.list_plan_items_by_log(log_id).unwrap();
    assert_eq!(items.len(), 1);
    let checkpoints = service
        .store
        .list_checkpoints_by_plan_item(items[0].id)
        .unwrap();
    assert!(!checkpoints.is_empty());
    assert_eq!(checkpoints[0].kind, CheckpointKind::StartCheck);
}
