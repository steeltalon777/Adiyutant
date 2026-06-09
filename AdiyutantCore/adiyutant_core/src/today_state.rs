use crate::datetime::AdiyutantDateTime;
use crate::model::alarm::AlarmDefinition;
use crate::model::check_in::{CheckIn, CheckInType};
use crate::model::context_document::ContextDocument;
use crate::model::daily_log::DailyLog;
use crate::model::day_plan::PlanItem;
use crate::model::habit::Habit;
use crate::model::habit_event::HabitEvent;
use crate::model::plan::Plan;
use crate::model::reminder::ReminderDefinition;
use crate::model::timer::TimerDefinition;

/// Read-only snapshot of the current day's state.
///
/// Aggregates all domain models relevant to "today" into one structure
/// for CLI display, UI binding, or rule evaluation.
#[derive(Debug, Clone)]
pub struct TodayState {
    pub date: chrono::NaiveDate,
    pub daily_log: Option<DailyLog>,
    pub check_ins: Vec<CheckIn>,
    pub habits: Vec<Habit>,
    pub habit_events: Vec<HabitEvent>,
    pub plan: Option<Plan>,
    /// New-style plan items from `plan_items` table.
    pub plan_items: Vec<PlanItem>,
    pub alarms: Vec<AlarmDefinition>,
    pub reminders: Vec<ReminderDefinition>,
    pub timers: Vec<TimerDefinition>,
    pub context_documents: Vec<ContextDocument>,
    pub generated_at: AdiyutantDateTime,
}

impl TodayState {
    /// Return the morning check-in, if present.
    pub fn morning_check_in(&self) -> Option<&CheckIn> {
        self.check_ins
            .iter()
            .find(|c| c.check_in_type == CheckInType::Morning)
    }

    /// Return the evening or shutdown check-in, if present.
    pub fn evening_check_in(&self) -> Option<&CheckIn> {
        self.check_ins.iter().find(|c| {
            c.check_in_type == CheckInType::Evening || c.check_in_type == CheckInType::Shutdown
        })
    }

    /// Whether a plan with at least one item exists for today.
    ///
    /// Checks both legacy `plan` and new `plan_items`.
    pub fn has_plan(&self) -> bool {
        let legacy_has = self.plan.as_ref().is_some_and(|p| !p.items.is_empty());
        let new_has = !self.plan_items.is_empty();
        legacy_has || new_has
    }

    /// How many habit events have been logged today.
    pub fn habit_event_count(&self) -> usize {
        self.habit_events.len()
    }

    /// How many plan items (legacy + new) are not yet completed.
    pub fn pending_plan_items(&self) -> usize {
        let legacy_count = self.plan.as_ref().map_or(0, |p| {
            p.items
                .iter()
                .filter(|i| !matches!(i.status, crate::model::plan::PlanItemStatus::Done))
                .count()
        });
        let new_count = self
            .plan_items
            .iter()
            .filter(|i| {
                !matches!(
                    i.status,
                    crate::model::day_plan::PlanItemStatus::Done
                        | crate::model::day_plan::PlanItemStatus::Cancelled
                        | crate::model::day_plan::PlanItemStatus::Archived
                )
            })
            .count();
        legacy_count + new_count
    }
}

/// Builder for constructing a `TodayState` piece by piece.
#[derive(Debug, Default)]
pub struct TodayStateBuilder {
    date: Option<chrono::NaiveDate>,
    daily_log: Option<DailyLog>,
    check_ins: Vec<CheckIn>,
    habits: Vec<Habit>,
    habit_events: Vec<HabitEvent>,
    plan: Option<Plan>,
    plan_items: Vec<PlanItem>,
    alarms: Vec<AlarmDefinition>,
    reminders: Vec<ReminderDefinition>,
    timers: Vec<TimerDefinition>,
    context_documents: Vec<ContextDocument>,
}

impl TodayStateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn date(mut self, date: chrono::NaiveDate) -> Self {
        self.date = Some(date);
        self
    }

    pub fn daily_log(mut self, log: DailyLog) -> Self {
        self.daily_log = Some(log);
        self
    }

    pub fn check_in(mut self, ci: CheckIn) -> Self {
        self.check_ins.push(ci);
        self
    }

    pub fn check_ins(mut self, cis: Vec<CheckIn>) -> Self {
        self.check_ins = cis;
        self
    }

    pub fn habit(mut self, h: Habit) -> Self {
        self.habits.push(h);
        self
    }

    pub fn habits(mut self, habits: Vec<Habit>) -> Self {
        self.habits = habits;
        self
    }

    pub fn habit_event(mut self, he: HabitEvent) -> Self {
        self.habit_events.push(he);
        self
    }

    pub fn habit_events(mut self, events: Vec<HabitEvent>) -> Self {
        self.habit_events = events;
        self
    }

    pub fn plan(mut self, p: Plan) -> Self {
        self.plan = Some(p);
        self
    }

    /// Add new-style plan items.
    pub fn plan_items(mut self, items: Vec<PlanItem>) -> Self {
        self.plan_items = items;
        self
    }

    pub fn plan_item(mut self, item: PlanItem) -> Self {
        self.plan_items.push(item);
        self
    }

    pub fn alarm(mut self, a: AlarmDefinition) -> Self {
        self.alarms.push(a);
        self
    }

    pub fn alarms(mut self, alarms: Vec<AlarmDefinition>) -> Self {
        self.alarms = alarms;
        self
    }

    pub fn reminder(mut self, r: ReminderDefinition) -> Self {
        self.reminders.push(r);
        self
    }

    pub fn reminders(mut self, reminders: Vec<ReminderDefinition>) -> Self {
        self.reminders = reminders;
        self
    }

    pub fn timer(mut self, t: TimerDefinition) -> Self {
        self.timers.push(t);
        self
    }

    pub fn timers(mut self, timers: Vec<TimerDefinition>) -> Self {
        self.timers = timers;
        self
    }

    pub fn context_document(mut self, cd: ContextDocument) -> Self {
        self.context_documents.push(cd);
        self
    }

    pub fn context_documents(mut self, docs: Vec<ContextDocument>) -> Self {
        self.context_documents = docs;
        self
    }

    /// Build the `TodayState`.
    pub fn build(self) -> TodayState {
        let date = self.date.unwrap_or_else(|| chrono::Utc::now().date_naive());
        TodayState {
            date,
            daily_log: self.daily_log,
            check_ins: self.check_ins,
            habits: self.habits,
            habit_events: self.habit_events,
            plan: self.plan,
            plan_items: self.plan_items,
            alarms: self.alarms,
            reminders: self.reminders,
            timers: self.timers,
            context_documents: self.context_documents,
            generated_at: AdiyutantDateTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::check_in::CheckInType;
    use crate::model::daily_log::DailyLog;
    use crate::model::plan::PlanItemStatus;

    #[test]
    fn builder_default_date_is_today() {
        let state = TodayStateBuilder::new().build();
        let today = chrono::Utc::now().date_naive();
        assert_eq!(state.date, today);
    }

    #[test]
    fn builder_accepts_custom_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let state = TodayStateBuilder::new().date(date).build();
        assert_eq!(state.date, date);
    }

    #[test]
    fn builder_accepts_daily_log() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log = DailyLog::new(date);
        let state = TodayStateBuilder::new()
            .date(date)
            .daily_log(log.clone())
            .build();
        assert_eq!(state.daily_log.unwrap().id, log.id);
    }

    #[test]
    fn builder_accepts_check_ins() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log = DailyLog::new(date);
        let ci = CheckIn::new(log.id, CheckInType::Morning, "hello".into());
        let state = TodayStateBuilder::new().date(date).check_in(ci).build();
        assert_eq!(state.check_ins.len(), 1);
        assert_eq!(state.check_ins[0].check_in_type, CheckInType::Morning);
    }

    #[test]
    fn builder_accepts_habits() {
        let h = Habit::new("read".into());
        let state = TodayStateBuilder::new().habit(h).build();
        assert_eq!(state.habits.len(), 1);
        assert_eq!(state.habits[0].name, "read");
    }

    #[test]
    fn morning_check_in_finds_correct() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log = DailyLog::new(date);
        let morning = CheckIn::new(log.id, CheckInType::Morning, "morning".into());
        let day = CheckIn::new(log.id, CheckInType::Day, "day".into());
        let state = TodayStateBuilder::new()
            .date(date)
            .daily_log(log)
            .check_in(day)
            .check_in(morning)
            .build();
        assert!(state.morning_check_in().is_some());
        assert_eq!(state.morning_check_in().unwrap().raw_input, "morning");
    }

    #[test]
    fn has_plan_false_when_empty() {
        let state = TodayStateBuilder::new().build();
        assert!(!state.has_plan());
    }

    #[test]
    fn has_plan_true_with_legacy_items() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log = DailyLog::new(date);
        let mut plan = Plan::new(log.id, "Today".into());
        plan.add_item("Do something".into());
        let state = TodayStateBuilder::new().date(date).plan(plan).build();
        assert!(state.has_plan());
    }

    #[test]
    fn has_plan_true_with_new_plan_items() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log_id = crate::id::Id::<DailyLog>::new();
        let item = PlanItem::new(log_id, "New task".into());
        let state = TodayStateBuilder::new().date(date).plan_item(item).build();
        assert!(state.has_plan());
    }

    #[test]
    fn pending_plan_items_counts_correctly() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log = DailyLog::new(date);
        let mut plan = Plan::new(log.id, "Today".into());
        plan.add_item("Task 1".into());
        plan.add_item("Task 2".into());
        plan.items[0].status = PlanItemStatus::Done;
        let state = TodayStateBuilder::new().date(date).plan(plan).build();
        assert_eq!(state.pending_plan_items(), 1);
    }

    #[test]
    fn evening_check_in_finds_shutdown() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let log = DailyLog::new(date);
        let shutdown = CheckIn::new(log.id, CheckInType::Shutdown, "g night".into());
        let state = TodayStateBuilder::new()
            .date(date)
            .check_in(shutdown)
            .build();
        assert!(state.evening_check_in().is_some());
    }
}
