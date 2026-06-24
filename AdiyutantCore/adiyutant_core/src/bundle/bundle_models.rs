use serde::{Deserialize, Serialize};

use crate::store::Store;

/// Parsed data from a bundle's `life-core.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeCoreSection {
    #[serde(default)]
    pub profile: Option<LifeCoreProfile>,
    #[serde(default)]
    pub context_documents: Vec<LifeCoreContextDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeCoreProfile {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeCoreContextDoc {
    #[serde(default)]
    pub slug: Option<String>,
    pub title: String,
    pub doc_type: String,
    pub content: String,
}

/// Parsed data from a bundle's `routines.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutinesSection {
    #[serde(default)]
    pub routines: Vec<RoutineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineEntry {
    #[serde(default)]
    pub slug: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub steps: Vec<String>,
}

/// Parsed data from a bundle's `rules.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesSection {
    #[serde(default)]
    pub rules: Vec<RuleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    #[serde(default)]
    pub slug: Option<String>,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub category: Option<String>,
}

/// Parsed data from a bundle's `checklists.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistsSection {
    #[serde(default)]
    pub templates: Vec<BundleChecklistTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleChecklistTemplate {
    pub slug: String,
    pub title: String,
    pub category: String,
    #[serde(default)]
    pub items: Vec<BundleChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleChecklistItem {
    #[serde(default)]
    pub slug: Option<String>,
    pub question: String,
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub order: u32,
}

/// Parsed data from a bundle's `planning.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningSection {
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub daily_capacity: Option<DailyCapacity>,
    #[serde(default)]
    pub templates: Vec<PlanningTemplate>,
    #[serde(default)]
    pub candidate_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCapacity {
    pub deep_tasks_max: Option<u32>,
    pub light_tasks_max: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningTemplate {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub blocks: Vec<PlanningBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningBlock {
    pub kind: String,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub duration_minutes: Option<u32>,
}

/// Parsed data from `context/*.md` files — just the filename and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDocEntry {
    #[serde(default)]
    pub slug: Option<String>,
    pub title: String,
    pub content: String,
}

/// The unified internal representation of an Adiyutant Bundle.
///
/// After parsing, all source forms (directory, zip) produce this single type.
/// Validation, preview, apply, and export all work on `AdiyutantBundle`.
#[derive(Debug, Clone)]
pub struct AdiyutantBundle {
    pub manifest: crate::bundle::manifest::Manifest,
    pub life_core: Option<LifeCoreSection>,
    pub routines: Option<RoutinesSection>,
    pub rules: Option<RulesSection>,
    pub checklists: Option<ChecklistsSection>,
    pub planning: Option<PlanningSection>,
    pub context_docs: Vec<ContextDocEntry>,
    pub projects: Vec<BundleProject>,
    pub roadmaps: Vec<BundleRoadmap>,
    /// Raw file paths present in the bundle but not declared in manifest.
    pub unknown_files: Vec<String>,
}

/// Import mode for bundle apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    Append,
    Merge,
}

impl std::fmt::Display for ImportMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Append => write!(f, "append"),
            Self::Merge => write!(f, "merge"),
        }
    }
}

pub trait SlugLookup {
    fn slug_exists(&self, store: &dyn Store<Error = crate::error::CoreError>) -> bool;
}

impl SlugLookup for RoutineEntry {
    fn slug_exists(&self, store: &dyn Store<Error = crate::error::CoreError>) -> bool {
        let slug = self.slug.as_deref().unwrap_or(&self.title);
        store
            .get_context_document_by_source_slug(slug)
            .ok()
            .flatten()
            .is_some()
    }
}

impl SlugLookup for RuleEntry {
    fn slug_exists(&self, store: &dyn Store<Error = crate::error::CoreError>) -> bool {
        let slug = self.slug.as_deref().unwrap_or(&self.title);
        store
            .get_context_document_by_source_slug(slug)
            .ok()
            .flatten()
            .is_some()
    }
}

impl SlugLookup for BundleChecklistTemplate {
    fn slug_exists(&self, store: &dyn Store<Error = crate::error::CoreError>) -> bool {
        store
            .get_checklist_template_by_slug(&self.slug)
            .ok()
            .flatten()
            .is_some()
    }
}

// ── Phase 8.4B: Bundle types for projects/roadmaps (apply/export supported) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleProject {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_project_status_str")]
    pub status: String,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub why: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRoadmap {
    pub slug: String,
    pub project_slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_horizon_str")]
    pub horizon: String,
    #[serde(default = "default_project_status_str")]
    pub status: String,
    #[serde(default)]
    pub phases: Vec<BundleRoadmapPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRoadmapPhase {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub order_index: u32,
    #[serde(default = "default_phase_status_str")]
    pub status: String,
    #[serde(default)]
    pub items: Vec<BundleRoadmapItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRoadmapItem {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_item_status_str")]
    pub status: String,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub links: Vec<String>,
}

fn default_project_status_str() -> String {
    "active".to_string()
}
fn default_horizon_str() -> String {
    "month".to_string()
}
fn default_phase_status_str() -> String {
    "planned".to_string()
}
fn default_item_status_str() -> String {
    "planned".to_string()
}
fn default_priority() -> u8 {
    5
}

impl SlugLookup for ContextDocEntry {
    fn slug_exists(&self, store: &dyn Store<Error = crate::error::CoreError>) -> bool {
        let slug = self.slug.as_deref().unwrap_or(&self.title);
        store
            .get_context_document_by_source_slug(slug)
            .ok()
            .flatten()
            .is_some()
    }
}
