use super::*;
use crate::dto::{AlarmDto, ContextDocumentDto, HabitDto, HabitEventDto, ReminderDto, TimerDto};
use crate::model::alarm::AlarmDefinition;
use crate::model::context_document::{ContextDocument, ContextDocumentType};
use crate::model::habit::Habit;
use crate::model::habit_event::{HabitEvent, HabitEventLevel, HabitEventStatus};
use crate::model::reminder::ReminderDefinition;
use crate::model::timer::{TimerDefinition, TimerMode};

impl AdiyutantCoreService {
    pub fn add_habit(&self, name: &str) -> Result<HabitDto, crate::error::CoreError> {
        let habit = Habit::new(name.to_string());
        self.store.insert_habit(&habit)?;
        Ok(HabitDto {
            id: habit.id.value().to_string(),
            name: habit.name.clone(),
            is_active: habit.is_active,
        })
    }

    pub fn list_habits(&self) -> Result<Vec<HabitDto>, crate::error::CoreError> {
        let habits = self.store.list_habits()?;
        Ok(habits
            .iter()
            .map(|h| HabitDto {
                id: h.id.value().to_string(),
                name: h.name.clone(),
                is_active: h.is_active,
            })
            .collect())
    }

    pub fn mark_habit_done(
        &self,
        habit_id: &str,
        level: Option<&str>,
    ) -> Result<HabitEventDto, crate::error::CoreError> {
        let id = crate::service::helpers::parse_id::<Habit>(habit_id)?;
        let habit = self
            .store
            .get_habit(id)?
            .ok_or_else(|| crate::error::CoreError::NotFound(habit_id.to_string()))?;
        let event_level = level
            .and_then(|l| match l {
                "min" => Some(HabitEventLevel::Min),
                "light" => Some(HabitEventLevel::Light),
                "base" => Some(HabitEventLevel::Base),
                "full" => Some(HabitEventLevel::Full),
                _ => None,
            })
            .unwrap_or(HabitEventLevel::Base);
        let event = HabitEvent::new(id, HabitEventStatus::Done, event_level);
        self.store.insert_habit_event(&event)?;
        Ok(HabitEventDto {
            id: event.id.value().to_string(),
            habit_id: habit_id.to_string(),
            habit_name: habit.name.clone(),
            status: crate::dto::habit_event_status_to_string(true).to_string(),
            level: crate::dto::habit_level_to_string(&event_level).to_string(),
            date_time: event.date_time.inner().to_rfc3339(),
        })
    }

    pub fn skip_habit(&self, habit_id: &str) -> Result<HabitEventDto, crate::error::CoreError> {
        let id = crate::service::helpers::parse_id::<Habit>(habit_id)?;
        let habit = self
            .store
            .get_habit(id)?
            .ok_or_else(|| crate::error::CoreError::NotFound(habit_id.to_string()))?;
        let event = HabitEvent::new(id, HabitEventStatus::Skipped, HabitEventLevel::Min);
        self.store.insert_habit_event(&event)?;
        Ok(HabitEventDto {
            id: event.id.value().to_string(),
            habit_id: habit_id.to_string(),
            habit_name: habit.name.clone(),
            status: crate::dto::habit_event_status_to_string(false).to_string(),
            level: crate::dto::habit_level_to_string(&HabitEventLevel::Min).to_string(),
            date_time: event.date_time.inner().to_rfc3339(),
        })
    }

    pub fn add_timer(
        &self,
        title: &str,
        duration_seconds: u64,
    ) -> Result<TimerDto, crate::error::CoreError> {
        let timer = TimerDefinition::new(title.to_string(), duration_seconds, TimerMode::Focus);
        self.store.insert_timer(&timer)?;
        Ok(TimerDto {
            id: timer.id.value().to_string(),
            title: timer.title.clone(),
            duration_seconds: timer.duration_seconds,
            mode: crate::dto::timer_mode_to_string(&timer.mode).to_string(),
        })
    }

    pub fn list_timers(&self) -> Result<Vec<TimerDto>, crate::error::CoreError> {
        let timers = self.store.list_timers()?;
        Ok(timers
            .iter()
            .map(|t| TimerDto {
                id: t.id.value().to_string(),
                title: t.title.clone(),
                duration_seconds: t.duration_seconds,
                mode: crate::dto::timer_mode_to_string(&t.mode).to_string(),
            })
            .collect())
    }

    pub fn add_reminder(
        &self,
        title: &str,
        schedule_rule: &str,
    ) -> Result<ReminderDto, crate::error::CoreError> {
        let reminder = ReminderDefinition::new(title.to_string(), schedule_rule.to_string());
        self.store.insert_reminder(&reminder)?;
        Ok(ReminderDto {
            id: reminder.id.value().to_string(),
            title: reminder.title.clone(),
            schedule_rule: reminder.schedule_rule.clone(),
        })
    }

    pub fn list_reminders(&self) -> Result<Vec<ReminderDto>, crate::error::CoreError> {
        let reminders = self.store.list_reminders()?;
        Ok(reminders
            .iter()
            .map(|r| ReminderDto {
                id: r.id.value().to_string(),
                title: r.title.clone(),
                schedule_rule: r.schedule_rule.clone(),
            })
            .collect())
    }

    pub fn add_alarm(&self, title: &str, time: &str) -> Result<AlarmDto, crate::error::CoreError> {
        let parsed_time = chrono::NaiveTime::parse_from_str(time, "%H:%M")
            .map_err(|e| crate::error::CoreError::InvalidInput(format!("invalid time: {e}")))?;
        let alarm = AlarmDefinition::new(title.to_string(), parsed_time);
        self.store.insert_alarm(&alarm)?;
        Ok(AlarmDto {
            id: alarm.id.value().to_string(),
            title: alarm.title.clone(),
            time: alarm.time.format("%H:%M").to_string(),
        })
    }

    pub fn list_alarms(&self) -> Result<Vec<AlarmDto>, crate::error::CoreError> {
        let alarms = self.store.list_alarms()?;
        Ok(alarms
            .iter()
            .map(|a| AlarmDto {
                id: a.id.value().to_string(),
                title: a.title.clone(),
                time: a.time.format("%H:%M").to_string(),
            })
            .collect())
    }

    pub fn add_context_document(
        &self,
        doc_type: &str,
        title: &str,
        content: &str,
    ) -> Result<ContextDocumentDto, crate::error::CoreError> {
        let dt: ContextDocumentType =
            serde_json::from_value(serde_json::json!(doc_type)).map_err(|_| {
                crate::error::CoreError::InvalidInput(format!("unknown doc_type: {doc_type}"))
            })?;
        let doc = ContextDocument::new(dt, title.to_string(), content.to_string());
        self.store.insert_context_document(&doc)?;
        Ok(ContextDocumentDto {
            id: doc.id.value().to_string(),
            doc_type: crate::dto::context_doc_type_to_string(&doc.doc_type).to_string(),
            title: doc.title.clone(),
            content: doc.content_markdown.clone(),
        })
    }

    pub fn list_context_documents(
        &self,
    ) -> Result<Vec<ContextDocumentDto>, crate::error::CoreError> {
        let docs = self.store.list_context_documents()?;
        Ok(docs
            .iter()
            .map(|d| ContextDocumentDto {
                id: d.id.value().to_string(),
                doc_type: crate::dto::context_doc_type_to_string(&d.doc_type).to_string(),
                title: d.title.clone(),
                content: d.content_markdown.clone(),
            })
            .collect())
    }
}
