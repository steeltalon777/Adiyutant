use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

/// Mode of a bundle import run.
///
/// `Append` adds new entities and leaves existing ones untouched on slug
/// conflict. `Merge` updates existing entities by stable slug/id and adds
/// missing ones. `Replace` is intentionally not part of MVP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportMode {
    Append,
    Merge,
}

impl ImportMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportMode::Append => "append",
            ImportMode::Merge => "merge",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "append" => Some(ImportMode::Append),
            "merge" => Some(ImportMode::Merge),
            _ => None,
        }
    }
}

/// Final status of a bundle import run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    Applied,
    Failed,
}

impl ImportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportStatus::Applied => "applied",
            ImportStatus::Failed => "failed",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "applied" => Some(ImportStatus::Applied),
            "failed" => Some(ImportStatus::Failed),
            _ => None,
        }
    }
}

/// History record for one apply run of a portable bundle.
///
/// `summary_json` carries a serialised `BundleApplyReportDto` (or
/// `BundleValidationReportDto` on failure) so the report survives a
/// process restart.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportRun {
    pub id: Id<ImportRun>,
    pub bundle_id: String,
    pub bundle_title: String,
    pub schema_version: String,
    pub mode: ImportMode,
    pub status: ImportStatus,
    pub started_at: AdiyutantDateTime,
    pub finished_at: Option<AdiyutantDateTime>,
    pub summary_json: String,
}

impl ImportRun {
    pub fn new(
        bundle_id: String,
        bundle_title: String,
        schema_version: String,
        mode: ImportMode,
    ) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            bundle_id,
            bundle_title,
            schema_version,
            mode,
            status: ImportStatus::Applied,
            started_at: now,
            finished_at: None,
            summary_json: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_mode_round_trip() {
        for m in &[ImportMode::Append, ImportMode::Merge] {
            assert_eq!(ImportMode::from_str(m.as_str()), Some(*m));
        }
    }

    #[test]
    fn import_mode_from_str_invalid() {
        assert_eq!(ImportMode::from_str("replace"), None);
        assert_eq!(ImportMode::from_str(""), None);
    }

    #[test]
    fn import_status_round_trip() {
        for s in &[ImportStatus::Applied, ImportStatus::Failed] {
            assert_eq!(ImportStatus::from_str(s.as_str()), Some(*s));
        }
    }

    #[test]
    fn import_run_new_defaults() {
        let r = ImportRun::new(
            "bundle-1".into(),
            "Title".into(),
            "adiyutant.bundle.v1".into(),
            ImportMode::Append,
        );
        assert_eq!(r.status, ImportStatus::Applied);
        assert!(r.finished_at.is_none());
        assert!(r.summary_json.is_empty());
    }
}
