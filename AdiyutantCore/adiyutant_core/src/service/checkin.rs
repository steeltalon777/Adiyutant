use super::*;
use crate::dto::CheckInDto;
use crate::model::check_in::{CheckIn, CheckInType};
use crate::model::journal_entry::{JournalEntry, JournalEntryType};

impl AdiyutantCoreService {
    /// Create a check-in and optionally update daily log metrics (transactional).
    pub fn create_checkin(
        &self,
        checkin_type: &str,
        text: &str,
        sleep_score: Option<u8>,
        energy: Option<u8>,
        mood: Option<u8>,
    ) -> Result<CheckInDto, crate::error::CoreError> {
        let daily_log_id = self.get_or_create_today_log()?;
        let ci_type = match checkin_type {
            "morning" => CheckInType::Morning,
            "day" => CheckInType::Day,
            "evening" => CheckInType::Evening,
            "shutdown" => CheckInType::Shutdown,
            _ => {
                return Err(crate::error::CoreError::InvalidInput(format!(
                    "unknown checkin type: {checkin_type}"
                )));
            }
        };
        let ci = CheckIn::new(daily_log_id, ci_type, text.to_string());

        // Build optional daily log update
        let daily_log_update = if sleep_score.is_some() || energy.is_some() || mood.is_some() {
            self.store.get_daily_log(daily_log_id)?.map(|mut log| {
                if let Some(s) = sleep_score {
                    log.sleep_score = Some(s);
                }
                if let Some(e) = energy {
                    log.energy = Some(e);
                }
                if let Some(m) = mood {
                    log.mood = Some(m);
                }
                log
            })
        } else {
            None
        };

        let entry = JournalEntry::new(
            daily_log_id,
            JournalEntryType::CheckInCreated,
            format!("{checkin_type} check-in: {text}"),
        );

        // Atomic: insert_check_in + update_daily_log + insert_journal_entry
        self.store
            .insert_checkin_composite(&ci, daily_log_update.as_ref(), &entry)?;
        Ok(CheckInDto {
            id: ci.id.value().to_string(),
            checkin_type: checkin_type.to_string(),
            text: text.to_string(),
            created_at: ci.created_at.inner().to_rfc3339(),
        })
    }
}
