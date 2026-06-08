use adiyutant_core::error::CoreError;
use adiyutant_core::id::Id;
use adiyutant_core::model::action_proposal::ProposalStatus;
use adiyutant_core::model::*;
use chrono::NaiveDate;
use chrono::NaiveTime;
use rusqlite::Connection;
use rusqlite::params;

// ──────────────────────────────────────────────
// Store trait — all 10 domain entities
// ──────────────────────────────────────────────

pub trait Store {
    type Error;

    // Bootstrap
    fn migrate(&self) -> Result<(), Self::Error>;
    fn health_check(&self) -> Result<(), Self::Error>;

    // ── DailyLog ──
    fn insert_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error>;
    fn get_daily_log(&self, id: Id<DailyLog>) -> Result<Option<DailyLog>, Self::Error>;
    fn get_daily_log_by_date(&self, date: NaiveDate) -> Result<Option<DailyLog>, Self::Error>;
    fn list_daily_logs(&self) -> Result<Vec<DailyLog>, Self::Error>;
    fn update_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error>;
    fn delete_daily_log(&self, id: Id<DailyLog>) -> Result<(), Self::Error>;

    // ── CheckIn ──
    fn insert_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error>;
    fn get_check_in(&self, id: Id<CheckIn>) -> Result<Option<CheckIn>, Self::Error>;
    fn list_check_ins_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<CheckIn>, Self::Error>;
    fn update_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error>;
    fn delete_check_in(&self, id: Id<CheckIn>) -> Result<(), Self::Error>;

    // ── Habit ──
    fn insert_habit(&self, h: &Habit) -> Result<(), Self::Error>;
    fn get_habit(&self, id: Id<Habit>) -> Result<Option<Habit>, Self::Error>;
    fn list_habits(&self) -> Result<Vec<Habit>, Self::Error>;
    fn update_habit(&self, h: &Habit) -> Result<(), Self::Error>;
    fn delete_habit(&self, id: Id<Habit>) -> Result<(), Self::Error>;

    // ── HabitEvent ──
    fn insert_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error>;
    fn get_habit_event(&self, id: Id<HabitEvent>) -> Result<Option<HabitEvent>, Self::Error>;
    fn list_habit_events_by_habit(
        &self,
        habit_id: Id<Habit>,
    ) -> Result<Vec<HabitEvent>, Self::Error>;
    fn update_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error>;
    fn delete_habit_event(&self, id: Id<HabitEvent>) -> Result<(), Self::Error>;

    // ── ReminderDefinition ──
    fn insert_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error>;
    fn get_reminder(
        &self,
        id: Id<ReminderDefinition>,
    ) -> Result<Option<ReminderDefinition>, Self::Error>;
    fn list_reminders(&self) -> Result<Vec<ReminderDefinition>, Self::Error>;
    fn update_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error>;
    fn delete_reminder(&self, id: Id<ReminderDefinition>) -> Result<(), Self::Error>;

    // ── AlarmDefinition ──
    fn insert_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error>;
    fn get_alarm(&self, id: Id<AlarmDefinition>) -> Result<Option<AlarmDefinition>, Self::Error>;
    fn list_alarms(&self) -> Result<Vec<AlarmDefinition>, Self::Error>;
    fn update_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error>;
    fn delete_alarm(&self, id: Id<AlarmDefinition>) -> Result<(), Self::Error>;

    // ── TimerDefinition ──
    fn insert_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error>;
    fn get_timer(&self, id: Id<TimerDefinition>) -> Result<Option<TimerDefinition>, Self::Error>;
    fn list_timers(&self) -> Result<Vec<TimerDefinition>, Self::Error>;
    fn update_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error>;
    fn delete_timer(&self, id: Id<TimerDefinition>) -> Result<(), Self::Error>;

    // ── ContextDocument ──
    fn insert_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error>;
    fn get_context_document(
        &self,
        id: Id<ContextDocument>,
    ) -> Result<Option<ContextDocument>, Self::Error>;
    fn list_context_documents(&self) -> Result<Vec<ContextDocument>, Self::Error>;
    fn update_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error>;
    fn delete_context_document(&self, id: Id<ContextDocument>) -> Result<(), Self::Error>;

    // ── Plan ──
    fn insert_plan(&self, p: &Plan) -> Result<(), Self::Error>;
    fn get_plan(&self, id: Id<Plan>) -> Result<Option<Plan>, Self::Error>;
    fn get_plan_by_daily_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Option<Plan>, Self::Error>;
    fn update_plan(&self, p: &Plan) -> Result<(), Self::Error>;
    fn delete_plan(&self, id: Id<Plan>) -> Result<(), Self::Error>;

    // ── ActionProposal ──
    fn insert_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error>;
    fn get_action_proposal(
        &self,
        id: Id<ActionProposal>,
    ) -> Result<Option<ActionProposal>, Self::Error>;
    fn list_action_proposals(&self) -> Result<Vec<ActionProposal>, Self::Error>;
    fn list_action_proposals_by_status(
        &self,
        status: ProposalStatus,
    ) -> Result<Vec<ActionProposal>, Self::Error>;
    fn update_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error>;
    fn delete_action_proposal(&self, id: Id<ActionProposal>) -> Result<(), Self::Error>;
}

// ──────────────────────────────────────────────
// Schema bootstrap
// ──────────────────────────────────────────────

fn run_migration(conn: &Connection) -> Result<(), CoreError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS daily_logs (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            mode TEXT,
            sleep_score INTEGER,
            energy INTEGER,
            mood INTEGER,
            raw_notes TEXT,
            ai_summary TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS check_ins (
            id TEXT PRIMARY KEY,
            daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
            check_in_type TEXT NOT NULL,
            raw_input TEXT NOT NULL,
            structured_data TEXT,
            agent_response TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS habits (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            frequency TEXT,
            target TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS habit_events (
            id TEXT PRIMARY KEY,
            habit_id TEXT NOT NULL REFERENCES habits(id),
            date_time TEXT NOT NULL,
            status TEXT NOT NULL,
            level TEXT NOT NULL,
            comment TEXT
        );

        CREATE TABLE IF NOT EXISTS reminders (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            message TEXT,
            schedule_rule TEXT NOT NULL,
            next_fire_at TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS alarms (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            time TEXT NOT NULL,
            repeat_rule TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            platform_binding_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS timers (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            duration_seconds INTEGER NOT NULL,
            mode TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS context_documents (
            id TEXT PRIMARY KEY,
            doc_type TEXT NOT NULL,
            title TEXT NOT NULL,
            content_markdown TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS plans (
            id TEXT PRIMARY KEY,
            daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
            title TEXT NOT NULL,
            items_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS action_proposals (
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            proposal_type TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            applied_at TEXT
        );
        ",
    )
    .map_err(|e| CoreError::Storage(e.to_string()))
}

// ──────────────────────────────────────────────
// Mapping helpers
// ──────────────────────────────────────────────

fn uuid_to_string<T>(id: Id<T>) -> String {
    id.value().to_string()
}

fn parse_uuid<T>(s: &str) -> Result<Id<T>, CoreError> {
    let uuid = uuid::Uuid::parse_str(s).map_err(|e| CoreError::InvalidInput(e.to_string()))?;
    // Id<T> serializes as {"value": "uuid-string", "_marker": null}.
    // Build the JSON value directly rather than going through string format.
    let value = serde_json::json!({"value": uuid, "_marker": null});
    serde_json::from_value(value).map_err(|e| CoreError::InvalidInput(e.to_string()))
}

fn datetime_to_string(dt: &adiyutant_core::datetime::AdiyutantDateTime) -> String {
    dt.inner().to_rfc3339()
}

// AdiyutantDateTime doesn't have a from_chrono constructor as of phase 3.
// We use serde round-trip through JSON instead.
fn deserialize_datetime(s: &str) -> Result<adiyutant_core::datetime::AdiyutantDateTime, CoreError> {
    // Build a JSON string: "2026-06-08T12:00:00+00:00"
    let json_str = format!("\"{}\"", s);
    serde_json::from_str(&json_str).map_err(|e| CoreError::InvalidInput(e.to_string()))
}

fn bool_to_int(b: bool) -> i64 {
    if b { 1 } else { 0 }
}

fn int_to_bool(i: i64) -> bool {
    i != 0
}

fn option_json_to_string<T: serde::Serialize>(val: &Option<T>) -> Option<String> {
    val.as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default())
}

fn option_json_from_str<T: serde::de::DeserializeOwned>(
    s: Option<String>,
) -> Result<Option<T>, CoreError> {
    match s {
        None => Ok(None),
        Some(ref v) if v.is_empty() || v == "null" => Ok(None),
        Some(v) => serde_json::from_str(&v)
            .map(Some)
            .map_err(|e| CoreError::Internal(e.to_string())),
    }
}

fn json_to_string<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string(val).unwrap_or_else(|_| "[]".to_string())
}

fn json_from_str<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, CoreError> {
    serde_json::from_str(s).map_err(|e| CoreError::Internal(e.to_string()))
}

/// Helper to extract a column from a SQLite row, mapping rusqlite errors to CoreError.
fn row_get<T: rusqlite::types::FromSql>(
    row: &rusqlite::Row<'_>,
    idx: usize,
) -> Result<T, CoreError> {
    row.get(idx).map_err(|e| CoreError::Storage(e.to_string()))
}

// ──────────────────────────────────────────────
// SqliteStore
// ──────────────────────────────────────────────

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn new(path: &str) -> Result<Self, CoreError> {
        let conn = Connection::open(path).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { conn })
    }

    pub fn new_in_memory() -> Result<Self, CoreError> {
        let conn = Connection::open_in_memory().map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { conn })
    }
}

impl Store for SqliteStore {
    type Error = CoreError;

    fn migrate(&self) -> Result<(), Self::Error> {
        run_migration(&self.conn)
    }

    fn health_check(&self) -> Result<(), Self::Error> {
        self.conn
            .query_row("SELECT 1", [], |_row| Ok(()))
            .map_err(|e| CoreError::Storage(e.to_string()))
    }

    // ── DailyLog ──

    fn insert_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO daily_logs (id, date, mode, sleep_score, energy, mood, raw_notes, ai_summary, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    uuid_to_string(log.id),
                    log.date.to_string(),
                    log.mode,
                    log.sleep_score.map(|v| v as i64),
                    log.energy.map(|v| v as i64),
                    log.mood.map(|v| v as i64),
                    log.raw_notes,
                    log.ai_summary,
                    datetime_to_string(&log.created_at),
                    datetime_to_string(&log.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_daily_log(&self, id: Id<DailyLog>) -> Result<Option<DailyLog>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, date, mode, sleep_score, energy, mood, raw_notes, ai_summary, created_at, updated_at FROM daily_logs WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_daily_log(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(log)) => Ok(Some(log?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn get_daily_log_by_date(&self, date: NaiveDate) -> Result<Option<DailyLog>, Self::Error> {
        let date_str = date.to_string();
        let mut stmt = self
            .conn
            .prepare("SELECT id, date, mode, sleep_score, energy, mood, raw_notes, ai_summary, created_at, updated_at FROM daily_logs WHERE date = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![date_str], |row| Ok(row_to_daily_log(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(log)) => Ok(Some(log?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_daily_logs(&self) -> Result<Vec<DailyLog>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, date, mode, sleep_score, energy, mood, raw_notes, ai_summary, created_at, updated_at FROM daily_logs ORDER BY date DESC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_daily_log(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE daily_logs SET date = ?1, mode = ?2, sleep_score = ?3, energy = ?4, mood = ?5, raw_notes = ?6, ai_summary = ?7, updated_at = ?8 WHERE id = ?9",
                params![
                    log.date.to_string(),
                    log.mode,
                    log.sleep_score.map(|v| v as i64),
                    log.energy.map(|v| v as i64),
                    log.mood.map(|v| v as i64),
                    log.raw_notes,
                    log.ai_summary,
                    datetime_to_string(&log.updated_at),
                    uuid_to_string(log.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("daily_log".into()));
        }
        Ok(())
    }

    fn delete_daily_log(&self, id: Id<DailyLog>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM daily_logs WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("daily_log".into()));
        }
        Ok(())
    }

    // ── CheckIn ──

    fn insert_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error> {
        let structured_data_str = option_json_to_string(&ci.structured_data);
        self.conn
            .execute(
                "INSERT INTO check_ins (id, daily_log_id, check_in_type, raw_input, structured_data, agent_response, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    uuid_to_string(ci.id),
                    uuid_to_string(ci.daily_log_id),
                    json_to_string(&ci.check_in_type),
                    ci.raw_input,
                    structured_data_str,
                    ci.agent_response,
                    datetime_to_string(&ci.created_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_check_in(&self, id: Id<CheckIn>) -> Result<Option<CheckIn>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, check_in_type, raw_input, structured_data, agent_response, created_at FROM check_ins WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_check_in(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(ci)) => Ok(Some(ci?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_check_ins_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<CheckIn>, Self::Error> {
        let log_id_str = uuid_to_string(daily_log_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, check_in_type, raw_input, structured_data, agent_response, created_at FROM check_ins WHERE daily_log_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![log_id_str], |row| Ok(row_to_check_in(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error> {
        let structured_data_str = option_json_to_string(&ci.structured_data);
        let affected = self
            .conn
            .execute(
                "UPDATE check_ins SET daily_log_id = ?1, check_in_type = ?2, raw_input = ?3, structured_data = ?4, agent_response = ?5 WHERE id = ?6",
                params![
                    uuid_to_string(ci.daily_log_id),
                    json_to_string(&ci.check_in_type),
                    ci.raw_input,
                    structured_data_str,
                    ci.agent_response,
                    uuid_to_string(ci.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("check_in".into()));
        }
        Ok(())
    }

    fn delete_check_in(&self, id: Id<CheckIn>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM check_ins WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("check_in".into()));
        }
        Ok(())
    }

    // ── Habit ──

    fn insert_habit(&self, h: &Habit) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO habits (id, name, description, frequency, target, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    uuid_to_string(h.id),
                    h.name,
                    h.description,
                    h.frequency,
                    h.target,
                    bool_to_int(h.is_active),
                    datetime_to_string(&h.created_at),
                    datetime_to_string(&h.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_habit(&self, id: Id<Habit>) -> Result<Option<Habit>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, frequency, target, is_active, created_at, updated_at FROM habits WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_habit(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(h)) => Ok(Some(h?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_habits(&self) -> Result<Vec<Habit>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, frequency, target, is_active, created_at, updated_at FROM habits ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_habit(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_habit(&self, h: &Habit) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE habits SET name = ?1, description = ?2, frequency = ?3, target = ?4, is_active = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    h.name,
                    h.description,
                    h.frequency,
                    h.target,
                    bool_to_int(h.is_active),
                    datetime_to_string(&h.updated_at),
                    uuid_to_string(h.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("habit".into()));
        }
        Ok(())
    }

    fn delete_habit(&self, id: Id<Habit>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM habits WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("habit".into()));
        }
        Ok(())
    }

    // ── HabitEvent ──

    fn insert_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO habit_events (id, habit_id, date_time, status, level, comment) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    uuid_to_string(he.id),
                    uuid_to_string(he.habit_id),
                    datetime_to_string(&he.date_time),
                    json_to_string(&he.status),
                    json_to_string(&he.level),
                    he.comment,
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_habit_event(&self, id: Id<HabitEvent>) -> Result<Option<HabitEvent>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, habit_id, date_time, status, level, comment FROM habit_events WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_habit_event(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(he)) => Ok(Some(he?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_habit_events_by_habit(
        &self,
        habit_id: Id<Habit>,
    ) -> Result<Vec<HabitEvent>, Self::Error> {
        let habit_id_str = uuid_to_string(habit_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, habit_id, date_time, status, level, comment FROM habit_events WHERE habit_id = ?1 ORDER BY date_time DESC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![habit_id_str], |row| Ok(row_to_habit_event(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE habit_events SET habit_id = ?1, date_time = ?2, status = ?3, level = ?4, comment = ?5 WHERE id = ?6",
                params![
                    uuid_to_string(he.habit_id),
                    datetime_to_string(&he.date_time),
                    json_to_string(&he.status),
                    json_to_string(&he.level),
                    he.comment,
                    uuid_to_string(he.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("habit_event".into()));
        }
        Ok(())
    }

    fn delete_habit_event(&self, id: Id<HabitEvent>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM habit_events WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("habit_event".into()));
        }
        Ok(())
    }

    // ── ReminderDefinition ──

    fn insert_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO reminders (id, title, message, schedule_rule, next_fire_at, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    uuid_to_string(r.id),
                    r.title,
                    r.message,
                    r.schedule_rule,
                    r.next_fire_at.map(|dt| datetime_to_string(&dt)),
                    bool_to_int(r.enabled),
                    datetime_to_string(&r.created_at),
                    datetime_to_string(&r.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_reminder(
        &self,
        id: Id<ReminderDefinition>,
    ) -> Result<Option<ReminderDefinition>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, message, schedule_rule, next_fire_at, enabled, created_at, updated_at FROM reminders WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_reminder(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(r)) => Ok(Some(r?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_reminders(&self) -> Result<Vec<ReminderDefinition>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, message, schedule_rule, next_fire_at, enabled, created_at, updated_at FROM reminders ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_reminder(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE reminders SET title = ?1, message = ?2, schedule_rule = ?3, next_fire_at = ?4, enabled = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    r.title,
                    r.message,
                    r.schedule_rule,
                    r.next_fire_at.map(|dt| datetime_to_string(&dt)),
                    bool_to_int(r.enabled),
                    datetime_to_string(&r.updated_at),
                    uuid_to_string(r.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("reminder".into()));
        }
        Ok(())
    }

    fn delete_reminder(&self, id: Id<ReminderDefinition>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM reminders WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("reminder".into()));
        }
        Ok(())
    }

    // ── AlarmDefinition ──

    fn insert_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO alarms (id, title, time, repeat_rule, enabled, platform_binding_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    uuid_to_string(a.id),
                    a.title,
                    a.time.to_string(),
                    a.repeat_rule,
                    bool_to_int(a.enabled),
                    a.platform_binding_id,
                    datetime_to_string(&a.created_at),
                    datetime_to_string(&a.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_alarm(&self, id: Id<AlarmDefinition>) -> Result<Option<AlarmDefinition>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, time, repeat_rule, enabled, platform_binding_id, created_at, updated_at FROM alarms WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_alarm(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(a)) => Ok(Some(a?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_alarms(&self) -> Result<Vec<AlarmDefinition>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, time, repeat_rule, enabled, platform_binding_id, created_at, updated_at FROM alarms ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_alarm(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE alarms SET title = ?1, time = ?2, repeat_rule = ?3, enabled = ?4, platform_binding_id = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    a.title,
                    a.time.to_string(),
                    a.repeat_rule,
                    bool_to_int(a.enabled),
                    a.platform_binding_id,
                    datetime_to_string(&a.updated_at),
                    uuid_to_string(a.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("alarm".into()));
        }
        Ok(())
    }

    fn delete_alarm(&self, id: Id<AlarmDefinition>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM alarms WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("alarm".into()));
        }
        Ok(())
    }

    // ── TimerDefinition ──

    fn insert_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO timers (id, title, duration_seconds, mode, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    uuid_to_string(t.id),
                    t.title,
                    t.duration_seconds as i64,
                    json_to_string(&t.mode),
                    datetime_to_string(&t.created_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_timer(&self, id: Id<TimerDefinition>) -> Result<Option<TimerDefinition>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, duration_seconds, mode, created_at FROM timers WHERE id = ?1",
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_timer(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(t)) => Ok(Some(t?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_timers(&self) -> Result<Vec<TimerDefinition>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, duration_seconds, mode, created_at FROM timers ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_timer(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE timers SET title = ?1, duration_seconds = ?2, mode = ?3 WHERE id = ?4",
                params![
                    t.title,
                    t.duration_seconds as i64,
                    json_to_string(&t.mode),
                    uuid_to_string(t.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("timer".into()));
        }
        Ok(())
    }

    fn delete_timer(&self, id: Id<TimerDefinition>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM timers WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("timer".into()));
        }
        Ok(())
    }

    // ── ContextDocument ──

    fn insert_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO context_documents (id, doc_type, title, content_markdown, version, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    uuid_to_string(cd.id),
                    json_to_string(&cd.doc_type),
                    cd.title,
                    cd.content_markdown,
                    cd.version as i64,
                    bool_to_int(cd.is_active),
                    datetime_to_string(&cd.created_at),
                    datetime_to_string(&cd.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_context_document(
        &self,
        id: Id<ContextDocument>,
    ) -> Result<Option<ContextDocument>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, doc_type, title, content_markdown, version, is_active, created_at, updated_at FROM context_documents WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_context_document(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(cd)) => Ok(Some(cd?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_context_documents(&self) -> Result<Vec<ContextDocument>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, doc_type, title, content_markdown, version, is_active, created_at, updated_at FROM context_documents ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_context_document(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE context_documents SET doc_type = ?1, title = ?2, content_markdown = ?3, version = ?4, is_active = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    json_to_string(&cd.doc_type),
                    cd.title,
                    cd.content_markdown,
                    cd.version as i64,
                    bool_to_int(cd.is_active),
                    datetime_to_string(&cd.updated_at),
                    uuid_to_string(cd.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("context_document".into()));
        }
        Ok(())
    }

    fn delete_context_document(&self, id: Id<ContextDocument>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute(
                "DELETE FROM context_documents WHERE id = ?1",
                params![id_str],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("context_document".into()));
        }
        Ok(())
    }

    // ── Plan ──

    fn insert_plan(&self, p: &Plan) -> Result<(), Self::Error> {
        let items_json = json_to_string(&p.items);
        self.conn
            .execute(
                "INSERT INTO plans (id, daily_log_id, title, items_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    uuid_to_string(p.id),
                    uuid_to_string(p.daily_log_id),
                    p.title,
                    items_json,
                    datetime_to_string(&p.created_at),
                    datetime_to_string(&p.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_plan(&self, id: Id<Plan>) -> Result<Option<Plan>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, title, items_json, created_at, updated_at FROM plans WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_plan(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(p)) => Ok(Some(p?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn get_plan_by_daily_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Option<Plan>, Self::Error> {
        let log_id_str = uuid_to_string(daily_log_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, title, items_json, created_at, updated_at FROM plans WHERE daily_log_id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![log_id_str], |row| Ok(row_to_plan(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(p)) => Ok(Some(p?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn update_plan(&self, p: &Plan) -> Result<(), Self::Error> {
        let items_json = json_to_string(&p.items);
        let affected = self
            .conn
            .execute(
                "UPDATE plans SET daily_log_id = ?1, title = ?2, items_json = ?3, updated_at = ?4 WHERE id = ?5",
                params![
                    uuid_to_string(p.daily_log_id),
                    p.title,
                    items_json,
                    datetime_to_string(&p.updated_at),
                    uuid_to_string(p.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("plan".into()));
        }
        Ok(())
    }

    fn delete_plan(&self, id: Id<Plan>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM plans WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("plan".into()));
        }
        Ok(())
    }

    // ── ActionProposal ──

    fn insert_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error> {
        let payload_str = json_to_string(&ap.payload_json);
        self.conn
            .execute(
                "INSERT INTO action_proposals (id, source, proposal_type, payload_json, status, created_at, applied_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    uuid_to_string(ap.id),
                    json_to_string(&ap.source),
                    ap.proposal_type,
                    payload_str,
                    json_to_string(&ap.status),
                    datetime_to_string(&ap.created_at),
                    ap.applied_at.map(|dt| datetime_to_string(&dt)),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_action_proposal(
        &self,
        id: Id<ActionProposal>,
    ) -> Result<Option<ActionProposal>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, source, proposal_type, payload_json, status, created_at, applied_at FROM action_proposals WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_action_proposal(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        match rows.next() {
            Some(Ok(ap)) => Ok(Some(ap?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_action_proposals(&self) -> Result<Vec<ActionProposal>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, source, proposal_type, payload_json, status, created_at, applied_at FROM action_proposals ORDER BY created_at DESC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_action_proposal(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn list_action_proposals_by_status(
        &self,
        status: ProposalStatus,
    ) -> Result<Vec<ActionProposal>, Self::Error> {
        let status_str = json_to_string(&status);
        let mut stmt = self
            .conn
            .prepare("SELECT id, source, proposal_type, payload_json, status, created_at, applied_at FROM action_proposals WHERE status = ?1 ORDER BY created_at DESC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![status_str], |row| Ok(row_to_action_proposal(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error> {
        let payload_str = json_to_string(&ap.payload_json);
        let affected = self
            .conn
            .execute(
                "UPDATE action_proposals SET source = ?1, proposal_type = ?2, payload_json = ?3, status = ?4, applied_at = ?5 WHERE id = ?6",
                params![
                    json_to_string(&ap.source),
                    ap.proposal_type,
                    payload_str,
                    json_to_string(&ap.status),
                    ap.applied_at.map(|dt| datetime_to_string(&dt)),
                    uuid_to_string(ap.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("action_proposal".into()));
        }
        Ok(())
    }

    fn delete_action_proposal(&self, id: Id<ActionProposal>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute(
                "DELETE FROM action_proposals WHERE id = ?1",
                params![id_str],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("action_proposal".into()));
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────
// Row mapping functions (used by SqliteStore)
// ──────────────────────────────────────────────

fn row_to_daily_log(row: &rusqlite::Row<'_>) -> Result<DailyLog, CoreError> {
    Ok(DailyLog {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        date: NaiveDate::parse_from_str(&row_get::<String>(row, 1)?, "%Y-%m-%d")
            .map_err(|e| CoreError::InvalidInput(e.to_string()))?,
        mode: row_get(row, 2)?,
        sleep_score: row_get::<Option<i64>>(row, 3)?.map(|v| v as u8),
        energy: row_get::<Option<i64>>(row, 4)?.map(|v| v as u8),
        mood: row_get::<Option<i64>>(row, 5)?.map(|v| v as u8),
        raw_notes: row_get(row, 6)?,
        ai_summary: row_get(row, 7)?,
        created_at: deserialize_datetime(&row_get::<String>(row, 8)?)?,
        updated_at: deserialize_datetime(&row_get::<String>(row, 9)?)?,
    })
}

fn row_to_check_in(row: &rusqlite::Row<'_>) -> Result<CheckIn, CoreError> {
    Ok(CheckIn {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        daily_log_id: parse_uuid(row_get::<String>(row, 1)?.as_str())?,
        check_in_type: json_from_str(&row_get::<String>(row, 2)?)?,
        raw_input: row_get(row, 3)?,
        structured_data: option_json_from_str(row_get::<Option<String>>(row, 4)?)?,
        agent_response: row_get(row, 5)?,
        created_at: deserialize_datetime(&row_get::<String>(row, 6)?)?,
    })
}

fn row_to_habit(row: &rusqlite::Row<'_>) -> Result<Habit, CoreError> {
    Ok(Habit {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        name: row_get(row, 1)?,
        description: row_get(row, 2)?,
        frequency: row_get(row, 3)?,
        target: row_get(row, 4)?,
        is_active: int_to_bool(row_get::<i64>(row, 5)?),
        created_at: deserialize_datetime(&row_get::<String>(row, 6)?)?,
        updated_at: deserialize_datetime(&row_get::<String>(row, 7)?)?,
    })
}

fn row_to_habit_event(row: &rusqlite::Row<'_>) -> Result<HabitEvent, CoreError> {
    Ok(HabitEvent {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        habit_id: parse_uuid(row_get::<String>(row, 1)?.as_str())?,
        date_time: deserialize_datetime(&row_get::<String>(row, 2)?)?,
        status: json_from_str(&row_get::<String>(row, 3)?)?,
        level: json_from_str(&row_get::<String>(row, 4)?)?,
        comment: row_get(row, 5)?,
    })
}

fn row_to_reminder(row: &rusqlite::Row<'_>) -> Result<ReminderDefinition, CoreError> {
    Ok(ReminderDefinition {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        title: row_get(row, 1)?,
        message: row_get(row, 2)?,
        schedule_rule: row_get(row, 3)?,
        next_fire_at: match row_get::<Option<String>>(row, 4)? {
            Some(s) => Some(deserialize_datetime(&s)?),
            None => None,
        },
        enabled: int_to_bool(row_get::<i64>(row, 5)?),
        created_at: deserialize_datetime(&row_get::<String>(row, 6)?)?,
        updated_at: deserialize_datetime(&row_get::<String>(row, 7)?)?,
    })
}

fn row_to_alarm(row: &rusqlite::Row<'_>) -> Result<AlarmDefinition, CoreError> {
    Ok(AlarmDefinition {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        title: row_get(row, 1)?,
        time: NaiveTime::parse_from_str(&row_get::<String>(row, 2)?, "%H:%M:%S")
            .map_err(|e| CoreError::InvalidInput(e.to_string()))?,
        repeat_rule: row_get(row, 3)?,
        enabled: int_to_bool(row_get::<i64>(row, 4)?),
        platform_binding_id: row_get(row, 5)?,
        created_at: deserialize_datetime(&row_get::<String>(row, 6)?)?,
        updated_at: deserialize_datetime(&row_get::<String>(row, 7)?)?,
    })
}

fn row_to_timer(row: &rusqlite::Row<'_>) -> Result<TimerDefinition, CoreError> {
    Ok(TimerDefinition {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        title: row_get(row, 1)?,
        duration_seconds: row_get::<i64>(row, 2)? as u64,
        mode: json_from_str(&row_get::<String>(row, 3)?)?,
        created_at: deserialize_datetime(&row_get::<String>(row, 4)?)?,
    })
}

fn row_to_context_document(row: &rusqlite::Row<'_>) -> Result<ContextDocument, CoreError> {
    Ok(ContextDocument {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        doc_type: json_from_str(&row_get::<String>(row, 1)?)?,
        title: row_get(row, 2)?,
        content_markdown: row_get(row, 3)?,
        version: row_get::<i64>(row, 4)? as u32,
        is_active: int_to_bool(row_get::<i64>(row, 5)?),
        created_at: deserialize_datetime(&row_get::<String>(row, 6)?)?,
        updated_at: deserialize_datetime(&row_get::<String>(row, 7)?)?,
    })
}

fn row_to_plan(row: &rusqlite::Row<'_>) -> Result<Plan, CoreError> {
    Ok(Plan {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        daily_log_id: parse_uuid(row_get::<String>(row, 1)?.as_str())?,
        title: row_get(row, 2)?,
        items: json_from_str(&row_get::<String>(row, 3)?)?,
        created_at: deserialize_datetime(&row_get::<String>(row, 4)?)?,
        updated_at: deserialize_datetime(&row_get::<String>(row, 5)?)?,
    })
}

fn row_to_action_proposal(row: &rusqlite::Row<'_>) -> Result<ActionProposal, CoreError> {
    Ok(ActionProposal {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        source: json_from_str(&row_get::<String>(row, 1)?)?,
        proposal_type: row_get(row, 2)?,
        payload_json: option_json_from_str::<serde_json::Value>(Some(row_get::<String>(row, 3)?))?
            .unwrap_or(serde_json::Value::Null),
        status: json_from_str(&row_get::<String>(row, 4)?)?,
        created_at: deserialize_datetime(&row_get::<String>(row, 5)?)?,
        applied_at: match row_get::<Option<String>>(row, 6)? {
            Some(s) => Some(deserialize_datetime(&s)?),
            None => None,
        },
    })
}

// ──────────────────────────────────────────────
// NoopStore — stub implementations
// ──────────────────────────────────────────────

pub struct NoopStore;

impl Store for NoopStore {
    type Error = CoreError;

    fn migrate(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn health_check(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_daily_log(&self, _log: &DailyLog) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_daily_log(&self, _id: Id<DailyLog>) -> Result<Option<DailyLog>, Self::Error> {
        Ok(None)
    }
    fn get_daily_log_by_date(&self, _date: NaiveDate) -> Result<Option<DailyLog>, Self::Error> {
        Ok(None)
    }
    fn list_daily_logs(&self) -> Result<Vec<DailyLog>, Self::Error> {
        Ok(vec![])
    }
    fn update_daily_log(&self, _log: &DailyLog) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_daily_log(&self, _id: Id<DailyLog>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_check_in(&self, _ci: &CheckIn) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_check_in(&self, _id: Id<CheckIn>) -> Result<Option<CheckIn>, Self::Error> {
        Ok(None)
    }
    fn list_check_ins_by_log(
        &self,
        _daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<CheckIn>, Self::Error> {
        Ok(vec![])
    }
    fn update_check_in(&self, _ci: &CheckIn) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_check_in(&self, _id: Id<CheckIn>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_habit(&self, _h: &Habit) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_habit(&self, _id: Id<Habit>) -> Result<Option<Habit>, Self::Error> {
        Ok(None)
    }
    fn list_habits(&self) -> Result<Vec<Habit>, Self::Error> {
        Ok(vec![])
    }
    fn update_habit(&self, _h: &Habit) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_habit(&self, _id: Id<Habit>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_habit_event(&self, _he: &HabitEvent) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_habit_event(&self, _id: Id<HabitEvent>) -> Result<Option<HabitEvent>, Self::Error> {
        Ok(None)
    }
    fn list_habit_events_by_habit(
        &self,
        _habit_id: Id<Habit>,
    ) -> Result<Vec<HabitEvent>, Self::Error> {
        Ok(vec![])
    }
    fn update_habit_event(&self, _he: &HabitEvent) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_habit_event(&self, _id: Id<HabitEvent>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_reminder(&self, _r: &ReminderDefinition) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_reminder(
        &self,
        _id: Id<ReminderDefinition>,
    ) -> Result<Option<ReminderDefinition>, Self::Error> {
        Ok(None)
    }
    fn list_reminders(&self) -> Result<Vec<ReminderDefinition>, Self::Error> {
        Ok(vec![])
    }
    fn update_reminder(&self, _r: &ReminderDefinition) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_reminder(&self, _id: Id<ReminderDefinition>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_alarm(&self, _a: &AlarmDefinition) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_alarm(&self, _id: Id<AlarmDefinition>) -> Result<Option<AlarmDefinition>, Self::Error> {
        Ok(None)
    }
    fn list_alarms(&self) -> Result<Vec<AlarmDefinition>, Self::Error> {
        Ok(vec![])
    }
    fn update_alarm(&self, _a: &AlarmDefinition) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_alarm(&self, _id: Id<AlarmDefinition>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_timer(&self, _t: &TimerDefinition) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_timer(&self, _id: Id<TimerDefinition>) -> Result<Option<TimerDefinition>, Self::Error> {
        Ok(None)
    }
    fn list_timers(&self) -> Result<Vec<TimerDefinition>, Self::Error> {
        Ok(vec![])
    }
    fn update_timer(&self, _t: &TimerDefinition) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_timer(&self, _id: Id<TimerDefinition>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_context_document(&self, _cd: &ContextDocument) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_context_document(
        &self,
        _id: Id<ContextDocument>,
    ) -> Result<Option<ContextDocument>, Self::Error> {
        Ok(None)
    }
    fn list_context_documents(&self) -> Result<Vec<ContextDocument>, Self::Error> {
        Ok(vec![])
    }
    fn update_context_document(&self, _cd: &ContextDocument) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_context_document(&self, _id: Id<ContextDocument>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_plan(&self, _p: &Plan) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_plan(&self, _id: Id<Plan>) -> Result<Option<Plan>, Self::Error> {
        Ok(None)
    }
    fn get_plan_by_daily_log(
        &self,
        _daily_log_id: Id<DailyLog>,
    ) -> Result<Option<Plan>, Self::Error> {
        Ok(None)
    }
    fn update_plan(&self, _p: &Plan) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_plan(&self, _id: Id<Plan>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_action_proposal(&self, _ap: &ActionProposal) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_action_proposal(
        &self,
        _id: Id<ActionProposal>,
    ) -> Result<Option<ActionProposal>, Self::Error> {
        Ok(None)
    }
    fn list_action_proposals(&self) -> Result<Vec<ActionProposal>, Self::Error> {
        Ok(vec![])
    }
    fn list_action_proposals_by_status(
        &self,
        _status: ProposalStatus,
    ) -> Result<Vec<ActionProposal>, Self::Error> {
        Ok(vec![])
    }
    fn update_action_proposal(&self, _ap: &ActionProposal) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_action_proposal(&self, _id: Id<ActionProposal>) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use adiyutant_core::model::action_proposal::{ProposalSource, ProposalStatus};
    use adiyutant_core::model::check_in::CheckInType;
    use adiyutant_core::model::context_document::ContextDocumentType;
    use adiyutant_core::model::habit_event::{HabitEvent, HabitEventLevel, HabitEventStatus};
    use adiyutant_core::model::timer::TimerMode;
    use chrono::NaiveDate;
    use serde_json::json;

    fn setup() -> SqliteStore {
        let store = SqliteStore::new_in_memory().unwrap();
        store.migrate().unwrap();
        store
    }

    // ── Helper to create test entities ──

    fn make_daily_log(date_str: &str) -> DailyLog {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        DailyLog::new(date)
    }

    // ── 1. DailyLog: insert_and_get_daily_log ──

    #[test]
    fn insert_and_get_daily_log() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();
        let retrieved = store.get_daily_log(log.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, log.id);
    }

    // ── 2. DailyLog: get_daily_log_by_date ──

    #[test]
    fn get_daily_log_by_date() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();
        let retrieved = store
            .get_daily_log_by_date(NaiveDate::from_ymd_opt(2026, 6, 8).unwrap())
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, log.id);

        let not_found = store
            .get_daily_log_by_date(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
            .unwrap();
        assert!(not_found.is_none());
    }

    // ── 3. CheckIn: insert_and_get_check_in ──

    #[test]
    fn insert_and_get_check_in() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();

        let ci = CheckIn::new(log.id, CheckInType::Morning, "feeling great".into());
        store.insert_check_in(&ci).unwrap();
        let retrieved = store.get_check_in(ci.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, ci.id);
    }

    // ── 4. CheckIn: list_check_ins_by_log ──

    #[test]
    fn list_check_ins_by_log() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();

        let ci1 = CheckIn::new(log.id, CheckInType::Morning, "morning".into());
        let ci2 = CheckIn::new(log.id, CheckInType::Evening, "evening".into());
        store.insert_check_in(&ci1).unwrap();
        store.insert_check_in(&ci2).unwrap();

        let list = store.list_check_ins_by_log(log.id).unwrap();
        assert_eq!(list.len(), 2);

        let empty_log = make_daily_log("2026-06-09");
        store.insert_daily_log(&empty_log).unwrap();
        let empty_list = store.list_check_ins_by_log(empty_log.id).unwrap();
        assert!(empty_list.is_empty());
    }

    // ── 5. Habit: insert_and_get_habit ──

    #[test]
    fn insert_and_get_habit() {
        let store = setup();
        let h = Habit::new("read".into());
        store.insert_habit(&h).unwrap();
        let retrieved = store.get_habit(h.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, h.id);
    }

    // ── 6. HabitEvent: insert_and_get_habit_event ──

    #[test]
    fn insert_and_get_habit_event() {
        let store = setup();
        let h = Habit::new("exercise".into());
        store.insert_habit(&h).unwrap();

        let he = HabitEvent::new(h.id, HabitEventStatus::Done, HabitEventLevel::Base);
        store.insert_habit_event(&he).unwrap();
        let retrieved = store.get_habit_event(he.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, he.id);
    }

    // ── 7. Reminder: insert_and_get_reminder ──

    #[test]
    fn insert_and_get_reminder() {
        let store = setup();
        let r = ReminderDefinition::new("Drink water".into(), "every 2h".into());
        store.insert_reminder(&r).unwrap();
        let retrieved = store.get_reminder(r.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, r.id);
    }

    // ── 8. Alarm: insert_and_get_alarm ──

    #[test]
    fn insert_and_get_alarm() {
        let store = setup();
        let time = chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap();
        let a = AlarmDefinition::new("Wake up".into(), time);
        store.insert_alarm(&a).unwrap();
        let retrieved = store.get_alarm(a.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, a.id);
    }

    // ── 9. Timer: insert_and_get_timer ──

    #[test]
    fn insert_and_get_timer() {
        let store = setup();
        let t = TimerDefinition::new("Focus".into(), 1500, TimerMode::Focus);
        store.insert_timer(&t).unwrap();
        let retrieved = store.get_timer(t.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, t.id);
    }

    // ── 10. ContextDocument: insert_and_get_context_document ──

    #[test]
    fn insert_and_get_context_document() {
        let store = setup();
        let cd = ContextDocument::new(
            ContextDocumentType::Core,
            "My Core".into(),
            "# Values".into(),
        );
        store.insert_context_document(&cd).unwrap();
        let retrieved = store.get_context_document(cd.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, cd.id);
    }

    // ── 11. Plan with JSON items ──

    #[test]
    fn insert_and_get_plan_with_items() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();

        let mut plan = Plan::new(log.id, "My Day".into());
        plan.add_item("Write code".into());
        plan.add_item("Review PR".into());
        store.insert_plan(&plan).unwrap();

        let retrieved = store.get_plan(plan.id).unwrap();
        assert!(retrieved.is_some());
        let r = retrieved.unwrap();
        assert_eq!(r.items.len(), 2);
        assert_eq!(r.items[0].description, "Write code");
        assert_eq!(r.items[1].description, "Review PR");
    }

    // ── 12. ActionProposal: insert_and_get_action_proposal ──

    #[test]
    fn insert_and_get_action_proposal() {
        let store = setup();
        let ap = ActionProposal::new(
            ProposalSource::Agent,
            "add_habit".into(),
            json!({"name": "speech"}),
        );
        store.insert_action_proposal(&ap).unwrap();
        let retrieved = store.get_action_proposal(ap.id).unwrap();
        assert!(retrieved.is_some());
        let r = retrieved.unwrap();
        assert_eq!(r.id, ap.id);
        assert_eq!(r.payload_json, json!({"name": "speech"}));
    }

    // ── 13. DailyLog: update ──

    #[test]
    fn update_daily_log() {
        let store = setup();
        let mut log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();

        log.mode = Some("recovery".into());
        store.update_daily_log(&log).unwrap();

        let retrieved = store.get_daily_log(log.id).unwrap().unwrap();
        assert_eq!(retrieved.mode, Some("recovery".into()));
    }

    // ── 14. DailyLog: delete ──

    #[test]
    fn delete_daily_log() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();
        store.delete_daily_log(log.id).unwrap();
        let retrieved = store.get_daily_log(log.id).unwrap();
        assert!(retrieved.is_none());
    }

    // ── 15. Nonexistent returns None ──

    #[test]
    fn get_nonexistent_returns_none() {
        let store = setup();
        let fake_id = Id::<DailyLog>::new();
        let result = store.get_daily_log(fake_id).unwrap();
        assert!(result.is_none());
    }

    // ── 16. NoopStore methods don't panic ──

    #[test]
    fn noop_store_methods_dont_panic() {
        let store = NoopStore;

        // Bootstrap
        assert!(store.migrate().is_ok());
        assert!(store.health_check().is_ok());

        // DailyLog
        let log = make_daily_log("2026-06-08");
        assert!(store.insert_daily_log(&log).is_ok());
        assert!(store.get_daily_log(log.id).unwrap().is_none());
        assert!(
            store
                .get_daily_log_by_date(NaiveDate::from_ymd_opt(2026, 6, 8).unwrap())
                .unwrap()
                .is_none()
        );
        assert!(store.list_daily_logs().unwrap().is_empty());
        assert!(store.update_daily_log(&log).is_ok());
        assert!(store.delete_daily_log(log.id).is_ok());

        // CheckIn
        let ci = CheckIn::new(log.id, CheckInType::Morning, "test".into());
        assert!(store.insert_check_in(&ci).is_ok());
        assert!(store.get_check_in(ci.id).unwrap().is_none());
        assert!(store.list_check_ins_by_log(log.id).unwrap().is_empty());
        assert!(store.update_check_in(&ci).is_ok());
        assert!(store.delete_check_in(ci.id).is_ok());

        // Habit
        let h = Habit::new("test".into());
        assert!(store.insert_habit(&h).is_ok());
        assert!(store.get_habit(h.id).unwrap().is_none());
        assert!(store.list_habits().unwrap().is_empty());
        assert!(store.update_habit(&h).is_ok());
        assert!(store.delete_habit(h.id).is_ok());

        // HabitEvent
        let he = HabitEvent::new(h.id, HabitEventStatus::Done, HabitEventLevel::Base);
        assert!(store.insert_habit_event(&he).is_ok());
        assert!(store.get_habit_event(he.id).unwrap().is_none());
        assert!(store.list_habit_events_by_habit(h.id).unwrap().is_empty());
        assert!(store.update_habit_event(&he).is_ok());
        assert!(store.delete_habit_event(he.id).is_ok());

        // Reminder
        let r = ReminderDefinition::new("test".into(), "daily".into());
        assert!(store.insert_reminder(&r).is_ok());
        assert!(store.get_reminder(r.id).unwrap().is_none());
        assert!(store.list_reminders().unwrap().is_empty());
        assert!(store.update_reminder(&r).is_ok());
        assert!(store.delete_reminder(r.id).is_ok());

        // Alarm
        let time = chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap();
        let a = AlarmDefinition::new("test".into(), time);
        assert!(store.insert_alarm(&a).is_ok());
        assert!(store.get_alarm(a.id).unwrap().is_none());
        assert!(store.list_alarms().unwrap().is_empty());
        assert!(store.update_alarm(&a).is_ok());
        assert!(store.delete_alarm(a.id).is_ok());

        // Timer
        let t = TimerDefinition::new("test".into(), 1500, TimerMode::Focus);
        assert!(store.insert_timer(&t).is_ok());
        assert!(store.get_timer(t.id).unwrap().is_none());
        assert!(store.list_timers().unwrap().is_empty());
        assert!(store.update_timer(&t).is_ok());
        assert!(store.delete_timer(t.id).is_ok());

        // ContextDocument
        let cd = ContextDocument::new(ContextDocumentType::Core, "test".into(), "content".into());
        assert!(store.insert_context_document(&cd).is_ok());
        assert!(store.get_context_document(cd.id).unwrap().is_none());
        assert!(store.list_context_documents().unwrap().is_empty());
        assert!(store.update_context_document(&cd).is_ok());
        assert!(store.delete_context_document(cd.id).is_ok());

        // Plan
        let mut plan = Plan::new(log.id, "test".into());
        plan.add_item("item".into());
        assert!(store.insert_plan(&plan).is_ok());
        assert!(store.get_plan(plan.id).unwrap().is_none());
        assert!(store.get_plan_by_daily_log(log.id).unwrap().is_none());
        assert!(store.update_plan(&plan).is_ok());
        assert!(store.delete_plan(plan.id).is_ok());

        // ActionProposal
        let ap = ActionProposal::new(ProposalSource::Agent, "test".into(), json!({}));
        assert!(store.insert_action_proposal(&ap).is_ok());
        assert!(store.get_action_proposal(ap.id).unwrap().is_none());
        assert!(store.list_action_proposals().unwrap().is_empty());
        assert!(
            store
                .list_action_proposals_by_status(ProposalStatus::Pending)
                .unwrap()
                .is_empty()
        );
        assert!(store.update_action_proposal(&ap).is_ok());
        assert!(store.delete_action_proposal(ap.id).is_ok());
    }

    // ── Extra: list_daily_logs returns multiple ──

    #[test]
    fn list_daily_logs_returns_all() {
        let store = setup();
        let log1 = make_daily_log("2026-06-08");
        let log2 = make_daily_log("2026-06-09");
        store.insert_daily_log(&log1).unwrap();
        store.insert_daily_log(&log2).unwrap();
        let list = store.list_daily_logs().unwrap();
        assert_eq!(list.len(), 2);
    }

    // ── Extra: list habits returns all ──

    #[test]
    fn list_habits_returns_all() {
        let store = setup();
        let h1 = Habit::new("read".into());
        let h2 = Habit::new("exercise".into());
        store.insert_habit(&h1).unwrap();
        store.insert_habit(&h2).unwrap();
        assert_eq!(store.list_habits().unwrap().len(), 2);
    }

    // ── Extra: migrate is idempotent ──

    #[test]
    fn migrate_is_idempotent() {
        let store = setup();
        // setup() already called migrate once. Calling again should succeed.
        store.migrate().unwrap();
        // Both the table count and operations should still work
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();
        assert!(store.get_daily_log(log.id).unwrap().is_some());
    }

    // ── Extra: health_check ──

    #[test]
    fn health_check_returns_ok() {
        let store = setup();
        assert!(store.health_check().is_ok());
    }

    // ── Extra: action_proposal list by status ──

    #[test]
    fn list_action_proposals_by_status() {
        let store = setup();
        let ap1 = ActionProposal::new(ProposalSource::Agent, "add_habit".into(), json!({"a": 1}));
        let ap2 = ActionProposal::new(ProposalSource::User, "remind".into(), json!({"b": 2}));
        store.insert_action_proposal(&ap1).unwrap();
        store.insert_action_proposal(&ap2).unwrap();
        assert_eq!(
            store
                .list_action_proposals_by_status(ProposalStatus::Pending)
                .unwrap()
                .len(),
            2
        );
        assert!(
            store
                .list_action_proposals_by_status(ProposalStatus::Applied)
                .unwrap()
                .is_empty()
        );
    }

    // ── Extra: check_in with structured_data ──

    #[test]
    fn check_in_with_structured_data() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();
        let mut ci = CheckIn::new(log.id, CheckInType::Evening, "tired".into());
        ci.structured_data = Some(json!({"sleep_hours": 6}));
        store.insert_check_in(&ci).unwrap();
        let retrieved = store.get_check_in(ci.id).unwrap().unwrap();
        assert_eq!(retrieved.structured_data, Some(json!({"sleep_hours": 6})));
    }

    // ── Extra: plan with daily log query ──

    #[test]
    fn get_plan_by_daily_log() {
        let store = setup();
        let log = make_daily_log("2026-06-08");
        store.insert_daily_log(&log).unwrap();
        let plan = Plan::new(log.id, "My Plan".into());
        store.insert_plan(&plan).unwrap();
        let retrieved = store.get_plan_by_daily_log(log.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, plan.id);
    }
}
