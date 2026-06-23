use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
// Secondary DTOs — Bundle/Import/Export reports
// Rules: flat, stable output models.
// All dates/times are ISO-8601 strings.
// ──────────────────────────────────────────────

/// Status of a bundle validation or apply operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BundleStatus {
    #[serde(rename = "valid")]
    Valid,
    #[serde(rename = "valid_with_warnings")]
    ValidWithWarnings,
    #[serde(rename = "invalid")]
    Invalid,
    #[serde(rename = "applied")]
    Applied,
    #[serde(rename = "failed")]
    Failed,
}

impl std::fmt::Display for BundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::ValidWithWarnings => write!(f, "valid_with_warnings"),
            Self::Invalid => write!(f, "invalid"),
            Self::Applied => write!(f, "applied"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// A single issue (error or warning) found during bundle validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleIssueDto {
    pub severity: String, // "error" or "warning"
    pub section: String,  // "manifest", "life_core", "checklists", "context", etc.
    pub code: String,     // e.g. "missing_field", "invalid_enum", "duplicate_slug", "unsafe_path"
    pub message: String,
    pub field: Option<String>,
}

/// Action kind for import plan / report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BundleActionKind {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "skip_existing")]
    SkipExisting,
    #[serde(rename = "ignored_preview_only")]
    IgnoredPreviewOnly,
    #[serde(rename = "conflict")]
    Conflict,
    #[serde(rename = "error")]
    Error,
}

impl std::fmt::Display for BundleActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::Update => write!(f, "update"),
            Self::SkipExisting => write!(f, "skip_existing"),
            Self::IgnoredPreviewOnly => write!(f, "ignored_preview_only"),
            Self::Conflict => write!(f, "conflict"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// A single action in the import plan / report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleActionDto {
    pub kind: BundleActionKind,
    pub section: String,
    pub entity_identifier: String, // slug or title
    pub message: String,
}

/// Per-section report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSectionReportDto {
    pub section: String,
    pub status: String, // "ok", "skipped", "error", "preview_only"
    pub entity_count: usize,
    pub actions: Vec<BundleActionDto>,
    pub issues: Vec<BundleIssueDto>,
}

/// Validation report (returned by validate and as part of preview/apply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleValidationReportDto {
    pub bundle_id: String,
    pub schema_version: String,
    pub title: String,
    pub status: BundleStatus,
    pub errors: Vec<BundleIssueDto>,
    pub warnings: Vec<BundleIssueDto>,
    pub sections: Vec<BundleSectionReportDto>,
}

/// Preview report (computed before apply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePreviewDto {
    pub bundle_id: String,
    pub schema_version: String,
    pub title: String,
    pub status: BundleStatus,
    pub errors: Vec<BundleIssueDto>,
    pub warnings: Vec<BundleIssueDto>,
    pub sections: Vec<BundleSectionReportDto>,
    pub actions: Vec<BundleActionDto>,
    pub mode: String, // "append" or "merge"
}

/// Apply report (returned after import).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleApplyReportDto {
    pub bundle_id: String,
    pub schema_version: String,
    pub title: String,
    pub status: BundleStatus,
    pub errors: Vec<BundleIssueDto>,
    pub warnings: Vec<BundleIssueDto>,
    pub sections: Vec<BundleSectionReportDto>,
    pub actions: Vec<BundleActionDto>,
    pub mode: String,
    pub import_run_id: String,
}

/// Export report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleExportReportDto {
    pub bundle_id: String,
    pub schema_version: String,
    pub title: String,
    pub status: BundleStatus,
    pub target_path: String,
    pub section_count: usize,
    pub errors: Vec<BundleIssueDto>,
    pub warnings: Vec<BundleIssueDto>,
}
