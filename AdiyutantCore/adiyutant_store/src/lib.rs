use adiyutant_core::bundle::bundle_models::{
    AdiyutantBundle, BundleChecklistItem, BundleChecklistTemplate, ImportMode,
};
use adiyutant_core::error::CoreError;
use adiyutant_core::id::Id;
use adiyutant_core::model::action_proposal::ProposalStatus;
use adiyutant_core::model::context_document::ContextDocumentType;
use adiyutant_core::model::*;
use adiyutant_core::store::{Store, TakeRoadmapItemInput, TakeRoadmapItemResult};
use chrono::NaiveDate;
use chrono::NaiveTime;
use rusqlite::Connection;
use rusqlite::params;

// ──────────────────────────────────────────────
// Schema bootstrap
// ──────────────────────────────────────────────

fn run_migration(conn: &Connection) -> Result<(), CoreError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| CoreError::Storage(e.to_string()))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_versions (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_versions",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if current < 1 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS daily_logs (
                id TEXT PRIMARY KEY, date TEXT NOT NULL, mode TEXT,
                sleep_score INTEGER, energy INTEGER, mood INTEGER,
                raw_notes TEXT, ai_summary TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS check_ins (
                id TEXT PRIMARY KEY, daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
                check_in_type TEXT NOT NULL, raw_input TEXT NOT NULL, structured_data TEXT,
                agent_response TEXT, created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS habits (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT, frequency TEXT,
                target TEXT, is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS habit_events (
                id TEXT PRIMARY KEY, habit_id TEXT NOT NULL REFERENCES habits(id),
                date_time TEXT NOT NULL, status TEXT NOT NULL, level TEXT NOT NULL, comment TEXT
            );
            CREATE TABLE IF NOT EXISTS reminders (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, message TEXT,
                schedule_rule TEXT NOT NULL, next_fire_at TEXT, enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS alarms (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, time TEXT NOT NULL, repeat_rule TEXT,
                enabled INTEGER NOT NULL DEFAULT 1, platform_binding_id TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS timers (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, duration_seconds INTEGER NOT NULL,
                mode TEXT NOT NULL, created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS context_documents (
                id TEXT PRIMARY KEY, doc_type TEXT NOT NULL, title TEXT NOT NULL,
                content_markdown TEXT NOT NULL, version INTEGER NOT NULL DEFAULT 1,
                is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS plans (
                id TEXT PRIMARY KEY, daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
                title TEXT NOT NULL, items_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS action_proposals (
                id TEXT PRIMARY KEY, source TEXT NOT NULL, proposal_type TEXT NOT NULL,
                payload_json TEXT NOT NULL, status TEXT NOT NULL,
                created_at TEXT NOT NULL, applied_at TEXT
            );
        ").map_err(|e| CoreError::Storage(e.to_string()))?;
        mark_migration(conn, 1, "v0.1-initial")?;
    }

    if current < 2 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS plan_items (
                id TEXT PRIMARY KEY, daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
                title TEXT NOT NULL, description TEXT, quadrant TEXT NOT NULL,
                planned_start TEXT, planned_end TEXT, status TEXT NOT NULL DEFAULT 'Planned',
                priority INTEGER NOT NULL DEFAULT 5, source TEXT NOT NULL DEFAULT 'Manual',
                waiting_review_at TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS task_checkpoints (
                id TEXT PRIMARY KEY, plan_item_id TEXT NOT NULL REFERENCES plan_items(id),
                kind TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'Pending', response TEXT,
                created_at TEXT NOT NULL, answered_at TEXT
            );
            CREATE TABLE IF NOT EXISTS checklist_templates (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, category TEXT NOT NULL,
                items_json TEXT NOT NULL DEFAULT '[]', version INTEGER NOT NULL DEFAULT 1,
                is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS checklist_runs (
                id TEXT PRIMARY KEY, template_id TEXT NOT NULL REFERENCES checklist_templates(id),
                daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
                started_at TEXT NOT NULL, completed_at TEXT, answers_json TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE IF NOT EXISTS journal_entries (
                id TEXT PRIMARY KEY, daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
                timestamp TEXT NOT NULL, entry_type TEXT NOT NULL, summary TEXT NOT NULL
            );
        ").map_err(|e| CoreError::Storage(e.to_string()))?;
        mark_migration(conn, 2, "v0.2-planning-checklists-journal")?;
    }

    if current < 3 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS import_runs (
                id TEXT PRIMARY KEY,
                bundle_id TEXT NOT NULL,
                bundle_title TEXT NOT NULL,
                schema_version TEXT NOT NULL,
                mode TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT NOT NULL,
                summary_json TEXT NOT NULL
            );

            ALTER TABLE checklist_templates ADD COLUMN slug TEXT;

            ALTER TABLE context_documents ADD COLUMN source_slug TEXT;
            ALTER TABLE context_documents ADD COLUMN source_metadata_json TEXT;

            CREATE UNIQUE INDEX IF NOT EXISTS idx_checklist_templates_slug ON checklist_templates(slug) WHERE slug IS NOT NULL;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_context_documents_source_slug ON context_documents(source_slug) WHERE source_slug IS NOT NULL;
        ").map_err(|e| CoreError::Storage(e.to_string()))?;
        mark_migration(conn, 3, "v0.3-bundle-import-history")?;
    }

    if current < 4 {
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                slug TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                priority INTEGER NOT NULL DEFAULT 5,
                why TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS roadmaps (
                id TEXT PRIMARY KEY,
                slug TEXT NOT NULL,
                project_id TEXT NOT NULL REFERENCES projects(id),
                title TEXT NOT NULL,
                description TEXT,
                horizon TEXT NOT NULL DEFAULT 'month',
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (project_id, slug)
            );

            CREATE TABLE IF NOT EXISTS roadmap_phases (
                id TEXT PRIMARY KEY,
                slug TEXT NOT NULL,
                roadmap_id TEXT NOT NULL REFERENCES roadmaps(id),
                title TEXT NOT NULL,
                order_index INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'planned',
                UNIQUE (roadmap_id, slug)
            );

            CREATE TABLE IF NOT EXISTS roadmap_items (
                id TEXT PRIMARY KEY,
                slug TEXT NOT NULL,
                phase_id TEXT NOT NULL REFERENCES roadmap_phases(id),
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'planned',
                priority INTEGER NOT NULL DEFAULT 5,
                acceptance_criteria_json TEXT NOT NULL DEFAULT '[]',
                depends_on_json TEXT NOT NULL DEFAULT '[]',
                links_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (phase_id, slug)
            );

            CREATE TABLE IF NOT EXISTS roadmap_plan_links (
                id TEXT PRIMARY KEY,
                roadmap_item_id TEXT NOT NULL REFERENCES roadmap_items(id),
                plan_item_id TEXT NOT NULL REFERENCES plan_items(id),
                link_type TEXT NOT NULL DEFAULT 'taken_into_day',
                created_at TEXT NOT NULL
            );
        ",
        )
        .map_err(|e| CoreError::Storage(e.to_string()))?;
        mark_migration(conn, 4, "v0.4-projects-roadmaps")?;
    }

    Ok(())
}

fn mark_migration(conn: &Connection, version: i64, name: &str) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO schema_versions (version, name, applied_at) VALUES (?1, ?2, ?3)",
        params![version, name, chrono::Utc::now().to_rfc3339()],
    )
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
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

fn parse_datetime(s: &str) -> Result<adiyutant_core::datetime::AdiyutantDateTime, CoreError> {
    deserialize_datetime(s)
}

fn row_get_datetime(
    row: &rusqlite::Row<'_>,
    idx: usize,
) -> Result<adiyutant_core::datetime::AdiyutantDateTime, CoreError> {
    let s: String = row_get(row, idx)?;
    parse_datetime(&s)
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

    pub fn with_transaction<F, T>(&self, f: F) -> Result<T, CoreError>
    where
        F: FnOnce(&Connection) -> Result<T, CoreError>,
    {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match f(&self.conn) {
            Ok(val) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(val)
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn apply_bundle_inner(
        &self,
        bundle: &AdiyutantBundle,
        mode: ImportMode,
        import_run: &ImportRun,
    ) -> Result<(), CoreError> {
        self.insert_import_run(import_run)?;

        // ── life_core ──
        if let Some(lc) = &bundle.life_core {
            if lc.profile.is_some() {
                let exists = self
                    .get_context_document_by_source_slug("profile")
                    .ok()
                    .flatten();
                if let Some(mut existing) = exists {
                    existing.updated_at = adiyutant_core::datetime::AdiyutantDateTime::now();
                    self.upsert_context_document(&existing)?;
                } else {
                    let doc = adiyutant_core::model::context_document::ContextDocument::with_source(
                        adiyutant_core::model::context_document::ContextDocumentType::LifeCore,
                        "Profile".into(),
                        serde_json::to_string(&lc.profile).unwrap_or_default(),
                        Some("profile".into()),
                        None,
                    );
                    self.upsert_context_document(&doc)?;
                }
            }
            for doc in &lc.context_documents {
                apply_context_doc_internal(
                    &doc.slug,
                    &doc.title,
                    &doc.doc_type,
                    &doc.content,
                    self,
                    mode,
                )?;
            }
        }

        // ── routines ──
        if let Some(rs) = &bundle.routines {
            for entry in &rs.routines {
                let slug = entry.slug.as_deref().unwrap_or(&entry.title);
                let content = format!(
                    "# {}\n\n{}\n\n## Steps\n{}",
                    entry.title,
                    entry.description.as_deref().unwrap_or(""),
                    entry
                        .steps
                        .iter()
                        .map(|s| format!("- {s}"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
                apply_context_doc_internal(
                    &Some(slug.to_string()),
                    &entry.title,
                    "routine",
                    &content,
                    self,
                    mode,
                )?;
            }
        }

        // ── rules ──
        if let Some(rs) = &bundle.rules {
            for entry in &rs.rules {
                let slug = entry.slug.as_deref().unwrap_or(&entry.title);
                let category = entry.category.as_deref().unwrap_or("rules");
                apply_context_doc_internal(
                    &Some(slug.to_string()),
                    &entry.title,
                    category,
                    &entry.description,
                    self,
                    mode,
                )?;
            }
        }

        // ── checklists ──
        if let Some(cs) = &bundle.checklists {
            for ct in &cs.templates {
                apply_checklist_template_internal(ct, self, mode)?;
            }
        }

        // ── planning ──
        if bundle.planning.is_some() {
            let exists = self
                .get_context_document_by_source_slug("planning")
                .ok()
                .flatten();
            if let Some(mut existing) = exists {
                existing.updated_at = adiyutant_core::datetime::AdiyutantDateTime::now();
                self.upsert_context_document(&existing)?;
            } else {
                let doc = adiyutant_core::model::context_document::ContextDocument::with_source(
                    adiyutant_core::model::context_document::ContextDocumentType::PlanningPreferences,
                    "Planning Preferences".into(),
                    "# Planning Preferences\n\nDefault planning configuration.".into(),
                    Some("planning".into()),
                    None,
                );
                self.upsert_context_document(&doc)?;
            }
        }

        // ── context ──
        for doc_entry in &bundle.context_docs {
            apply_context_doc_internal(
                &doc_entry.slug,
                &doc_entry.title,
                "custom",
                &doc_entry.content,
                self,
                mode,
            )?;
        }

        // ── projects ──
        for bp in &bundle.projects {
            let exists = self.get_project_by_slug(&bp.slug).ok().flatten();
            match mode {
                ImportMode::Append => {
                    if exists.is_none() {
                        let mut project = adiyutant_core::model::project::Project::new(
                            bp.slug.clone(),
                            bp.title.clone(),
                        );
                        if let Some(ref desc) = bp.description {
                            project.description = Some(desc.clone());
                        }
                        project.status =
                            adiyutant_core::model::project::ProjectStatus::from_str(&bp.status)
                                .unwrap_or(adiyutant_core::model::project::ProjectStatus::Active);
                        project.priority = bp.priority;
                        project.why = bp.why.clone();
                        self.insert_project(&project)?;
                    }
                }
                ImportMode::Merge => {
                    if let Some(mut existing) = exists {
                        existing.title = bp.title.clone();
                        existing.description.clone_from(&bp.description);
                        existing.status =
                            adiyutant_core::model::project::ProjectStatus::from_str(&bp.status)
                                .unwrap_or(adiyutant_core::model::project::ProjectStatus::Active);
                        existing.priority = bp.priority;
                        existing.why = bp.why.clone();
                        existing.updated_at = adiyutant_core::datetime::AdiyutantDateTime::now();
                        self.update_project(&existing)?;
                    } else {
                        let mut project = adiyutant_core::model::project::Project::new(
                            bp.slug.clone(),
                            bp.title.clone(),
                        );
                        if let Some(ref desc) = bp.description {
                            project.description = Some(desc.clone());
                        }
                        project.status =
                            adiyutant_core::model::project::ProjectStatus::from_str(&bp.status)
                                .unwrap_or(adiyutant_core::model::project::ProjectStatus::Active);
                        project.priority = bp.priority;
                        project.why = bp.why.clone();
                        self.insert_project(&project)?;
                    }
                }
            }
        }

        // ── roadmaps ──
        for br in &bundle.roadmaps {
            let project = self.get_project_by_slug(&br.project_slug).ok().flatten();
            let project_id = match project {
                Some(ref proj) => proj.id,
                None => continue,
            };

            let existing_roadmap = self
                .get_roadmap_by_project_and_slug(project_id, &br.slug)
                .ok()
                .flatten();

            let roadmap_id = match mode {
                ImportMode::Append => {
                    if let Some(existing) = existing_roadmap {
                        existing.id
                    } else {
                        let mut roadmap = adiyutant_core::model::roadmap::Roadmap::new(
                            br.slug.clone(),
                            project_id,
                            br.title.clone(),
                        );
                        if let Some(ref desc) = br.description {
                            roadmap.description = Some(desc.clone());
                        }
                        roadmap.horizon =
                            adiyutant_core::model::roadmap::RoadmapHorizon::from_str(&br.horizon)
                                .unwrap_or(adiyutant_core::model::roadmap::RoadmapHorizon::Month);
                        roadmap.status =
                            adiyutant_core::model::project::ProjectStatus::from_str(&br.status)
                                .unwrap_or(adiyutant_core::model::project::ProjectStatus::Active);
                        self.insert_roadmap(&roadmap)?;
                        roadmap.id
                    }
                }
                ImportMode::Merge => {
                    if let Some(mut existing) = existing_roadmap {
                        existing.title = br.title.clone();
                        existing.description.clone_from(&br.description);
                        existing.horizon =
                            adiyutant_core::model::roadmap::RoadmapHorizon::from_str(&br.horizon)
                                .unwrap_or(adiyutant_core::model::roadmap::RoadmapHorizon::Month);
                        existing.status =
                            adiyutant_core::model::project::ProjectStatus::from_str(&br.status)
                                .unwrap_or(adiyutant_core::model::project::ProjectStatus::Active);
                        existing.updated_at = adiyutant_core::datetime::AdiyutantDateTime::now();
                        self.update_roadmap(&existing)?;
                        existing.id
                    } else {
                        let mut roadmap = adiyutant_core::model::roadmap::Roadmap::new(
                            br.slug.clone(),
                            project_id,
                            br.title.clone(),
                        );
                        if let Some(ref desc) = br.description {
                            roadmap.description = Some(desc.clone());
                        }
                        roadmap.horizon =
                            adiyutant_core::model::roadmap::RoadmapHorizon::from_str(&br.horizon)
                                .unwrap_or(adiyutant_core::model::roadmap::RoadmapHorizon::Month);
                        roadmap.status =
                            adiyutant_core::model::project::ProjectStatus::from_str(&br.status)
                                .unwrap_or(adiyutant_core::model::project::ProjectStatus::Active);
                        self.insert_roadmap(&roadmap)?;
                        roadmap.id
                    }
                }
            };

            // ── roadmap phases ──
            let existing_phases = self.list_roadmap_phases(roadmap_id).unwrap_or_default();
            for phase in &br.phases {
                let existing_phase = existing_phases.iter().find(|p| p.slug == phase.slug);
                let phase_id = match mode {
                    ImportMode::Append => {
                        if let Some(ep) = existing_phase {
                            ep.id
                        } else {
                            let new_phase = adiyutant_core::model::roadmap::RoadmapPhase::new(
                                phase.slug.clone(),
                                roadmap_id,
                                phase.title.clone(),
                                phase.order_index,
                            );
                            self.insert_roadmap_phase(&new_phase)?;
                            new_phase.id
                        }
                    }
                    ImportMode::Merge => {
                        if let Some(ep) = existing_phase {
                            self.update_roadmap_phase(
                                &adiyutant_core::model::roadmap::RoadmapPhase {
                                    id: ep.id,
                                    slug: phase.slug.clone(),
                                    roadmap_id,
                                    title: phase.title.clone(),
                                    order_index: phase.order_index,
                                    status: ep.status,
                                },
                            )?;
                            ep.id
                        } else {
                            let new_phase = adiyutant_core::model::roadmap::RoadmapPhase::new(
                                phase.slug.clone(),
                                roadmap_id,
                                phase.title.clone(),
                                phase.order_index,
                            );
                            self.insert_roadmap_phase(&new_phase)?;
                            new_phase.id
                        }
                    }
                };

                // ── roadmap items ──
                let existing_items = self.list_roadmap_items(phase_id).unwrap_or_default();
                for item in &phase.items {
                    let existing_item = existing_items.iter().find(|i| i.slug == item.slug);
                    match mode {
                        ImportMode::Append => {
                            if existing_item.is_none() {
                                let mut new_item = adiyutant_core::model::roadmap::RoadmapItem::new(
                                    item.slug.clone(),
                                    phase_id,
                                    item.title.clone(),
                                );
                                new_item.description = item.description.clone();
                                new_item.priority = item.priority;
                                new_item.acceptance_criteria = item.acceptance_criteria.clone();
                                new_item.depends_on = item.depends_on.clone();
                                new_item.links = item.links.clone();
                                self.insert_roadmap_item(&new_item)?;
                            }
                        }
                        ImportMode::Merge => {
                            if let Some(ei) = existing_item {
                                let now = adiyutant_core::datetime::AdiyutantDateTime::now();
                                self.update_roadmap_item(
                                    &adiyutant_core::model::roadmap::RoadmapItem {
                                        id: ei.id,
                                        slug: item.slug.clone(),
                                        phase_id,
                                        title: item.title.clone(),
                                        description: item.description.clone(),
                                        status: adiyutant_core::model::roadmap::RoadmapItemStatus::from_str(&item.status)
                                            .unwrap_or(ei.status),
                                        priority: item.priority,
                                        acceptance_criteria: item.acceptance_criteria.clone(),
                                        depends_on: item.depends_on.clone(),
                                        links: item.links.clone(),
                                        created_at: ei.created_at,
                                        updated_at: now,
                                    },
                                )?;
                            } else {
                                let mut new_item = adiyutant_core::model::roadmap::RoadmapItem::new(
                                    item.slug.clone(),
                                    phase_id,
                                    item.title.clone(),
                                );
                                new_item.description = item.description.clone();
                                new_item.status =
                                    adiyutant_core::model::roadmap::RoadmapItemStatus::from_str(
                                        &item.status,
                                    )
                                    .unwrap_or(
                                        adiyutant_core::model::roadmap::RoadmapItemStatus::Planned,
                                    );
                                new_item.priority = item.priority;
                                new_item.acceptance_criteria = item.acceptance_criteria.clone();
                                new_item.depends_on = item.depends_on.clone();
                                new_item.links = item.links.clone();
                                self.insert_roadmap_item(&new_item)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
                "INSERT INTO context_documents (id, doc_type, title, content_markdown, version, is_active, created_at, updated_at, source_slug, source_metadata_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    uuid_to_string(cd.id),
                    json_to_string(&cd.doc_type),
                    cd.title,
                    cd.content_markdown,
                    cd.version as i64,
                    bool_to_int(cd.is_active),
                    datetime_to_string(&cd.created_at),
                    datetime_to_string(&cd.updated_at),
                    cd.source_slug,
                    cd.source_metadata_json,
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
            .prepare("SELECT id, doc_type, title, content_markdown, version, is_active, created_at, updated_at, source_slug, source_metadata_json FROM context_documents WHERE id = ?1")
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
            .prepare("SELECT id, doc_type, title, content_markdown, version, is_active, created_at, updated_at, source_slug, source_metadata_json FROM context_documents ORDER BY created_at ASC")
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
                "UPDATE context_documents SET doc_type = ?1, title = ?2, content_markdown = ?3, version = ?4, is_active = ?5, source_slug = ?6, source_metadata_json = ?7, updated_at = ?8 WHERE id = ?9",
                params![
                    json_to_string(&cd.doc_type),
                    cd.title,
                    cd.content_markdown,
                    cd.version as i64,
                    bool_to_int(cd.is_active),
                    cd.source_slug,
                    cd.source_metadata_json,
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

    // ── PlanItem ──

    fn insert_plan_item(&self, item: &PlanItem) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO plan_items (id, daily_log_id, title, description, quadrant, planned_start, planned_end, status, priority, source, waiting_review_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                uuid_to_string(item.id),
                uuid_to_string(item.daily_log_id),
                item.title,
                item.description,
                item.quadrant.as_str(),
                item.planned_start.map(|t| t.format("%H:%M").to_string()),
                item.planned_end.map(|t| t.format("%H:%M").to_string()),
                item.status.as_str(),
                item.priority,
                item.source.as_str(),
                item.waiting_review_at.map(|d| d.format("%Y-%m-%d").to_string()),
                datetime_to_string(&item.created_at),
                datetime_to_string(&item.updated_at),
            ],
        )
        .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_plan_item(&self, id: Id<PlanItem>) -> Result<Option<PlanItem>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, title, description, quadrant, planned_start, planned_end, status, priority, source, waiting_review_at, created_at, updated_at FROM plan_items WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_plan_item(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(item)) => Ok(Some(item?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_plan_items_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        let log_id_str = uuid_to_string(daily_log_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, title, description, quadrant, planned_start, planned_end, status, priority, source, waiting_review_at, created_at, updated_at FROM plan_items WHERE daily_log_id = ?1 ORDER BY priority ASC, created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![log_id_str], |row| Ok(row_to_plan_item(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(items)
    }

    fn update_plan_item(&self, item: &PlanItem) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE plan_items SET title = ?1, description = ?2, quadrant = ?3, planned_start = ?4, planned_end = ?5, status = ?6, priority = ?7, source = ?8, waiting_review_at = ?9, updated_at = ?10 WHERE id = ?11",
                params![
                    item.title,
                    item.description,
                    item.quadrant.as_str(),
                    item.planned_start.map(|t| t.format("%H:%M").to_string()),
                    item.planned_end.map(|t| t.format("%H:%M").to_string()),
                    item.status.as_str(),
                    item.priority,
                    item.source.as_str(),
                    item.waiting_review_at.map(|d| d.format("%Y-%m-%d").to_string()),
                    datetime_to_string(&item.updated_at),
                    uuid_to_string(item.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("plan_item".into()));
        }
        Ok(())
    }

    fn delete_plan_item(&self, id: Id<PlanItem>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM plan_items WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("plan_item".into()));
        }
        Ok(())
    }

    fn list_plan_items_by_status(
        &self,
        status: PlanItemStatus,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, title, description, quadrant, planned_start, planned_end, status, priority, source, waiting_review_at, created_at, updated_at FROM plan_items WHERE status = ?1 ORDER BY created_at DESC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![status.as_str()], |row| Ok(row_to_plan_item(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(items)
    }

    fn list_plan_items_due_for_review(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let mut stmt = self
            .conn
            .prepare("SELECT id, daily_log_id, title, description, quadrant, planned_start, planned_end, status, priority, source, waiting_review_at, created_at, updated_at FROM plan_items WHERE status = 'Waiting' AND waiting_review_at <= ?1 ORDER BY waiting_review_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![date_str], |row| Ok(row_to_plan_item(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(items)
    }

    // ── TaskCheckpoint ──

    fn insert_task_checkpoint(&self, cp: &TaskCheckpoint) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO task_checkpoints (id, plan_item_id, kind, status, response, created_at, answered_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                uuid_to_string(cp.id),
                uuid_to_string(cp.plan_item_id),
                cp.kind.as_str(),
                cp.status.as_str(),
                cp.response.as_ref().map(|r| r.as_str()),
                datetime_to_string(&cp.created_at),
                cp.answered_at.as_ref().map(datetime_to_string),
            ],
        )
        .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_task_checkpoint(
        &self,
        id: Id<TaskCheckpoint>,
    ) -> Result<Option<TaskCheckpoint>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, plan_item_id, kind, status, response, created_at, answered_at FROM task_checkpoints WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_task_checkpoint(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(cp)) => Ok(Some(cp?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_checkpoints_by_plan_item(
        &self,
        plan_item_id: Id<PlanItem>,
    ) -> Result<Vec<TaskCheckpoint>, Self::Error> {
        let item_id_str = uuid_to_string(plan_item_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, plan_item_id, kind, status, response, created_at, answered_at FROM task_checkpoints WHERE plan_item_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![item_id_str], |row| Ok(row_to_task_checkpoint(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut checkpoints = Vec::new();
        for row in rows {
            checkpoints.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(checkpoints)
    }

    fn update_task_checkpoint(&self, cp: &TaskCheckpoint) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE task_checkpoints SET kind = ?1, status = ?2, response = ?3, answered_at = ?4 WHERE id = ?5",
                params![
                    cp.kind.as_str(),
                    cp.status.as_str(),
                    cp.response.as_ref().map(|r| r.as_str()),
                    cp.answered_at.as_ref().map(datetime_to_string),
                    uuid_to_string(cp.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("task_checkpoint".into()));
        }
        Ok(())
    }

    fn delete_task_checkpoint(&self, id: Id<TaskCheckpoint>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute(
                "DELETE FROM task_checkpoints WHERE id = ?1",
                params![id_str],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("task_checkpoint".into()));
        }
        Ok(())
    }

    // ── ChecklistTemplate ──

    fn insert_checklist_template(&self, template: &ChecklistTemplate) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO checklist_templates (id, title, category, items_json, version, is_active, created_at, updated_at, slug) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                uuid_to_string(template.id),
                template.title,
                template.category,
                serde_json::to_string(&template.items).map_err(|e| CoreError::Storage(e.to_string()))?,
                template.version,
                template.is_active as i32,
                datetime_to_string(&template.created_at),
                datetime_to_string(&template.updated_at),
                template.slug,
            ],
        )
        .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_checklist_template(
        &self,
        id: Id<ChecklistTemplate>,
    ) -> Result<Option<ChecklistTemplate>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self.conn
            .prepare("SELECT id, title, category, items_json, version, is_active, created_at, updated_at, slug FROM checklist_templates WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_checklist_template(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(t)) => Ok(Some(t?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_checklist_templates(&self) -> Result<Vec<ChecklistTemplate>, Self::Error> {
        let mut stmt = self.conn
            .prepare("SELECT id, title, category, items_json, version, is_active, created_at, updated_at, slug FROM checklist_templates ORDER BY title")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| Ok(row_to_checklist_template(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut templates = Vec::new();
        for row in rows {
            templates.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(templates)
    }

    fn list_checklist_templates_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<ChecklistTemplate>, Self::Error> {
        let mut stmt = self.conn
            .prepare("SELECT id, title, category, items_json, version, is_active, created_at, updated_at, slug FROM checklist_templates WHERE category = ?1 ORDER BY title")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![category], |row| Ok(row_to_checklist_template(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut templates = Vec::new();
        for row in rows {
            templates.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(templates)
    }

    fn update_checklist_template(&self, template: &ChecklistTemplate) -> Result<(), Self::Error> {
        let affected = self.conn.execute(
            "UPDATE checklist_templates SET title = ?1, category = ?2, items_json = ?3, version = ?4, is_active = ?5, slug = ?6, updated_at = ?7 WHERE id = ?8",
            params![
                template.title,
                template.category,
                serde_json::to_string(&template.items).map_err(|e| CoreError::Storage(e.to_string()))?,
                template.version,
                template.is_active as i32,
                template.slug,
                datetime_to_string(&template.updated_at),
                uuid_to_string(template.id),
            ],
        ).map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("checklist_template".into()));
        }
        Ok(())
    }

    fn delete_checklist_template(&self, id: Id<ChecklistTemplate>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute(
                "DELETE FROM checklist_templates WHERE id = ?1",
                params![id_str],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("checklist_template".into()));
        }
        Ok(())
    }

    // ── ChecklistRun ──

    fn insert_checklist_run(&self, run: &ChecklistRun) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO checklist_runs (id, template_id, daily_log_id, started_at, completed_at, answers_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                uuid_to_string(run.id),
                uuid_to_string(run.template_id),
                uuid_to_string(run.daily_log_id),
                datetime_to_string(&run.started_at),
                run.completed_at.as_ref().map(datetime_to_string),
                serde_json::to_string(&run.answers).map_err(|e| CoreError::Storage(e.to_string()))?,
            ],
        ).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_checklist_run(&self, id: Id<ChecklistRun>) -> Result<Option<ChecklistRun>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self.conn
            .prepare("SELECT id, template_id, daily_log_id, started_at, completed_at, answers_json FROM checklist_runs WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_checklist_run(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_checklist_runs_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<ChecklistRun>, Self::Error> {
        let log_id_str = uuid_to_string(daily_log_id);
        let mut stmt = self.conn
            .prepare("SELECT id, template_id, daily_log_id, started_at, completed_at, answers_json FROM checklist_runs WHERE daily_log_id = ?1 ORDER BY started_at DESC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![log_id_str], |row| Ok(row_to_checklist_run(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut runs = Vec::new();
        for row in rows {
            runs.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(runs)
    }

    fn update_checklist_run(&self, run: &ChecklistRun) -> Result<(), Self::Error> {
        let affected = self.conn.execute(
            "UPDATE checklist_runs SET template_id = ?1, daily_log_id = ?2, started_at = ?3, completed_at = ?4, answers_json = ?5 WHERE id = ?6",
            params![
                uuid_to_string(run.template_id),
                uuid_to_string(run.daily_log_id),
                datetime_to_string(&run.started_at),
                run.completed_at.as_ref().map(datetime_to_string),
                serde_json::to_string(&run.answers).map_err(|e| CoreError::Storage(e.to_string()))?,
                uuid_to_string(run.id),
            ],
        ).map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("checklist_run".into()));
        }
        Ok(())
    }

    fn delete_checklist_run(&self, id: Id<ChecklistRun>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM checklist_runs WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("checklist_run".into()));
        }
        Ok(())
    }

    // ── JournalEntry ──

    fn insert_journal_entry(&self, entry: &JournalEntry) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO journal_entries (id, daily_log_id, timestamp, entry_type, summary) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                uuid_to_string(entry.id),
                uuid_to_string(entry.daily_log_id),
                datetime_to_string(&entry.timestamp),
                entry.entry_type.as_str(),
                entry.summary,
            ],
        ).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_journal_entry(&self, id: Id<JournalEntry>) -> Result<Option<JournalEntry>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self.conn
            .prepare("SELECT id, daily_log_id, timestamp, entry_type, summary FROM journal_entries WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_journal_entry(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(e)) => Ok(Some(e?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_journal_entries_by_log(
        &self,
        daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<JournalEntry>, Self::Error> {
        let log_id_str = uuid_to_string(daily_log_id);
        let mut stmt = self.conn
            .prepare("SELECT id, daily_log_id, timestamp, entry_type, summary FROM journal_entries WHERE daily_log_id = ?1 ORDER BY timestamp ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![log_id_str], |row| Ok(row_to_journal_entry(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(entries)
    }

    fn update_journal_entry(&self, entry: &JournalEntry) -> Result<(), Self::Error> {
        let affected = self.conn.execute(
            "UPDATE journal_entries SET daily_log_id = ?1, timestamp = ?2, entry_type = ?3, summary = ?4 WHERE id = ?5",
            params![
                uuid_to_string(entry.daily_log_id),
                datetime_to_string(&entry.timestamp),
                entry.entry_type.as_str(),
                entry.summary,
                uuid_to_string(entry.id),
            ],
        ).map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("journal_entry".into()));
        }
        Ok(())
    }

    fn delete_journal_entry(&self, id: Id<JournalEntry>) -> Result<(), Self::Error> {
        let id_str = uuid_to_string(id);
        let affected = self
            .conn
            .execute("DELETE FROM journal_entries WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("journal_entry".into()));
        }
        Ok(())
    }

    fn insert_checkin_composite(
        &self,
        ci: &CheckIn,
        daily_log_update: Option<&DailyLog>,
        journal: &JournalEntry,
    ) -> Result<(), CoreError> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = self
            .insert_check_in(ci)
            .and_then(|()| {
                if let Some(log) = daily_log_update {
                    self.update_daily_log(log)
                } else {
                    Ok(())
                }
            })
            .and_then(|()| self.insert_journal_entry(journal));
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn start_plan_item_composite(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
        journal: &JournalEntry,
    ) -> Result<(), CoreError> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = self
            .update_plan_item(item)
            .and_then(|()| self.insert_task_checkpoint(checkpoint))
            .and_then(|()| self.insert_journal_entry(journal));
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn done_plan_item_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), CoreError> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = self
            .update_plan_item(item)
            .and_then(|()| self.insert_journal_entry(journal));
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn move_to_waiting_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), CoreError> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = self
            .update_plan_item(item)
            .and_then(|()| self.insert_journal_entry(journal));
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn complete_checklist_composite(
        &self,
        run: &ChecklistRun,
        journal: &JournalEntry,
    ) -> Result<(), CoreError> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = self
            .update_checklist_run(run)
            .and_then(|()| self.insert_journal_entry(journal));
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn insert_plan_item_with_checkpoint(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
    ) -> Result<(), CoreError> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = self
            .insert_plan_item(item)
            .and_then(|()| self.insert_task_checkpoint(checkpoint));
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn answer_checkpoint_composite(
        &self,
        checkpoint: &TaskCheckpoint,
        next_checkpoint: Option<&TaskCheckpoint>,
        journal: &JournalEntry,
    ) -> Result<(), CoreError> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = (|| {
            self.update_task_checkpoint(checkpoint)?;
            if let Some(next_cp) = next_checkpoint {
                self.insert_task_checkpoint(next_cp)?;
            }
            self.insert_journal_entry(journal)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    // ── ImportRun ──

    fn insert_import_run(&self, run: &ImportRun) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO import_runs (id, bundle_id, bundle_title, schema_version, mode, status, started_at, finished_at, summary_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                uuid_to_string(run.id),
                run.bundle_id,
                run.bundle_title,
                run.schema_version,
                run.mode,
                run.status,
                datetime_to_string(&run.started_at),
                datetime_to_string(&run.finished_at),
                run.summary_json,
            ],
        ).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn list_import_runs(&self) -> Result<Vec<ImportRun>, Self::Error> {
        let mut stmt = self.conn
            .prepare("SELECT id, bundle_id, bundle_title, schema_version, mode, status, started_at, finished_at, summary_json FROM import_runs ORDER BY started_at DESC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| Ok(row_to_import_run(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut runs = Vec::new();
        for row in rows {
            runs.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(runs)
    }

    // ── Slug-based lookups ──

    fn get_checklist_template_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ChecklistTemplate>, Self::Error> {
        let mut stmt = self.conn
            .prepare("SELECT id, title, category, items_json, version, is_active, created_at, updated_at, slug FROM checklist_templates WHERE slug = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![slug], |row| Ok(row_to_checklist_template(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(t)) => Ok(Some(t?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn upsert_checklist_template(&self, template: &ChecklistTemplate) -> Result<(), Self::Error> {
        if let Some(ref slug) = template.slug {
            let existing = self.get_checklist_template_by_slug(slug)?;
            if let Some(existing_template) = existing {
                self.conn.execute(
                    "UPDATE checklist_templates SET title = ?1, category = ?2, items_json = ?3, version = ?4, is_active = ?5, updated_at = ?6 WHERE id = ?7",
                    params![
                        template.title,
                        template.category,
                        json_to_string(&template.items),
                        template.version,
                        bool_to_int(template.is_active),
                        datetime_to_string(&template.updated_at),
                        uuid_to_string(existing_template.id),
                    ],
                ).map_err(|e| CoreError::Storage(e.to_string()))?;
            } else {
                self.insert_checklist_template(template)?;
            }
        } else {
            self.insert_checklist_template(template)?;
        }
        Ok(())
    }

    fn get_context_document_by_source_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ContextDocument>, Self::Error> {
        let mut stmt = self.conn
            .prepare("SELECT id, doc_type, title, content_markdown, version, is_active, created_at, updated_at, source_slug, source_metadata_json FROM context_documents WHERE source_slug = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![slug], |row| Ok(row_to_context_document(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(cd)) => Ok(Some(cd?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn upsert_context_document(&self, doc: &ContextDocument) -> Result<(), Self::Error> {
        if let Some(ref slug) = doc.source_slug {
            let existing = self.get_context_document_by_source_slug(slug)?;
            if let Some(existing_doc) = existing {
                self.conn.execute(
                    "UPDATE context_documents SET doc_type = ?1, title = ?2, content_markdown = ?3, version = ?4, is_active = ?5, source_metadata_json = ?6, updated_at = ?7 WHERE id = ?8",
                    params![
                        json_to_string(&doc.doc_type),
                        doc.title,
                        doc.content_markdown,
                        doc.version as i64,
                        bool_to_int(doc.is_active),
                        doc.source_metadata_json,
                        datetime_to_string(&doc.updated_at),
                        uuid_to_string(existing_doc.id),
                    ],
                ).map_err(|e| CoreError::Storage(e.to_string()))?;
            } else {
                self.insert_context_document(doc)?;
            }
        } else {
            self.insert_context_document(doc)?;
        }
        Ok(())
    }

    fn bundle_apply_composite(
        &self,
        bundle: &AdiyutantBundle,
        mode: ImportMode,
        import_run: &ImportRun,
    ) -> Result<(), Self::Error> {
        self.conn
            .execute("BEGIN IMMEDIATE", [])
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let result = self.apply_bundle_inner(bundle, mode, import_run);
        match result {
            Ok(()) => {
                self.conn
                    .execute("COMMIT", [])
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    // ── Project ──

    fn insert_project(&self, p: &Project) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO projects (id, slug, title, description, status, priority, why, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    uuid_to_string(p.id),
                    p.slug,
                    p.title,
                    p.description,
                    p.status.as_str(),
                    p.priority as i64,
                    p.why,
                    datetime_to_string(&p.created_at),
                    datetime_to_string(&p.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_project(&self, id: Id<Project>) -> Result<Option<Project>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, title, description, status, priority, why, created_at, updated_at FROM projects WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_project(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(p)) => Ok(Some(p?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn get_project_by_slug(&self, slug: &str) -> Result<Option<Project>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, title, description, status, priority, why, created_at, updated_at FROM projects WHERE slug = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![slug], |row| Ok(row_to_project(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(p)) => Ok(Some(p?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_projects(&self) -> Result<Vec<Project>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, title, description, status, priority, why, created_at, updated_at FROM projects ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| Ok(row_to_project(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_project(&self, p: &Project) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE projects SET slug = ?1, title = ?2, description = ?3, status = ?4, priority = ?5, why = ?6, updated_at = ?7 WHERE id = ?8",
                params![
                    p.slug,
                    p.title,
                    p.description,
                    p.status.as_str(),
                    p.priority as i64,
                    p.why,
                    datetime_to_string(&p.updated_at),
                    uuid_to_string(p.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("project".into()));
        }
        Ok(())
    }

    // ── Roadmap ──

    fn insert_roadmap(&self, r: &Roadmap) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO roadmaps (id, slug, project_id, title, description, horizon, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    uuid_to_string(r.id),
                    r.slug,
                    uuid_to_string(r.project_id),
                    r.title,
                    r.description,
                    r.horizon.as_str(),
                    r.status.as_str(),
                    datetime_to_string(&r.created_at),
                    datetime_to_string(&r.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_roadmap(&self, id: Id<Roadmap>) -> Result<Option<Roadmap>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, project_id, title, description, horizon, status, created_at, updated_at FROM roadmaps WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_roadmap(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn get_roadmap_by_project_and_slug(
        &self,
        project_id: Id<Project>,
        slug: &str,
    ) -> Result<Option<Roadmap>, Self::Error> {
        let pid_str = uuid_to_string(project_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, project_id, title, description, horizon, status, created_at, updated_at FROM roadmaps WHERE project_id = ?1 AND slug = ?2")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![pid_str, slug], |row| Ok(row_to_roadmap(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_roadmaps(&self) -> Result<Vec<Roadmap>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, project_id, title, description, horizon, status, created_at, updated_at FROM roadmaps ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| Ok(row_to_roadmap(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn list_roadmaps_by_project(
        &self,
        project_id: Id<Project>,
    ) -> Result<Vec<Roadmap>, Self::Error> {
        let pid_str = uuid_to_string(project_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, project_id, title, description, horizon, status, created_at, updated_at FROM roadmaps WHERE project_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![pid_str], |row| Ok(row_to_roadmap(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_roadmap(&self, r: &Roadmap) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE roadmaps SET slug = ?1, project_id = ?2, title = ?3, description = ?4, horizon = ?5, status = ?6, updated_at = ?7 WHERE id = ?8",
                params![
                    r.slug,
                    uuid_to_string(r.project_id),
                    r.title,
                    r.description,
                    r.horizon.as_str(),
                    r.status.as_str(),
                    datetime_to_string(&r.updated_at),
                    uuid_to_string(r.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("roadmap".into()));
        }
        Ok(())
    }

    // ── RoadmapPhase ──

    fn insert_roadmap_phase(&self, p: &RoadmapPhase) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO roadmap_phases (id, slug, roadmap_id, title, order_index, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    uuid_to_string(p.id),
                    p.slug,
                    uuid_to_string(p.roadmap_id),
                    p.title,
                    p.order_index as i64,
                    p.status.as_str(),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_roadmap_phase(&self, id: Id<RoadmapPhase>) -> Result<Option<RoadmapPhase>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, roadmap_id, title, order_index, status FROM roadmap_phases WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_roadmap_phase(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(p)) => Ok(Some(p?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_roadmap_phases(
        &self,
        roadmap_id: Id<Roadmap>,
    ) -> Result<Vec<RoadmapPhase>, Self::Error> {
        let rid_str = uuid_to_string(roadmap_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, roadmap_id, title, order_index, status FROM roadmap_phases WHERE roadmap_id = ?1 ORDER BY order_index ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![rid_str], |row| Ok(row_to_roadmap_phase(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn list_all_roadmap_phases(&self) -> Result<Vec<RoadmapPhase>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, roadmap_id, title, order_index, status FROM roadmap_phases ORDER BY order_index ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| Ok(row_to_roadmap_phase(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_roadmap_phase(&self, p: &RoadmapPhase) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE roadmap_phases SET slug = ?1, roadmap_id = ?2, title = ?3, order_index = ?4, status = ?5 WHERE id = ?6",
                params![
                    p.slug,
                    uuid_to_string(p.roadmap_id),
                    p.title,
                    p.order_index as i64,
                    p.status.as_str(),
                    uuid_to_string(p.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("roadmap_phase".into()));
        }
        Ok(())
    }

    // ── RoadmapItem ──

    fn insert_roadmap_item(&self, i: &RoadmapItem) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO roadmap_items (id, slug, phase_id, title, description, status, priority, acceptance_criteria_json, depends_on_json, links_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    uuid_to_string(i.id),
                    i.slug,
                    uuid_to_string(i.phase_id),
                    i.title,
                    i.description,
                    i.status.as_str(),
                    i.priority as i64,
                    json_to_string(&i.acceptance_criteria),
                    json_to_string(&i.depends_on),
                    json_to_string(&i.links),
                    datetime_to_string(&i.created_at),
                    datetime_to_string(&i.updated_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_roadmap_item(&self, id: Id<RoadmapItem>) -> Result<Option<RoadmapItem>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, phase_id, title, description, status, priority, acceptance_criteria_json, depends_on_json, links_json, created_at, updated_at FROM roadmap_items WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_roadmap_item(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(i)) => Ok(Some(i?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_roadmap_items(
        &self,
        phase_id: Id<RoadmapPhase>,
    ) -> Result<Vec<RoadmapItem>, Self::Error> {
        let pid_str = uuid_to_string(phase_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, phase_id, title, description, status, priority, acceptance_criteria_json, depends_on_json, links_json, created_at, updated_at FROM roadmap_items WHERE phase_id = ?1 ORDER BY priority ASC, created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![pid_str], |row| Ok(row_to_roadmap_item(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn list_all_roadmap_items(&self) -> Result<Vec<RoadmapItem>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, slug, phase_id, title, description, status, priority, acceptance_criteria_json, depends_on_json, links_json, created_at, updated_at FROM roadmap_items ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| Ok(row_to_roadmap_item(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn update_roadmap_item(&self, i: &RoadmapItem) -> Result<(), Self::Error> {
        let affected = self
            .conn
            .execute(
                "UPDATE roadmap_items SET slug = ?1, phase_id = ?2, title = ?3, description = ?4, status = ?5, priority = ?6, acceptance_criteria_json = ?7, depends_on_json = ?8, links_json = ?9, updated_at = ?10 WHERE id = ?11",
                params![
                    i.slug,
                    uuid_to_string(i.phase_id),
                    i.title,
                    i.description,
                    i.status.as_str(),
                    i.priority as i64,
                    json_to_string(&i.acceptance_criteria),
                    json_to_string(&i.depends_on),
                    json_to_string(&i.links),
                    datetime_to_string(&i.updated_at),
                    uuid_to_string(i.id),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(CoreError::NotFound("roadmap_item".into()));
        }
        Ok(())
    }

    // ── RoadmapPlanLink ──

    fn insert_roadmap_plan_link(&self, link: &RoadmapPlanLink) -> Result<(), Self::Error> {
        self.conn
            .execute(
                "INSERT INTO roadmap_plan_links (id, roadmap_item_id, plan_item_id, link_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    uuid_to_string(link.id),
                    uuid_to_string(link.roadmap_item_id),
                    uuid_to_string(link.plan_item_id),
                    link.link_type,
                    datetime_to_string(&link.created_at),
                ],
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_roadmap_plan_link(
        &self,
        id: Id<RoadmapPlanLink>,
    ) -> Result<Option<RoadmapPlanLink>, Self::Error> {
        let id_str = uuid_to_string(id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, roadmap_item_id, plan_item_id, link_type, created_at FROM roadmap_plan_links WHERE id = ?1")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id_str], |row| Ok(row_to_roadmap_plan_link(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(l)) => Ok(Some(l?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    fn list_roadmap_plan_links_by_roadmap_item(
        &self,
        roadmap_item_id: Id<RoadmapItem>,
    ) -> Result<Vec<RoadmapPlanLink>, Self::Error> {
        let rid_str = uuid_to_string(roadmap_item_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, roadmap_item_id, plan_item_id, link_type, created_at FROM roadmap_plan_links WHERE roadmap_item_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![rid_str], |row| Ok(row_to_roadmap_plan_link(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn list_roadmap_plan_links_by_plan_item(
        &self,
        plan_item_id: Id<PlanItem>,
    ) -> Result<Vec<RoadmapPlanLink>, Self::Error> {
        let pid_str = uuid_to_string(plan_item_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, roadmap_item_id, plan_item_id, link_type, created_at FROM roadmap_plan_links WHERE plan_item_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![pid_str], |row| Ok(row_to_roadmap_plan_link(row)))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CoreError::Storage(e.to_string()))??);
        }
        Ok(result)
    }

    fn find_roadmap_plan_link(
        &self,
        roadmap_item_id: Id<RoadmapItem>,
        plan_item_id: Id<PlanItem>,
    ) -> Result<Option<RoadmapPlanLink>, Self::Error> {
        let rid_str = uuid_to_string(roadmap_item_id);
        let pid_str = uuid_to_string(plan_item_id);
        let mut stmt = self
            .conn
            .prepare("SELECT id, roadmap_item_id, plan_item_id, link_type, created_at FROM roadmap_plan_links WHERE roadmap_item_id = ?1 AND plan_item_id = ?2")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![rid_str, pid_str], |row| {
                Ok(row_to_roadmap_plan_link(row))
            })
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        match rows.next() {
            Some(Ok(l)) => Ok(Some(l?)),
            Some(Err(e)) => Err(CoreError::Storage(e.to_string())),
            None => Ok(None),
        }
    }

    // ── Take-item composite ──

    fn take_roadmap_item_composite(
        &self,
        input: TakeRoadmapItemInput,
    ) -> Result<TakeRoadmapItemResult, Self::Error> {
        if let Some(existing_link) =
            self.find_roadmap_plan_link(input.roadmap_item_id, input.plan_item.id)?
        {
            return Ok(TakeRoadmapItemResult {
                plan_item_id: existing_link.plan_item_id,
                link_id: existing_link.id,
                plan_item_reused: true,
                link_reused: true,
            });
        }

        self.with_transaction(|conn| {
            if let Some(dl) = &input.daily_log_to_create {
                conn.execute(
                    "INSERT INTO daily_logs (id, date, mode, sleep_score, energy, mood, raw_notes, ai_summary, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        uuid_to_string(dl.id),
                        dl.date.to_string(),
                        dl.mode,
                        dl.sleep_score.map(|v| v as i64),
                        dl.energy.map(|v| v as i64),
                        dl.mood.map(|v| v as i64),
                        dl.raw_notes,
                        dl.ai_summary,
                        datetime_to_string(&dl.created_at),
                        datetime_to_string(&dl.updated_at),
                    ],
                ).map_err(|e| CoreError::Storage(e.to_string()))?;
            }

            let pi = &input.plan_item;
            conn.execute(
                "INSERT INTO plan_items (id, daily_log_id, title, description, quadrant, planned_start, planned_end, status, priority, source, waiting_review_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    uuid_to_string(pi.id),
                    uuid_to_string(pi.daily_log_id),
                    pi.title,
                    pi.description,
                    pi.quadrant.as_str(),
                    pi.planned_start.map(|t| t.format("%H:%M").to_string()),
                    pi.planned_end.map(|t| t.format("%H:%M").to_string()),
                    pi.status.as_str(),
                    pi.priority,
                    pi.source.as_str(),
                    pi.waiting_review_at.map(|d| d.format("%Y-%m-%d").to_string()),
                    datetime_to_string(&pi.created_at),
                    datetime_to_string(&pi.updated_at),
                ],
            ).map_err(|e| CoreError::Storage(e.to_string()))?;

            let cp = &input.initial_checkpoint;
            conn.execute(
                "INSERT INTO task_checkpoints (id, plan_item_id, kind, status, response, created_at, answered_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    uuid_to_string(cp.id),
                    uuid_to_string(cp.plan_item_id),
                    cp.kind.as_str(),
                    cp.status.as_str(),
                    cp.response.as_ref().map(|r| r.as_str()),
                    datetime_to_string(&cp.created_at),
                    cp.answered_at.as_ref().map(datetime_to_string),
                ],
            ).map_err(|e| CoreError::Storage(e.to_string()))?;

            let link = &input.link;
            conn.execute(
                "INSERT INTO roadmap_plan_links (id, roadmap_item_id, plan_item_id, link_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    uuid_to_string(link.id),
                    uuid_to_string(link.roadmap_item_id),
                    uuid_to_string(link.plan_item_id),
                    link.link_type,
                    datetime_to_string(&link.created_at),
                ],
            ).map_err(|e| CoreError::Storage(e.to_string()))?;

            if input.activate_item {
                conn.execute(
                    "UPDATE roadmap_items SET status = 'active' WHERE id = ?1 AND status = 'planned'",
                    params![uuid_to_string(input.roadmap_item_id)],
                ).map_err(|e| CoreError::Storage(e.to_string()))?;
            }

            Ok(())
        })?;

        Ok(TakeRoadmapItemResult {
            plan_item_id: input.plan_item.id,
            link_id: input.link.id,
            plan_item_reused: false,
            link_reused: false,
        })
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
    let source_slug: Option<String> = row.get(8).ok();
    let source_metadata_json: Option<String> = row.get(9).ok();
    Ok(ContextDocument {
        id: parse_uuid(row_get::<String>(row, 0)?.as_str())?,
        doc_type: json_from_str(&row_get::<String>(row, 1)?)?,
        title: row_get(row, 2)?,
        content_markdown: row_get(row, 3)?,
        version: row_get::<i64>(row, 4)? as u32,
        is_active: int_to_bool(row_get::<i64>(row, 5)?),
        created_at: deserialize_datetime(&row_get::<String>(row, 6)?)?,
        updated_at: deserialize_datetime(&row_get::<String>(row, 7)?)?,
        source_slug,
        source_metadata_json,
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

fn row_to_project(row: &rusqlite::Row<'_>) -> Result<Project, CoreError> {
    use adiyutant_core::model::project::ProjectStatus;
    let status_str: String = row_get(row, 4)?;
    Ok(Project {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        slug: row_get(row, 1)?,
        title: row_get(row, 2)?,
        description: row_get(row, 3)?,
        status: ProjectStatus::from_str(&status_str).ok_or_else(|| {
            CoreError::InvalidInput(format!("invalid project status: {status_str}"))
        })?,
        priority: row_get::<i64>(row, 5)? as u8,
        why: row_get(row, 6)?,
        created_at: parse_datetime(&row_get::<String>(row, 7)?)?,
        updated_at: parse_datetime(&row_get::<String>(row, 8)?)?,
    })
}

fn row_to_roadmap(row: &rusqlite::Row<'_>) -> Result<Roadmap, CoreError> {
    use adiyutant_core::model::project::ProjectStatus;
    use adiyutant_core::model::roadmap::RoadmapHorizon;
    let horizon_str: String = row_get(row, 5)?;
    let status_str: String = row_get(row, 6)?;
    Ok(Roadmap {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        slug: row_get(row, 1)?,
        project_id: parse_uuid(&row_get::<String>(row, 2)?)?,
        title: row_get(row, 3)?,
        description: row_get(row, 4)?,
        horizon: RoadmapHorizon::from_str(&horizon_str)
            .ok_or_else(|| CoreError::InvalidInput(format!("invalid horizon: {horizon_str}")))?,
        status: ProjectStatus::from_str(&status_str).ok_or_else(|| {
            CoreError::InvalidInput(format!("invalid roadmap status: {status_str}"))
        })?,
        created_at: parse_datetime(&row_get::<String>(row, 7)?)?,
        updated_at: parse_datetime(&row_get::<String>(row, 8)?)?,
    })
}

fn row_to_roadmap_phase(row: &rusqlite::Row<'_>) -> Result<RoadmapPhase, CoreError> {
    use adiyutant_core::model::roadmap::PhaseStatus;
    let status_str: String = row_get(row, 5)?;
    Ok(RoadmapPhase {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        slug: row_get(row, 1)?,
        roadmap_id: parse_uuid(&row_get::<String>(row, 2)?)?,
        title: row_get(row, 3)?,
        order_index: row_get::<i64>(row, 4)? as u32,
        status: PhaseStatus::from_str(&status_str).ok_or_else(|| {
            CoreError::InvalidInput(format!("invalid phase status: {status_str}"))
        })?,
    })
}

fn row_to_roadmap_item(row: &rusqlite::Row<'_>) -> Result<RoadmapItem, CoreError> {
    use adiyutant_core::model::roadmap::RoadmapItemStatus;
    let status_str: String = row_get(row, 5)?;
    Ok(RoadmapItem {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        slug: row_get(row, 1)?,
        phase_id: parse_uuid(&row_get::<String>(row, 2)?)?,
        title: row_get(row, 3)?,
        description: row_get(row, 4)?,
        status: RoadmapItemStatus::from_str(&status_str)
            .ok_or_else(|| CoreError::InvalidInput(format!("invalid item status: {status_str}")))?,
        priority: row_get::<i64>(row, 6)? as u8,
        acceptance_criteria: json_from_str(&row_get::<String>(row, 7)?)?,
        depends_on: json_from_str(&row_get::<String>(row, 8)?)?,
        links: json_from_str(&row_get::<String>(row, 9)?)?,
        created_at: parse_datetime(&row_get::<String>(row, 10)?)?,
        updated_at: parse_datetime(&row_get::<String>(row, 11)?)?,
    })
}

fn row_to_roadmap_plan_link(row: &rusqlite::Row<'_>) -> Result<RoadmapPlanLink, CoreError> {
    Ok(RoadmapPlanLink {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        roadmap_item_id: parse_uuid(&row_get::<String>(row, 1)?)?,
        plan_item_id: parse_uuid(&row_get::<String>(row, 2)?)?,
        link_type: row_get(row, 3)?,
        created_at: parse_datetime(&row_get::<String>(row, 4)?)?,
    })
}

fn row_to_plan_item(row: &rusqlite::Row<'_>) -> Result<PlanItem, CoreError> {
    use adiyutant_core::model::day_plan::{EisenhowerQuadrant, PlanItemStatus, PlanningMode};

    let id: String = row_get(row, 0)?;
    let daily_log_id: String = row_get(row, 1)?;
    let title: String = row_get(row, 2)?;
    let description: Option<String> = row_get(row, 3)?;
    let quadrant_str: String = row_get(row, 4)?;
    let planned_start_str: Option<String> = row_get(row, 5)?;
    let planned_end_str: Option<String> = row_get(row, 6)?;
    let status_str: String = row_get(row, 7)?;
    let priority_i64: i64 = row_get(row, 8)?;
    let source_str: String = row_get(row, 9)?;
    let waiting_review_at_str: Option<String> = row_get(row, 10)?;
    let created_at = row_get_datetime(row, 11)?;
    let updated_at = row_get_datetime(row, 12)?;

    Ok(PlanItem {
        id: parse_uuid(&id)?,
        daily_log_id: parse_uuid(&daily_log_id)?,
        title,
        description,
        quadrant: EisenhowerQuadrant::from_str(&quadrant_str)
            .ok_or_else(|| CoreError::InvalidInput(format!("invalid quadrant: {quadrant_str}")))?,
        planned_start: planned_start_str.and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M").ok()),
        planned_end: planned_end_str.and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M").ok()),
        status: PlanItemStatus::from_str(&status_str)
            .ok_or_else(|| CoreError::InvalidInput(format!("invalid status: {status_str}")))?,
        priority: priority_i64 as u8,
        source: PlanningMode::from_str(&source_str)
            .ok_or_else(|| CoreError::InvalidInput(format!("invalid source: {source_str}")))?,
        waiting_review_at: waiting_review_at_str
            .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
        created_at,
        updated_at,
    })
}

fn row_to_task_checkpoint(row: &rusqlite::Row<'_>) -> Result<TaskCheckpoint, CoreError> {
    use adiyutant_core::model::task_checkpoint::{
        CheckpointKind, CheckpointResponse, CheckpointStatus,
    };

    let id: String = row_get(row, 0)?;
    let plan_item_id: String = row_get(row, 1)?;
    let kind_str: String = row_get(row, 2)?;
    let status_str: String = row_get(row, 3)?;
    let response_str: Option<String> = row_get(row, 4)?;
    let created_at = row_get_datetime(row, 5)?;
    let answered_at_str: Option<String> = row_get(row, 6)?;

    Ok(TaskCheckpoint {
        id: parse_uuid(&id)?,
        plan_item_id: parse_uuid(&plan_item_id)?,
        kind: CheckpointKind::from_str(&kind_str)
            .ok_or_else(|| CoreError::InvalidInput(format!("invalid kind: {kind_str}")))?,
        status: CheckpointStatus::from_str(&status_str)
            .ok_or_else(|| CoreError::InvalidInput(format!("invalid status: {status_str}")))?,
        response: response_str.and_then(|r| CheckpointResponse::from_str(&r)),
        created_at,
        answered_at: answered_at_str.map(|t| parse_datetime(&t)).transpose()?,
    })
}

fn row_to_checklist_template(row: &rusqlite::Row<'_>) -> Result<ChecklistTemplate, CoreError> {
    let items_json: String = row_get(row, 3)?;
    let slug: Option<String> = row.get(8).ok();
    Ok(ChecklistTemplate {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        title: row_get(row, 1)?,
        category: row_get(row, 2)?,
        items: serde_json::from_str(&items_json).unwrap_or_default(),
        version: row_get::<i64>(row, 4)? as u32,
        is_active: row_get::<i64>(row, 5)? != 0,
        created_at: parse_datetime(&row_get::<String>(row, 6)?)?,
        updated_at: parse_datetime(&row_get::<String>(row, 7)?)?,
        slug,
    })
}

fn row_to_checklist_run(row: &rusqlite::Row<'_>) -> Result<ChecklistRun, CoreError> {
    let answers_json: String = row_get(row, 5)?;
    Ok(ChecklistRun {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        template_id: parse_uuid(&row_get::<String>(row, 1)?)?,
        daily_log_id: parse_uuid(&row_get::<String>(row, 2)?)?,
        started_at: parse_datetime(&row_get::<String>(row, 3)?)?,
        completed_at: row_get::<Option<String>>(row, 4)?
            .map(|s| parse_datetime(&s))
            .transpose()?,
        answers: serde_json::from_str(&answers_json).unwrap_or_default(),
    })
}

fn row_to_journal_entry(row: &rusqlite::Row<'_>) -> Result<JournalEntry, CoreError> {
    use adiyutant_core::model::journal_entry::JournalEntryType;

    let entry_type_raw: String = row_get(row, 3)?;
    Ok(JournalEntry {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        daily_log_id: parse_uuid(&row_get::<String>(row, 1)?)?,
        timestamp: parse_datetime(&row_get::<String>(row, 2)?)?,
        entry_type: JournalEntryType::from_str(&entry_type_raw).ok_or_else(|| {
            CoreError::InvalidInput(format!("invalid entry_type: {entry_type_raw}"))
        })?,
        summary: row_get(row, 4)?,
    })
}

fn row_to_import_run(row: &rusqlite::Row<'_>) -> Result<ImportRun, CoreError> {
    Ok(ImportRun {
        id: parse_uuid(&row_get::<String>(row, 0)?)?,
        bundle_id: row_get(row, 1)?,
        bundle_title: row_get(row, 2)?,
        schema_version: row_get(row, 3)?,
        mode: row_get(row, 4)?,
        status: row_get(row, 5)?,
        started_at: parse_datetime(&row_get::<String>(row, 6)?)?,
        finished_at: parse_datetime(&row_get::<String>(row, 7)?)?,
        summary_json: row_get(row, 8)?,
    })
}

// ──────────────────────────────────────────────
// Bundle apply helpers (used by bundle_apply_composite)
// ──────────────────────────────────────────────

fn apply_context_doc_internal(
    slug: &Option<String>,
    title: &str,
    doc_type_str: &str,
    content: &str,
    store: &SqliteStore,
    mode: ImportMode,
) -> Result<(), CoreError> {
    let lookup = slug.as_deref().unwrap_or(title);
    let doc_type = parse_doc_type_internal(doc_type_str);
    let exists = store
        .get_context_document_by_source_slug(lookup)
        .ok()
        .flatten();

    match (exists, mode) {
        (Some(mut existing), ImportMode::Merge) => {
            existing.title = title.to_string();
            existing.doc_type = doc_type;
            existing.content_markdown = content.to_string();
            existing.version += 1;
            existing.updated_at = adiyutant_core::datetime::AdiyutantDateTime::now();
            store.upsert_context_document(&existing)?;
        }
        (Some(_), ImportMode::Append) => {}
        (None, _) => {
            let doc = adiyutant_core::model::context_document::ContextDocument::with_source(
                doc_type,
                title.to_string(),
                content.to_string(),
                slug.clone(),
                None,
            );
            store.upsert_context_document(&doc)?;
        }
    }
    Ok(())
}

fn apply_checklist_template_internal(
    bundle_ct: &BundleChecklistTemplate,
    store: &SqliteStore,
    mode: ImportMode,
) -> Result<(), CoreError> {
    let exists = store
        .get_checklist_template_by_slug(&bundle_ct.slug)
        .ok()
        .flatten();

    match (exists, mode) {
        (Some(mut existing), ImportMode::Merge) => {
            existing.title = bundle_ct.title.clone();
            existing.category = bundle_ct.category.clone();
            existing.items = convert_checklist_items(&bundle_ct.items, existing.id);
            existing.version += 1;
            existing.updated_at = adiyutant_core::datetime::AdiyutantDateTime::now();
            store.upsert_checklist_template(&existing)?;
        }
        (Some(_), ImportMode::Append) => {}
        (None, _) => {
            let mut template = ChecklistTemplate::with_slug(
                bundle_ct.title.clone(),
                bundle_ct.category.clone(),
                Some(bundle_ct.slug.clone()),
            );
            template.items = convert_checklist_items(&bundle_ct.items, template.id);
            store.upsert_checklist_template(&template)?;
        }
    }
    Ok(())
}

fn convert_checklist_items(
    items: &[BundleChecklistItem],
    template_id: Id<ChecklistTemplate>,
) -> Vec<ChecklistItem> {
    items
        .iter()
        .map(|i| {
            let mut item = ChecklistItem::new(
                template_id,
                i.question.clone(),
                parse_checklist_kind_internal(&i.kind),
                i.order,
            );
            item.options = i.options.clone();
            item.slug = i.slug.clone();
            item
        })
        .collect()
}

fn parse_checklist_kind_internal(s: &str) -> ChecklistItemKind {
    match s.to_lowercase().as_str() {
        "checkbox" => ChecklistItemKind::Checkbox,
        "choice" => ChecklistItemKind::Choice,
        "scale" => ChecklistItemKind::Scale,
        "text" => ChecklistItemKind::Text,
        "optionalcomment" | "optional_comment" => ChecklistItemKind::OptionalComment,
        "habitevent" | "habit_event" => ChecklistItemKind::HabitEvent,
        "tasklink" | "task_link" => ChecklistItemKind::TaskLink,
        "timerstart" | "timer_start" => ChecklistItemKind::TimerStart,
        _ => ChecklistItemKind::Text,
    }
}

fn parse_doc_type_internal(s: &str) -> ContextDocumentType {
    serde_json::from_value(serde_json::Value::String(s.to_lowercase()))
        .unwrap_or(ContextDocumentType::Custom)
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

    // ── PlanItem (Noop) ──
    fn insert_plan_item(&self, _item: &PlanItem) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_plan_item(&self, _id: Id<PlanItem>) -> Result<Option<PlanItem>, Self::Error> {
        Ok(None)
    }
    fn list_plan_items_by_log(
        &self,
        _daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        Ok(vec![])
    }
    fn update_plan_item(&self, _item: &PlanItem) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_plan_item(&self, _id: Id<PlanItem>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn list_plan_items_by_status(
        &self,
        _status: PlanItemStatus,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        Ok(vec![])
    }
    fn list_plan_items_due_for_review(
        &self,
        _date: NaiveDate,
    ) -> Result<Vec<PlanItem>, Self::Error> {
        Ok(vec![])
    }
    // ── TaskCheckpoint (Noop) ──
    fn insert_task_checkpoint(&self, _cp: &TaskCheckpoint) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_task_checkpoint(
        &self,
        _id: Id<TaskCheckpoint>,
    ) -> Result<Option<TaskCheckpoint>, Self::Error> {
        Ok(None)
    }
    fn list_checkpoints_by_plan_item(
        &self,
        _plan_item_id: Id<PlanItem>,
    ) -> Result<Vec<TaskCheckpoint>, Self::Error> {
        Ok(vec![])
    }
    fn update_task_checkpoint(&self, _cp: &TaskCheckpoint) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_task_checkpoint(&self, _id: Id<TaskCheckpoint>) -> Result<(), Self::Error> {
        Ok(())
    }

    // ── ChecklistTemplate (Noop) ──
    fn insert_checklist_template(&self, _template: &ChecklistTemplate) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_checklist_template(
        &self,
        _id: Id<ChecklistTemplate>,
    ) -> Result<Option<ChecklistTemplate>, Self::Error> {
        Ok(None)
    }
    fn list_checklist_templates(&self) -> Result<Vec<ChecklistTemplate>, Self::Error> {
        Ok(vec![])
    }
    fn list_checklist_templates_by_category(
        &self,
        _category: &str,
    ) -> Result<Vec<ChecklistTemplate>, Self::Error> {
        Ok(vec![])
    }
    fn update_checklist_template(&self, _template: &ChecklistTemplate) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_checklist_template(&self, _id: Id<ChecklistTemplate>) -> Result<(), Self::Error> {
        Ok(())
    }
    // ── ChecklistRun (Noop) ──
    fn insert_checklist_run(&self, _run: &ChecklistRun) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_checklist_run(
        &self,
        _id: Id<ChecklistRun>,
    ) -> Result<Option<ChecklistRun>, Self::Error> {
        Ok(None)
    }
    fn list_checklist_runs_by_log(
        &self,
        _daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<ChecklistRun>, Self::Error> {
        Ok(vec![])
    }
    fn update_checklist_run(&self, _run: &ChecklistRun) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_checklist_run(&self, _id: Id<ChecklistRun>) -> Result<(), Self::Error> {
        Ok(())
    }
    // ── JournalEntry (Noop) ──
    fn insert_journal_entry(&self, _entry: &JournalEntry) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_journal_entry(
        &self,
        _id: Id<JournalEntry>,
    ) -> Result<Option<JournalEntry>, Self::Error> {
        Ok(None)
    }
    fn list_journal_entries_by_log(
        &self,
        _daily_log_id: Id<DailyLog>,
    ) -> Result<Vec<JournalEntry>, Self::Error> {
        Ok(vec![])
    }
    fn update_journal_entry(&self, _entry: &JournalEntry) -> Result<(), Self::Error> {
        Ok(())
    }
    fn delete_journal_entry(&self, _id: Id<JournalEntry>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn insert_checkin_composite(
        &self,
        ci: &CheckIn,
        daily_log_update: Option<&DailyLog>,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.insert_check_in(ci)?;
        if let Some(log) = daily_log_update {
            self.update_daily_log(log)?;
        }
        self.insert_journal_entry(journal)
    }

    fn start_plan_item_composite(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_plan_item(item)?;
        self.insert_task_checkpoint(checkpoint)?;
        self.insert_journal_entry(journal)
    }

    fn done_plan_item_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_plan_item(item)?;
        self.insert_journal_entry(journal)
    }

    fn move_to_waiting_composite(
        &self,
        item: &PlanItem,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_plan_item(item)?;
        self.insert_journal_entry(journal)
    }

    fn complete_checklist_composite(
        &self,
        run: &ChecklistRun,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_checklist_run(run)?;
        self.insert_journal_entry(journal)
    }

    fn insert_plan_item_with_checkpoint(
        &self,
        item: &PlanItem,
        checkpoint: &TaskCheckpoint,
    ) -> Result<(), Self::Error> {
        self.insert_plan_item(item)?;
        self.insert_task_checkpoint(checkpoint)
    }

    fn answer_checkpoint_composite(
        &self,
        checkpoint: &TaskCheckpoint,
        next_checkpoint: Option<&TaskCheckpoint>,
        journal: &JournalEntry,
    ) -> Result<(), Self::Error> {
        self.update_task_checkpoint(checkpoint)?;
        if let Some(next_cp) = next_checkpoint {
            self.insert_task_checkpoint(next_cp)?;
        }
        self.insert_journal_entry(journal)
    }

    // ── ImportRun (Noop) ──
    fn insert_import_run(&self, _run: &ImportRun) -> Result<(), Self::Error> {
        Ok(())
    }
    fn list_import_runs(&self) -> Result<Vec<ImportRun>, Self::Error> {
        Ok(vec![])
    }

    // ── Slug-based lookups (Noop) ──
    fn get_checklist_template_by_slug(
        &self,
        _slug: &str,
    ) -> Result<Option<ChecklistTemplate>, Self::Error> {
        Ok(None)
    }
    fn upsert_checklist_template(&self, _template: &ChecklistTemplate) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_context_document_by_source_slug(
        &self,
        _slug: &str,
    ) -> Result<Option<ContextDocument>, Self::Error> {
        Ok(None)
    }
    fn upsert_context_document(&self, _doc: &ContextDocument) -> Result<(), Self::Error> {
        Ok(())
    }
    fn bundle_apply_composite(
        &self,
        _bundle: &AdiyutantBundle,
        _mode: ImportMode,
        _import_run: &ImportRun,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    // ── Phase 8.4B: Noop stubs ──
    fn insert_project(&self, _p: &Project) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_project(&self, _id: Id<Project>) -> Result<Option<Project>, Self::Error> {
        Ok(None)
    }
    fn get_project_by_slug(&self, _slug: &str) -> Result<Option<Project>, Self::Error> {
        Ok(None)
    }
    fn list_projects(&self) -> Result<Vec<Project>, Self::Error> {
        Ok(vec![])
    }
    fn update_project(&self, _p: &Project) -> Result<(), Self::Error> {
        Ok(())
    }
    fn insert_roadmap(&self, _r: &Roadmap) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_roadmap(&self, _id: Id<Roadmap>) -> Result<Option<Roadmap>, Self::Error> {
        Ok(None)
    }
    fn get_roadmap_by_project_and_slug(
        &self,
        _project_id: Id<Project>,
        _slug: &str,
    ) -> Result<Option<Roadmap>, Self::Error> {
        Ok(None)
    }
    fn list_roadmaps(&self) -> Result<Vec<Roadmap>, Self::Error> {
        Ok(vec![])
    }
    fn list_roadmaps_by_project(
        &self,
        _project_id: Id<Project>,
    ) -> Result<Vec<Roadmap>, Self::Error> {
        Ok(vec![])
    }
    fn update_roadmap(&self, _r: &Roadmap) -> Result<(), Self::Error> {
        Ok(())
    }
    fn insert_roadmap_phase(&self, _p: &RoadmapPhase) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_roadmap_phase(
        &self,
        _id: Id<RoadmapPhase>,
    ) -> Result<Option<RoadmapPhase>, Self::Error> {
        Ok(None)
    }
    fn list_roadmap_phases(
        &self,
        _roadmap_id: Id<Roadmap>,
    ) -> Result<Vec<RoadmapPhase>, Self::Error> {
        Ok(vec![])
    }
    fn list_all_roadmap_phases(&self) -> Result<Vec<RoadmapPhase>, Self::Error> {
        Ok(vec![])
    }
    fn update_roadmap_phase(&self, _p: &RoadmapPhase) -> Result<(), Self::Error> {
        Ok(())
    }
    fn insert_roadmap_item(&self, _i: &RoadmapItem) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_roadmap_item(&self, _id: Id<RoadmapItem>) -> Result<Option<RoadmapItem>, Self::Error> {
        Ok(None)
    }
    fn list_roadmap_items(
        &self,
        _phase_id: Id<RoadmapPhase>,
    ) -> Result<Vec<RoadmapItem>, Self::Error> {
        Ok(vec![])
    }
    fn list_all_roadmap_items(&self) -> Result<Vec<RoadmapItem>, Self::Error> {
        Ok(vec![])
    }
    fn update_roadmap_item(&self, _i: &RoadmapItem) -> Result<(), Self::Error> {
        Ok(())
    }
    fn insert_roadmap_plan_link(&self, _link: &RoadmapPlanLink) -> Result<(), Self::Error> {
        Ok(())
    }
    fn get_roadmap_plan_link(
        &self,
        _id: Id<RoadmapPlanLink>,
    ) -> Result<Option<RoadmapPlanLink>, Self::Error> {
        Ok(None)
    }
    fn list_roadmap_plan_links_by_roadmap_item(
        &self,
        _roadmap_item_id: Id<RoadmapItem>,
    ) -> Result<Vec<RoadmapPlanLink>, Self::Error> {
        Ok(vec![])
    }
    fn list_roadmap_plan_links_by_plan_item(
        &self,
        _plan_item_id: Id<PlanItem>,
    ) -> Result<Vec<RoadmapPlanLink>, Self::Error> {
        Ok(vec![])
    }
    fn find_roadmap_plan_link(
        &self,
        _roadmap_item_id: Id<RoadmapItem>,
        _plan_item_id: Id<PlanItem>,
    ) -> Result<Option<RoadmapPlanLink>, Self::Error> {
        Ok(None)
    }
    fn take_roadmap_item_composite(
        &self,
        _input: TakeRoadmapItemInput,
    ) -> Result<TakeRoadmapItemResult, Self::Error> {
        Ok(TakeRoadmapItemResult {
            plan_item_id: _input.plan_item.id,
            link_id: _input.link.id,
            plan_item_reused: false,
            link_reused: false,
        })
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
    use adiyutant_core::model::journal_entry::JournalEntryType;
    use adiyutant_core::model::task_checkpoint::{
        CheckpointKind, CheckpointResponse, CheckpointStatus,
    };
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

    // ── 17. PlanItem: insert_and_get_plan_item ──
    use adiyutant_core::model::checklist_run::ChecklistRun;
    use adiyutant_core::model::checklist_template::{
        ChecklistItem, ChecklistItemKind, ChecklistTemplate,
    };
    use adiyutant_core::model::day_plan::{PlanItem, PlanItemStatus};
    use adiyutant_core::model::journal_entry::JournalEntry;
    use adiyutant_core::model::task_checkpoint::TaskCheckpoint;

    fn make_plan_item(daily_log_id: Id<DailyLog>) -> PlanItem {
        PlanItem::new(daily_log_id, "Test task".into())
    }

    fn make_checklist_template() -> ChecklistTemplate {
        let mut t = ChecklistTemplate::new("Morning Routine".into(), "morning".into());
        t.items.push(ChecklistItem::new(
            t.id,
            "Did you sleep well?".into(),
            ChecklistItemKind::Checkbox,
            1,
        ));
        t
    }

    fn make_checklist_run(
        template_id: Id<ChecklistTemplate>,
        daily_log_id: Id<DailyLog>,
    ) -> ChecklistRun {
        ChecklistRun::new(template_id, daily_log_id)
    }

    fn make_journal_entry(daily_log_id: Id<DailyLog>) -> JournalEntry {
        JournalEntry::new(daily_log_id, JournalEntryType::Note, "test entry".into())
    }

    #[test]
    fn insert_and_get_plan_item() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let item = make_plan_item(log.id);
        store.insert_plan_item(&item).unwrap();
        let loaded = store.get_plan_item(item.id).unwrap().unwrap();
        assert_eq!(loaded.title, "Test task");
    }

    #[test]
    fn list_plan_items_by_log() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        store.insert_plan_item(&make_plan_item(log.id)).unwrap();
        store.insert_plan_item(&make_plan_item(log.id)).unwrap();
        assert_eq!(store.list_plan_items_by_log(log.id).unwrap().len(), 2);
    }

    #[test]
    fn list_plan_items_by_status() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let mut item = make_plan_item(log.id);
        item.status = PlanItemStatus::Waiting;
        store.insert_plan_item(&item).unwrap();
        let items = store
            .list_plan_items_by_status(PlanItemStatus::Waiting)
            .unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn update_plan_item() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let mut item = make_plan_item(log.id);
        store.insert_plan_item(&item).unwrap();
        item.status = PlanItemStatus::Started;
        store.update_plan_item(&item).unwrap();
        assert_eq!(
            store.get_plan_item(item.id).unwrap().unwrap().status,
            PlanItemStatus::Started
        );
    }

    #[test]
    fn list_plan_items_due_for_review() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let mut item = make_plan_item(log.id);
        item.status = PlanItemStatus::Waiting;
        item.waiting_review_at = Some(NaiveDate::from_ymd_opt(2026, 6, 9).unwrap());
        store.insert_plan_item(&item).unwrap();
        let items = store
            .list_plan_items_due_for_review(NaiveDate::from_ymd_opt(2026, 6, 9).unwrap())
            .unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn delete_plan_item() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let item = make_plan_item(log.id);
        store.insert_plan_item(&item).unwrap();
        store.delete_plan_item(item.id).unwrap();
        assert!(store.get_plan_item(item.id).unwrap().is_none());
    }

    #[test]
    fn insert_and_get_task_checkpoint() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let item = make_plan_item(log.id);
        store.insert_plan_item(&item).unwrap();
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        store.insert_task_checkpoint(&cp).unwrap();
        let loaded = store.get_task_checkpoint(cp.id).unwrap().unwrap();
        assert_eq!(
            loaded.status,
            adiyutant_core::model::task_checkpoint::CheckpointStatus::Pending
        );
    }

    #[test]
    fn list_checkpoints_by_plan_item() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let item = make_plan_item(log.id);
        store.insert_plan_item(&item).unwrap();
        store
            .insert_task_checkpoint(&TaskCheckpoint::new(item.id, CheckpointKind::StartCheck))
            .unwrap();
        store
            .insert_task_checkpoint(&TaskCheckpoint::new(item.id, CheckpointKind::FinishCheck))
            .unwrap();
        let cps = store.list_checkpoints_by_plan_item(item.id).unwrap();
        assert_eq!(cps.len(), 2);
    }

    #[test]
    fn update_task_checkpoint() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let item = make_plan_item(log.id);
        store.insert_plan_item(&item).unwrap();
        let mut cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        store.insert_task_checkpoint(&cp).unwrap();
        cp.response = Some(adiyutant_core::model::task_checkpoint::CheckpointResponse::Done);
        store.update_task_checkpoint(&cp).unwrap();
    }

    #[test]
    fn delete_task_checkpoint() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let item = make_plan_item(log.id);
        store.insert_plan_item(&item).unwrap();
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::StartCheck);
        store.insert_task_checkpoint(&cp).unwrap();
        store.delete_task_checkpoint(cp.id).unwrap();
        assert!(store.get_task_checkpoint(cp.id).unwrap().is_none());
    }

    #[test]
    fn insert_and_get_checklist_template() {
        let store = setup();
        let t = make_checklist_template();
        store.insert_checklist_template(&t).unwrap();
        let loaded = store.get_checklist_template(t.id).unwrap().unwrap();
        assert_eq!(loaded.title, "Morning Routine");
    }

    #[test]
    fn list_checklist_templates() {
        let store = setup();
        store
            .insert_checklist_template(&make_checklist_template())
            .unwrap();
        store
            .insert_checklist_template(&ChecklistTemplate::new(
                "Evening Wind-Down".into(),
                "evening".into(),
            ))
            .unwrap();
        assert_eq!(store.list_checklist_templates().unwrap().len(), 2);
    }

    #[test]
    fn list_checklist_templates_by_category() {
        let store = setup();
        store
            .insert_checklist_template(&make_checklist_template())
            .unwrap();
        let items = store
            .list_checklist_templates_by_category("morning")
            .unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn update_checklist_template() {
        let store = setup();
        let mut t = make_checklist_template();
        store.insert_checklist_template(&t).unwrap();
        t.title = "Updated".into();
        store.update_checklist_template(&t).unwrap();
        assert_eq!(
            store.get_checklist_template(t.id).unwrap().unwrap().title,
            "Updated"
        );
    }

    #[test]
    fn delete_checklist_template() {
        let store = setup();
        let t = make_checklist_template();
        store.insert_checklist_template(&t).unwrap();
        store.delete_checklist_template(t.id).unwrap();
        assert!(store.get_checklist_template(t.id).unwrap().is_none());
    }

    #[test]
    fn insert_and_get_checklist_run() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let t = make_checklist_template();
        store.insert_checklist_template(&t).unwrap();
        let run = make_checklist_run(t.id, log.id);
        store.insert_checklist_run(&run).unwrap();
        let loaded = store.get_checklist_run(run.id).unwrap().unwrap();
        assert!(loaded.completed_at.is_none());
    }

    #[test]
    fn list_checklist_runs_by_log() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let t = make_checklist_template();
        store.insert_checklist_template(&t).unwrap();
        store
            .insert_checklist_run(&make_checklist_run(t.id, log.id))
            .unwrap();
        assert_eq!(store.list_checklist_runs_by_log(log.id).unwrap().len(), 1);
    }

    #[test]
    fn update_checklist_run() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let t = make_checklist_template();
        store.insert_checklist_template(&t).unwrap();
        let mut run = make_checklist_run(t.id, log.id);
        store.insert_checklist_run(&run).unwrap();
        run.completed_at = Some(adiyutant_core::datetime::AdiyutantDateTime::now());
        store.update_checklist_run(&run).unwrap();
    }

    #[test]
    fn delete_checklist_run() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let t = make_checklist_template();
        store.insert_checklist_template(&t).unwrap();
        let run = make_checklist_run(t.id, log.id);
        store.insert_checklist_run(&run).unwrap();
        store.delete_checklist_run(run.id).unwrap();
        assert!(store.get_checklist_run(run.id).unwrap().is_none());
    }

    #[test]
    fn insert_and_get_journal_entry() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let entry = make_journal_entry(log.id);
        store.insert_journal_entry(&entry).unwrap();
        let loaded = store.get_journal_entry(entry.id).unwrap().unwrap();
        assert_eq!(loaded.summary, "test entry");
    }

    #[test]
    fn list_journal_entries_by_log() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        store
            .insert_journal_entry(&make_journal_entry(log.id))
            .unwrap();
        store
            .insert_journal_entry(&JournalEntry::new(
                log.id,
                JournalEntryType::TaskDone,
                "done".into(),
            ))
            .unwrap();
        assert_eq!(store.list_journal_entries_by_log(log.id).unwrap().len(), 2);
    }

    #[test]
    fn update_journal_entry() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let mut entry = make_journal_entry(log.id);
        store.insert_journal_entry(&entry).unwrap();
        entry.summary = "updated".into();
        store.update_journal_entry(&entry).unwrap();
        assert_eq!(
            store.get_journal_entry(entry.id).unwrap().unwrap().summary,
            "updated"
        );
    }

    #[test]
    fn delete_journal_entry() {
        let store = setup();
        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();
        let entry = make_journal_entry(log.id);
        store.insert_journal_entry(&entry).unwrap();
        store.delete_journal_entry(entry.id).unwrap();
        assert!(store.get_journal_entry(entry.id).unwrap().is_none());
    }

    #[test]
    fn migrate_v01_to_v02_preserves_data() {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute_batch("
            CREATE TABLE daily_logs (
                id TEXT PRIMARY KEY, date TEXT NOT NULL, mode TEXT,
                sleep_score INTEGER, energy INTEGER, mood INTEGER,
                raw_notes TEXT, ai_summary TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE check_ins (
                id TEXT PRIMARY KEY, daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
                check_in_type TEXT NOT NULL, raw_input TEXT NOT NULL, structured_data TEXT,
                agent_response TEXT, created_at TEXT NOT NULL
            );
            CREATE TABLE habits (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT, frequency TEXT,
                target TEXT, is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE habit_events (
                id TEXT PRIMARY KEY, habit_id TEXT NOT NULL REFERENCES habits(id),
                date_time TEXT NOT NULL, status TEXT NOT NULL, level TEXT NOT NULL, comment TEXT
            );
            CREATE TABLE reminders (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, message TEXT,
                schedule_rule TEXT NOT NULL, next_fire_at TEXT, enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE alarms (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, time TEXT NOT NULL, repeat_rule TEXT,
                enabled INTEGER NOT NULL DEFAULT 1, platform_binding_id TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE timers (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, duration_seconds INTEGER NOT NULL,
                mode TEXT NOT NULL, created_at TEXT NOT NULL
            );
            CREATE TABLE context_documents (
                id TEXT PRIMARY KEY, doc_type TEXT NOT NULL, title TEXT NOT NULL,
                content_markdown TEXT NOT NULL, version INTEGER NOT NULL DEFAULT 1,
                is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE plans (
                id TEXT PRIMARY KEY, daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
                title TEXT NOT NULL, items_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE action_proposals (
                id TEXT PRIMARY KEY, source TEXT NOT NULL, proposal_type TEXT NOT NULL,
                payload_json TEXT NOT NULL, status TEXT NOT NULL,
                created_at TEXT NOT NULL, applied_at TEXT
            );
        ").unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO daily_logs (id, date, mode, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["log-1", "2026-06-09", "normal", &now, &now],
        ).unwrap();

        run_migration(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM daily_logs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        conn.query_row("SELECT COUNT(*) FROM plan_items", [], |_| Ok(()))
            .expect("plan_items table should exist");
        conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |_| Ok(()))
            .expect("journal_entries table should exist");
        conn.query_row("SELECT COUNT(*) FROM checklist_templates", [], |_| Ok(()))
            .expect("checklist_templates table should exist");

        let version_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_versions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version_count, 4);
    }

    #[test]
    fn transaction_rollback_on_error() {
        let store = SqliteStore::new_in_memory().unwrap();
        store.migrate().unwrap();

        let log = make_daily_log("2026-06-09");
        store.insert_daily_log(&log).unwrap();

        let result: Result<(), CoreError> = store.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO check_ins (id, daily_log_id, check_in_type, raw_input, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["ci-1", uuid_to_string(log.id), "\"Morning\"", "test", &chrono::Utc::now().to_rfc3339()],
            ).map_err(|e| CoreError::Storage(e.to_string()))?;

            Err(CoreError::InvalidInput("forced rollback".into()))
        });

        assert!(result.is_err());

        let ci_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM check_ins WHERE id = 'ci-1'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        assert_eq!(
            ci_count, 0,
            "Transaction rollback should have removed the inserted row"
        );
    }

    // ── Composite: insert_plan_item_with_checkpoint ──

    #[test]
    fn composite_insert_plan_item_with_checkpoint_creates_both() {
        let store = setup();
        let log = make_daily_log("2026-06-19");
        store.insert_daily_log(&log).unwrap();
        let item = PlanItem::new(log.id, "Test item".into());
        let cp = TaskCheckpoint::new(item.id, CheckpointKind::StartCheck);

        store.insert_plan_item_with_checkpoint(&item, &cp).unwrap();

        let saved_item = store.get_plan_item(item.id).unwrap().unwrap();
        assert_eq!(saved_item.title, "Test item");

        let checkpoints = store.list_checkpoints_by_plan_item(item.id).unwrap();
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].kind, CheckpointKind::StartCheck);
    }

    #[test]
    fn composite_insert_plan_item_rollback_on_checkpoint_failure() {
        let store = setup();
        let log = make_daily_log("2026-06-19");
        store.insert_daily_log(&log).unwrap();
        let item = PlanItem::new(log.id, "Rollback item".into());

        // Create a checkpoint with a non-existent plan_item_id — will fail FK constraint
        let bad_cp = TaskCheckpoint::new(
            Id::<PlanItem>::new(), // random UUID, not in DB
            CheckpointKind::StartCheck,
        );

        let result = store.insert_plan_item_with_checkpoint(&item, &bad_cp);
        assert!(result.is_err(), "expected FK constraint error");

        // PlanItem should NOT have been saved (transaction rolled back)
        let saved = store.get_plan_item(item.id).unwrap();
        assert!(saved.is_none(), "plan_item should be rolled back");
    }

    // ── Composite: answer_checkpoint_composite ──

    #[test]
    fn composite_answer_checkpoint_with_next_checkpoint() {
        let store = setup();
        let log = make_daily_log("2026-06-19");
        store.insert_daily_log(&log).unwrap();
        let item = PlanItem::new(log.id, "Checkpoint test".into());
        store.insert_plan_item(&item).unwrap();

        let mut cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        cp.status = CheckpointStatus::Shown;
        store.insert_task_checkpoint(&cp).unwrap();
        let cp_id = cp.id;

        // Mutate cp in memory as answer_checkpoint would
        cp.status = CheckpointStatus::Answered;
        cp.response = Some(CheckpointResponse::Done);
        cp.answered_at = Some(adiyutant_core::datetime::AdiyutantDateTime::from_utc(
            chrono::Utc::now(),
        ));

        let next_cp = TaskCheckpoint::new(item.id, CheckpointKind::FinishCheck);
        let journal = JournalEntry::new(log.id, JournalEntryType::NudgeAnswered, "test".into());

        store
            .answer_checkpoint_composite(&cp, Some(&next_cp), &journal)
            .unwrap();

        // Checkpoint updated
        let updated = store.get_task_checkpoint(cp_id).unwrap().unwrap();
        assert_eq!(updated.status, CheckpointStatus::Answered);

        // Next checkpoint created
        let checkpoints = store.list_checkpoints_by_plan_item(item.id).unwrap();
        let finish: Vec<_> = checkpoints
            .iter()
            .filter(|c| c.kind == CheckpointKind::FinishCheck)
            .collect();
        assert_eq!(finish.len(), 1);

        // Journal entry created
        let entries = store.list_journal_entries_by_log(log.id).unwrap();
        assert!(!entries.is_empty());
    }

    #[test]
    fn composite_answer_checkpoint_rollback_on_journal_failure() {
        let store = setup();
        let log = make_daily_log("2026-06-19");
        store.insert_daily_log(&log).unwrap();
        let item = PlanItem::new(log.id, "Rollback cp test".into());
        store.insert_plan_item(&item).unwrap();

        let mut cp = TaskCheckpoint::new(item.id, CheckpointKind::ProgressCheck);
        cp.status = CheckpointStatus::Shown;
        store.insert_task_checkpoint(&cp).unwrap();
        let cp_id = cp.id;
        let original_status = cp.status;

        // Mutate cp
        cp.status = CheckpointStatus::Answered;
        cp.response = Some(CheckpointResponse::Done);
        cp.answered_at = Some(adiyutant_core::datetime::AdiyutantDateTime::from_utc(
            chrono::Utc::now(),
        ));

        // Create a journal entry with a type that will cause a constraint error
        // by using a non-existent daily_log_id
        let bad_journal = JournalEntry::new(
            Id::<DailyLog>::new(), // random UUID not in DB
            JournalEntryType::NudgeAnswered,
            "test".into(),
        );

        let result = store.answer_checkpoint_composite(&cp, None, &bad_journal);
        assert!(result.is_err(), "expected FK constraint error");

        // Checkpoint should NOT have been updated (transaction rolled back)
        let saved = store.get_task_checkpoint(cp_id).unwrap().unwrap();
        assert_eq!(
            saved.status, original_status,
            "checkpoint should not be updated after rollback"
        );
    }

    #[test]
    fn composite_answer_checkpoint_without_next_checkpoint() {
        let store = setup();
        let log = make_daily_log("2026-06-19");
        store.insert_daily_log(&log).unwrap();
        let item = PlanItem::new(log.id, "No next cp".into());
        store.insert_plan_item(&item).unwrap();

        let mut cp = TaskCheckpoint::new(item.id, CheckpointKind::FinishCheck);
        cp.status = CheckpointStatus::Shown;
        store.insert_task_checkpoint(&cp).unwrap();
        let cp_id = cp.id;

        cp.status = CheckpointStatus::Answered;
        cp.response = Some(CheckpointResponse::Done);
        cp.answered_at = Some(adiyutant_core::datetime::AdiyutantDateTime::from_utc(
            chrono::Utc::now(),
        ));

        let journal = JournalEntry::new(
            log.id,
            JournalEntryType::NudgeAnswered,
            "test no next".into(),
        );

        store
            .answer_checkpoint_composite(&cp, None, &journal)
            .unwrap();

        // Checkpoint updated
        let updated = store.get_task_checkpoint(cp_id).unwrap().unwrap();
        assert_eq!(updated.status, CheckpointStatus::Answered);

        // No additional checkpoints beyond the original
        let checkpoints = store.list_checkpoints_by_plan_item(item.id).unwrap();
        assert_eq!(checkpoints.len(), 1);

        // Journal entry created
        let entries = store.list_journal_entries_by_log(log.id).unwrap();
        assert!(!entries.is_empty());
    }

    // ── Bundle apply composite — success and rollback ──

    fn make_test_bundle() -> AdiyutantBundle {
        use adiyutant_core::bundle::bundle_models::{
            ContextDocEntry, LifeCoreContextDoc, LifeCoreProfile, LifeCoreSection,
        };
        use adiyutant_core::bundle::manifest::Manifest;
        use std::collections::HashMap;

        AdiyutantBundle {
            manifest: Manifest {
                schema_version: "adiyutant.bundle.v1".into(),
                bundle_id: "test-bundle".into(),
                title: "Test Bundle".into(),
                created_at: "2026-06-23T10:00:00Z".into(),
                capabilities: HashMap::new(),
                sections: {
                    let mut m = HashMap::new();
                    m.insert("life_core".into(), "life-core.yaml".into());
                    m
                },
            },
            life_core: Some(LifeCoreSection {
                profile: Some(LifeCoreProfile {
                    name: Some("Test".into()),
                    bio: None,
                    values: vec![],
                    goals: vec![],
                }),
                context_documents: vec![
                    LifeCoreContextDoc {
                        slug: Some("doc-1".into()),
                        title: "Document 1".into(),
                        doc_type: "life_core".into(),
                        content: "# Doc 1\n\nContent.".into(),
                    },
                    LifeCoreContextDoc {
                        slug: Some("doc-2".into()),
                        title: "Document 2".into(),
                        doc_type: "life_core".into(),
                        content: "# Doc 2\n\nMore content.".into(),
                    },
                ],
            }),
            routines: None,
            rules: None,
            checklists: None,
            planning: None,
            context_docs: vec![ContextDocEntry {
                slug: Some("personal".into()),
                title: "Personal".into(),
                content: "# Personal\n\nPersonal context.".into(),
            }],
            projects: vec![],
            roadmaps: vec![],
            unknown_files: vec![],
        }
    }

    fn make_import_run() -> ImportRun {
        use adiyutant_core::datetime::AdiyutantDateTime;
        ImportRun {
            id: Id::new(),
            bundle_id: "test-bundle".into(),
            bundle_title: "Test Bundle".into(),
            schema_version: "adiyutant.bundle.v1".into(),
            mode: "append".into(),
            status: "applied".into(),
            started_at: AdiyutantDateTime::now(),
            finished_at: AdiyutantDateTime::now(),
            summary_json: "{}".into(),
        }
    }

    #[test]
    fn bundle_apply_success_writes_data() {
        let store = SqliteStore::new_in_memory().unwrap();
        store.migrate().unwrap();

        let bundle = make_test_bundle();
        let mode = ImportMode::Append;
        let run = make_import_run();

        assert_eq!(store.list_import_runs().unwrap().len(), 0);
        assert_eq!(store.list_context_documents().unwrap().len(), 0);

        store.bundle_apply_composite(&bundle, mode, &run).unwrap();

        // All data should be committed
        assert_eq!(store.list_import_runs().unwrap().len(), 1);
        assert!(
            !store.list_context_documents().unwrap().is_empty(),
            "context documents should be written"
        );
    }

    fn make_test_bundle_with_checklist() -> AdiyutantBundle {
        use adiyutant_core::bundle::bundle_models::{
            BundleChecklistItem, BundleChecklistTemplate, ChecklistsSection,
        };
        let mut bundle = make_test_bundle();
        bundle.checklists = Some(ChecklistsSection {
            templates: vec![BundleChecklistTemplate {
                slug: "test-checklist".into(),
                title: "Test Checklist".into(),
                category: "morning".into(),
                items: vec![BundleChecklistItem {
                    slug: Some("item-1".into()),
                    question: "Did you test?".into(),
                    kind: "Checkbox".into(),
                    options: vec![],
                    order: 1,
                }],
            }],
        });
        bundle
    }

    #[test]
    fn bundle_apply_rollback_mid_apply() {
        let store = SqliteStore::new_in_memory().unwrap();
        store.migrate().unwrap();

        // Install a trigger that causes checklist INSERT to FAIL.
        // This simulates a mid-apply error AFTER life_core context documents
        // have been written (they are processed first in apply_bundle_inner)
        // but BEFORE the apply completes. The transaction ROLLBACK must undo
        // the already-written life_core context docs.
        store
            .conn
            .execute_batch(
                "CREATE TEMP TRIGGER fail_checklist_insert
                 BEFORE INSERT ON checklist_templates
                 BEGIN
                     SELECT RAISE(FAIL, 'forced: checklist template insert rejected');
                 END;",
            )
            .unwrap();

        // Bundle with life_core (processed FIRST) AND checklists (processed AFTER)
        let bundle = make_test_bundle_with_checklist();
        let mode = ImportMode::Append;
        let run = make_import_run();

        // Verify clean state
        assert!(store.list_context_documents().unwrap().is_empty());
        assert!(store.list_checklist_templates().unwrap().is_empty());
        assert_eq!(store.list_import_runs().unwrap().len(), 0);

        // Apply should fail — checklist trigger fires AFTER life_core writes
        let result = store.bundle_apply_composite(&bundle, mode, &run);
        assert!(
            result.is_err(),
            "checklist trigger should cause apply to fail mid-way"
        );

        // Verify TRANSACTIONAL ROLLBACK: life_core context docs written inside
        // the transaction must NOT persist after the checklist insert fails.
        assert!(
            store.list_context_documents().unwrap().is_empty(),
            "life_core context docs written before the failure must be rolled back"
        );

        // Verify import_run was also rolled back
        assert_eq!(
            store.list_import_runs().unwrap().len(),
            0,
            "import_run must be rolled back with the transaction"
        );

        // No partial checklist writes either
        assert!(
            store.list_checklist_templates().unwrap().is_empty(),
            "no checklist templates should exist after rolled-back apply"
        );
    }

    // ── Project/Roadmap integration tests ──

    fn make_project() -> Project {
        Project::new("test-project".into(), "Test Project".into())
    }

    fn make_roadmap(project_id: Id<Project>) -> Roadmap {
        Roadmap::new("test-roadmap".into(), project_id, "Test Roadmap".into())
    }

    fn make_roadmap_phase(roadmap_id: Id<Roadmap>) -> RoadmapPhase {
        RoadmapPhase::new("phase-1".into(), roadmap_id, "Phase 1".into(), 0)
    }

    fn make_roadmap_item(phase_id: Id<RoadmapPhase>) -> RoadmapItem {
        RoadmapItem::new("item-1".into(), phase_id, "Item 1".into())
    }

    fn make_daily_log_for_take(store: &SqliteStore, date_str: &str) -> DailyLog {
        let log = make_daily_log(date_str);
        store.insert_daily_log(&log).unwrap();
        log
    }

    #[test]
    fn insert_and_get_project() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let retrieved = store.get_project(p.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, p.id);
    }

    #[test]
    fn insert_project_duplicate_slug_fails() {
        let store = setup();
        let p1 = make_project();
        store.insert_project(&p1).unwrap();
        let mut p2 = make_project();
        p2.id = Id::new();
        let result = store.insert_project(&p2);
        assert!(result.is_err(), "duplicate slug should fail");
    }

    #[test]
    fn get_project_by_slug() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let retrieved = store.get_project_by_slug("test-project").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, p.id);

        let not_found = store.get_project_by_slug("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn list_projects() {
        let store = setup();
        let p1 = make_project();
        let p2 = Project::new("second-project".into(), "Second".into());
        store.insert_project(&p1).unwrap();
        store.insert_project(&p2).unwrap();
        let projects = store.list_projects().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn update_project() {
        let store = setup();
        let mut p = make_project();
        store.insert_project(&p).unwrap();
        p.title = "Updated Title".into();
        store.update_project(&p).unwrap();
        let retrieved = store.get_project(p.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
    }

    #[test]
    fn insert_and_get_roadmap() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let retrieved = store.get_roadmap(r.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, r.id);
    }

    #[test]
    fn list_roadmaps_by_project() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r1 = make_roadmap(p.id);
        let r2 = Roadmap::new("roadmap-2".into(), p.id, "Roadmap 2".into());
        store.insert_roadmap(&r1).unwrap();
        store.insert_roadmap(&r2).unwrap();

        let roadmaps = store.list_roadmaps_by_project(p.id).unwrap();
        assert_eq!(roadmaps.len(), 2);

        let all = store.list_roadmaps().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn get_roadmap_by_project_and_slug() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let retrieved = store
            .get_roadmap_by_project_and_slug(p.id, "test-roadmap")
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, r.id);
    }

    #[test]
    fn update_roadmap() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let mut r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        r.title = "Updated Roadmap".into();
        store.update_roadmap(&r).unwrap();
        let retrieved = store.get_roadmap(r.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Roadmap");
    }

    #[test]
    fn insert_and_get_roadmap_phase() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let retrieved = store.get_roadmap_phase(phase.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, phase.id);
    }

    #[test]
    fn list_roadmap_phases() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let ph1 = make_roadmap_phase(r.id);
        let ph2 = RoadmapPhase::new("phase-2".into(), r.id, "Phase 2".into(), 1);
        store.insert_roadmap_phase(&ph1).unwrap();
        store.insert_roadmap_phase(&ph2).unwrap();
        assert_eq!(store.list_roadmap_phases(r.id).unwrap().len(), 2);
        assert_eq!(store.list_all_roadmap_phases().unwrap().len(), 2);
    }

    #[test]
    fn update_roadmap_phase() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let mut phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        phase.title = "Updated Phase".into();
        store.update_roadmap_phase(&phase).unwrap();
        let retrieved = store.get_roadmap_phase(phase.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Phase");
    }

    #[test]
    fn insert_and_get_roadmap_item() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let mut item = make_roadmap_item(phase.id);
        item.acceptance_criteria = vec!["must work".into()];
        item.depends_on = vec!["dep-1".into()];
        item.links = vec!["link-1".into()];
        store.insert_roadmap_item(&item).unwrap();
        let retrieved = store.get_roadmap_item(item.id).unwrap().unwrap();
        assert_eq!(retrieved.acceptance_criteria, vec!["must work"]);
        assert_eq!(retrieved.depends_on, vec!["dep-1"]);
        assert_eq!(retrieved.links, vec!["link-1"]);
    }

    #[test]
    fn list_roadmap_items() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        store
            .insert_roadmap_item(&make_roadmap_item(phase.id))
            .unwrap();
        store
            .insert_roadmap_item(&RoadmapItem::new(
                "item-2".into(),
                phase.id,
                "Item 2".into(),
            ))
            .unwrap();
        assert_eq!(store.list_roadmap_items(phase.id).unwrap().len(), 2);
        assert_eq!(store.list_all_roadmap_items().unwrap().len(), 2);
    }

    #[test]
    fn update_roadmap_item() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let mut item = make_roadmap_item(phase.id);
        store.insert_roadmap_item(&item).unwrap();
        item.title = "Updated Item".into();
        store.update_roadmap_item(&item).unwrap();
        let retrieved = store.get_roadmap_item(item.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Item");
    }

    #[test]
    fn insert_and_get_roadmap_plan_link() {
        let store = setup();
        let log = make_daily_log_for_take(&store, "2026-06-20");
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let item = make_roadmap_item(phase.id);
        store.insert_roadmap_item(&item).unwrap();
        let plan_item = PlanItem::new(log.id, "Test plan".into());
        store.insert_plan_item(&plan_item).unwrap();
        let link = RoadmapPlanLink::new(item.id, plan_item.id);
        store.insert_roadmap_plan_link(&link).unwrap();
        let retrieved = store.get_roadmap_plan_link(link.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, link.id);
    }

    #[test]
    fn list_roadmap_plan_links() {
        let store = setup();
        let log = make_daily_log_for_take(&store, "2026-06-20");
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let item = make_roadmap_item(phase.id);
        store.insert_roadmap_item(&item).unwrap();
        let pi1 = PlanItem::new(log.id, "PI 1".into());
        let pi2 = PlanItem::new(log.id, "PI 2".into());
        store.insert_plan_item(&pi1).unwrap();
        store.insert_plan_item(&pi2).unwrap();
        let link1 = RoadmapPlanLink::new(item.id, pi1.id);
        let link2 = RoadmapPlanLink::new(item.id, pi2.id);
        store.insert_roadmap_plan_link(&link1).unwrap();
        store.insert_roadmap_plan_link(&link2).unwrap();

        let by_ri = store
            .list_roadmap_plan_links_by_roadmap_item(item.id)
            .unwrap();
        assert_eq!(by_ri.len(), 2);

        let by_pi = store.list_roadmap_plan_links_by_plan_item(pi1.id).unwrap();
        assert_eq!(by_pi.len(), 1);
    }

    #[test]
    fn find_roadmap_plan_link() {
        let store = setup();
        let log = make_daily_log_for_take(&store, "2026-06-20");
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let item = make_roadmap_item(phase.id);
        store.insert_roadmap_item(&item).unwrap();
        let plan_item = PlanItem::new(log.id, "Test".into());
        store.insert_plan_item(&plan_item).unwrap();
        let link = RoadmapPlanLink::new(item.id, plan_item.id);
        store.insert_roadmap_plan_link(&link).unwrap();

        let found = store.find_roadmap_plan_link(item.id, plan_item.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, link.id);

        let not_found = store.find_roadmap_plan_link(item.id, Id::new()).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn take_item_composite_creates_all_records() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let mut item = make_roadmap_item(phase.id);
        item.status = adiyutant_core::model::roadmap::RoadmapItemStatus::Planned;
        store.insert_roadmap_item(&item).unwrap();

        let daily_log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 20).unwrap());
        let plan_item = PlanItem::new(daily_log.id, "Taken item".into());
        let checkpoint = TaskCheckpoint::new(plan_item.id, CheckpointKind::StartCheck);
        let link = RoadmapPlanLink::new(item.id, plan_item.id);

        let input = TakeRoadmapItemInput {
            roadmap_item_id: item.id,
            target_date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            daily_log_to_create: Some(daily_log),
            plan_item,
            initial_checkpoint: checkpoint,
            link,
            activate_item: true,
        };

        let result = store.take_roadmap_item_composite(input).unwrap();
        assert!(!result.plan_item_reused);
        assert!(!result.link_reused);

        let saved_item = store.get_plan_item(result.plan_item_id).unwrap();
        assert!(saved_item.is_some(), "plan_item should exist");
        let saved_link = store.get_roadmap_plan_link(result.link_id).unwrap();
        assert!(saved_link.is_some(), "link should exist");
        let checkpoints = store
            .list_checkpoints_by_plan_item(result.plan_item_id)
            .unwrap();
        assert_eq!(checkpoints.len(), 1, "checkpoint should exist");

        let saved_ri = store.get_roadmap_item(item.id).unwrap().unwrap();
        assert_eq!(
            saved_ri.status,
            adiyutant_core::model::roadmap::RoadmapItemStatus::Active,
            "item should be activated"
        );
    }

    #[test]
    fn take_item_composite_idempotent() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let item = make_roadmap_item(phase.id);
        store.insert_roadmap_item(&item).unwrap();

        let daily_log = DailyLog::new(NaiveDate::from_ymd_opt(2026, 6, 20).unwrap());
        let plan_item = PlanItem::new(daily_log.id, "Idempotent item".into());
        let checkpoint = TaskCheckpoint::new(plan_item.id, CheckpointKind::StartCheck);
        let link = RoadmapPlanLink::new(item.id, plan_item.id);

        let input = TakeRoadmapItemInput {
            roadmap_item_id: item.id,
            target_date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            daily_log_to_create: Some(daily_log),
            plan_item,
            initial_checkpoint: checkpoint,
            link,
            activate_item: false,
        };

        let result1 = store.take_roadmap_item_composite(input).unwrap();
        assert!(!result1.plan_item_reused);
        let plan_item_id = result1.plan_item_id;

        let link2 = RoadmapPlanLink::new(item.id, plan_item_id);
        let input2 = TakeRoadmapItemInput {
            roadmap_item_id: item.id,
            target_date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            daily_log_to_create: None,
            plan_item: store.get_plan_item(plan_item_id).unwrap().unwrap(),
            initial_checkpoint: TaskCheckpoint::new(plan_item_id, CheckpointKind::ProgressCheck),
            link: link2,
            activate_item: false,
        };

        let result2 = store.take_roadmap_item_composite(input2).unwrap();
        assert!(result2.plan_item_reused, "should reuse plan_item");
        assert!(result2.link_reused, "should reuse link");
    }

    #[test]
    fn take_item_activates_item() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let phase = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&phase).unwrap();
        let mut item = make_roadmap_item(phase.id);
        item.status = adiyutant_core::model::roadmap::RoadmapItemStatus::Planned;
        store.insert_roadmap_item(&item).unwrap();

        let log = make_daily_log("2026-06-20");
        store.insert_daily_log(&log).unwrap();
        let plan_item = PlanItem::new(log.id, "Activate test".into());
        let checkpoint = TaskCheckpoint::new(plan_item.id, CheckpointKind::StartCheck);
        let link = RoadmapPlanLink::new(item.id, plan_item.id);

        let input = TakeRoadmapItemInput {
            roadmap_item_id: item.id,
            target_date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            daily_log_to_create: None,
            plan_item,
            initial_checkpoint: checkpoint,
            link,
            activate_item: true,
        };

        store.take_roadmap_item_composite(input).unwrap();
        let saved = store.get_roadmap_item(item.id).unwrap().unwrap();
        assert_eq!(
            saved.status,
            adiyutant_core::model::roadmap::RoadmapItemStatus::Active,
            "item should transition from planned to active"
        );
    }

    // ── FK enforcement (referential integrity) ──

    #[test]
    fn foreign_key_enforcement_rejects_orphan_roadmap() {
        let store = setup();
        let fake_project_id = Id::<Project>::new();
        let r = Roadmap::new("orphan-roadmap".into(), fake_project_id, "Orphan".into());
        let result = store.insert_roadmap(&r);
        assert!(
            result.is_err(),
            "FK should reject roadmap with non-existent project_id"
        );
    }

    #[test]
    fn foreign_key_enforcement_rejects_orphan_phase() {
        let store = setup();
        let fake_roadmap_id = Id::<Roadmap>::new();
        let ph = RoadmapPhase::new("orphan-phase".into(), fake_roadmap_id, "Orphan".into(), 0);
        let result = store.insert_roadmap_phase(&ph);
        assert!(
            result.is_err(),
            "FK should reject phase with non-existent roadmap_id"
        );
    }

    #[test]
    fn foreign_key_enforcement_rejects_orphan_item() {
        let store = setup();
        let fake_phase_id = Id::<RoadmapPhase>::new();
        let item = RoadmapItem::new("orphan-item".into(), fake_phase_id, "Orphan".into());
        let result = store.insert_roadmap_item(&item);
        assert!(
            result.is_err(),
            "FK should reject item with non-existent phase_id"
        );
    }

    #[test]
    fn foreign_key_enforcement_rejects_orphan_link() {
        let store = setup();
        let fake_item_id = Id::<RoadmapItem>::new();
        let fake_pi_id = Id::<PlanItem>::new();
        let link = RoadmapPlanLink::new(fake_item_id, fake_pi_id);
        let result = store.insert_roadmap_plan_link(&link);
        assert!(
            result.is_err(),
            "FK should reject link with non-existent roadmap_item_id or plan_item_id"
        );
    }

    #[test]
    fn take_item_composite_rollback_on_fk_violation() {
        let store = setup();
        let p = make_project();
        store.insert_project(&p).unwrap();
        let r = make_roadmap(p.id);
        store.insert_roadmap(&r).unwrap();
        let ph = make_roadmap_phase(r.id);
        store.insert_roadmap_phase(&ph).unwrap();
        let item = make_roadmap_item(ph.id);
        store.insert_roadmap_item(&item).unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 6, 25).unwrap();

        // PlanItem with non-existent daily_log_id, no daily_log_to_create
        let bad_pi = PlanItem::new(Id::<DailyLog>::new(), "Bad".into());
        let cp = TaskCheckpoint::new(bad_pi.id, CheckpointKind::StartCheck);
        let link = RoadmapPlanLink::new(item.id, bad_pi.id);

        let input = TakeRoadmapItemInput {
            roadmap_item_id: item.id,
            target_date: date,
            daily_log_to_create: None,
            plan_item: bad_pi.clone(),
            initial_checkpoint: cp,
            link,
            activate_item: false,
        };

        let result = store.take_roadmap_item_composite(input);
        assert!(
            result.is_err(),
            "should fail: FK violation on plan_items.daily_log_id"
        );

        // Verify no partial data persisted after rollback
        assert!(
            store.get_plan_item(bad_pi.id).unwrap().is_none(),
            "no plan_item should exist after rollback"
        );
        assert!(
            store
                .list_roadmap_plan_links_by_roadmap_item(item.id)
                .unwrap()
                .is_empty(),
            "no links should exist after rollback"
        );
    }
}
