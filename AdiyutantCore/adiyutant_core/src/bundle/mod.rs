//! Adiyutant Bundle v1 — portable import/export contract.
//!
//! This module is owned by Core. It owns:
//!
//! * Manifest schema and validation.
//! * Section schema for `life-core.yaml`, `routines.yaml`, `rules.yaml`,
//!   `checklists.yaml`, `planning.yaml`, `context/*.md`.
//! * `projects/*` and `roadmaps/*` parse + validate + preview-only; never
//!   applied in Phase 8.4A (those land in 8.4B).
//! * Zip security: path traversal, symlinks, hardlinks, size limits,
//!   extension allow-list, file/dir allow-list.
//! * Preview/apply planner — one planner is used by both operations to
//!   guarantee parity.
//! * Report DTOs (`BundleValidationReportDto`, `BundlePreviewDto`,
//!   `BundleApplyReportDto`, `BundleExportReportDto`).
//!
//! Bundle rules are stored as human context only and are not wired into
//! `LocalRuleGateway`.

pub mod dto;
pub mod manifest;
pub mod parser;
pub mod planner;
pub mod report;
pub mod sections;
pub mod source;
pub mod validation;
pub mod zip_security;

pub use source::BundleSource;

use crate::error::CoreError;
use crate::error::CoreResult;
use crate::model::import_run::{ImportMode, ImportStatus};
use crate::model::{ChecklistTemplate, ContextDocument, ImportRun};

// ──────────────────────────────────────────────
// Public bundle representation
// ──────────────────────────────────────────────

/// Canonical import/export container for Adiyutant Bundle v1.
///
/// Both directory bundles and `.adiyutant.zip` bundles parse into one
/// `AdiyutantBundle`. Supported sections are kept as parsed typed data;
/// unsupported `projects/*` and `roadmaps/*` sections are kept as raw
/// `serde_yaml::Value` trees so that the planner can report them as
/// preview-only without losing information.
#[derive(Debug, Clone)]
pub struct AdiyutantBundle {
    pub manifest: crate::bundle::manifest::BundleManifest,
    pub life_core: Vec<ContextDocument>,
    pub routines: Option<crate::bundle::sections::RoutinesSection>,
    pub rules: Option<crate::bundle::sections::RulesSection>,
    pub checklists: Vec<ChecklistTemplate>,
    pub planning: Option<crate::bundle::sections::PlanningSection>,
    pub context_markdown: Vec<crate::bundle::sections::ContextMarkdown>,
    pub projects: Vec<crate::bundle::sections::ProjectFile>,
    pub roadmaps: Vec<crate::bundle::sections::RoadmapFile>,
    /// All manifest-declared extra top-level files we noticed but do not
    /// understand. Used to surface "unknown top-level file" warnings.
    pub extra_files: Vec<crate::bundle::sections::ExtraFile>,
}

impl Default for AdiyutantBundle {
    fn default() -> Self {
        Self {
            manifest: crate::bundle::manifest::BundleManifest {
                schema_version: String::new(),
                bundle_id: String::new(),
                title: String::new(),
                created_at: String::new(),
                capabilities: Default::default(),
                sections: Default::default(),
            },
            life_core: Vec::new(),
            routines: None,
            rules: None,
            checklists: Vec::new(),
            planning: None,
            context_markdown: Vec::new(),
            projects: Vec::new(),
            roadmaps: Vec::new(),
            extra_files: Vec::new(),
        }
    }
}

// ──────────────────────────────────────────────
// Bundle apply operation (passed to Store for atomic write)
// ──────────────────────────────────────────────

/// A single supported-section write that the planner has resolved.
///
/// `ReplaceExistingId` is set when the planner chose to update an
/// existing row in `merge` mode; otherwise the operation is a pure
/// insert.
#[derive(Debug, Clone)]
pub struct BundleApplyOp {
    pub context_documents: Vec<ContextDocument>,
    pub checklist_templates: Vec<ChecklistTemplate>,
    pub import_run: ImportRun,
}

// ──────────────────────────────────────────────
// Bundle report status (stable string contract)
// ──────────────────────────────────────────────

/// Coarse-grained status for bundle operations. Stable string contract
/// surfaced to the CLI and consumers; see also the report DTO enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleStatus {
    Valid,
    ValidWithWarnings,
    Invalid,
    Applied,
    Failed,
}

impl BundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BundleStatus::Valid => "valid",
            BundleStatus::ValidWithWarnings => "valid_with_warnings",
            BundleStatus::Invalid => "invalid",
            BundleStatus::Applied => "applied",
            BundleStatus::Failed => "failed",
        }
    }
}

// ──────────────────────────────────────────────
// Convenience: convert from ImportMode/Status to bundle equivalents
// ──────────────────────────────────────────────

pub fn import_mode_str(mode: ImportMode) -> &'static str {
    mode.as_str()
}

pub fn import_status_str(status: ImportStatus) -> &'static str {
    status.as_str()
}

pub fn _bundle_result<T>(v: T) -> CoreResult<T> {
    Ok(v)
}

pub fn _err(msg: impl Into<String>) -> CoreError {
    CoreError::InvalidInput(msg.into())
}
