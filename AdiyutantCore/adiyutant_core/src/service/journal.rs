use super::*;
use crate::dto::JournalEntryDto;
use crate::model::journal_entry::{JournalEntry, JournalEntryType};

impl AdiyutantCoreService {
    /// List journal entries for today.
    pub fn get_journal(&self) -> Result<Vec<JournalEntryDto>, crate::error::CoreError> {
        let today = self.time.today();
        let entries = if let Some(log) = self.store.get_daily_log_by_date(today)? {
            self.store.list_journal_entries_by_log(log.id)?
        } else {
            vec![]
        };
        Ok(entries
            .iter()
            .map(|e| JournalEntryDto {
                id: e.id.value().to_string(),
                entry_type: e.entry_type.as_str().to_string(),
                summary: e.summary.clone(),
                timestamp: e.timestamp.inner().to_rfc3339(),
            })
            .collect())
    }

    /// Add a journal entry for today.
    pub fn add_journal_entry(
        &self,
        entry_type: &str,
        summary: &str,
    ) -> Result<JournalEntryDto, crate::error::CoreError> {
        let entry_type = JournalEntryType::from_str(entry_type).ok_or_else(|| {
            crate::error::CoreError::InvalidInput(format!("unknown entry type: {entry_type}"))
        })?;
        let daily_log_id = self.get_or_create_today_log()?;
        let entry = JournalEntry::new(daily_log_id, entry_type, summary.to_string());
        self.store.insert_journal_entry(&entry)?;
        Ok(JournalEntryDto {
            id: entry.id.value().to_string(),
            entry_type: entry.entry_type.as_str().to_string(),
            summary: entry.summary,
            timestamp: entry.timestamp.inner().to_rfc3339(),
        })
    }
}
