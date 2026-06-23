//! Bundle section models.
//!
//! Each supported section has a typed shape; preview-only
//! `projects/*` and `roadmaps/*` keep their raw value tree so the
//! planner can validate and report them without dropping information.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// ── Routines section ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RoutinesSection {
    #[serde(default)]
    pub routines: Vec<RoutineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutineEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub steps: Vec<String>,
}

// ── Rules section ─────────────────────────────────────────────────
//
// Bundle rules are stored as human context only. They are NOT mapped
// into `LocalRuleGateway`. Future Rule Engine v2 may interpret them.

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RulesSection {
    #[serde(default)]
    pub rules: Vec<RuleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
}

// ── Planning section ──────────────────────────────────────────────
//
// Phase 8.4A accepts only preferences, templates, and candidate
// hints. Concrete dated `DayPlan` imports are explicitly forbidden.

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PlanningSection {
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub daily_capacity: Option<PlanningDailyCapacity>,
    #[serde(default)]
    pub templates: Vec<PlanningTemplate>,
    #[serde(default)]
    pub candidate_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PlanningDailyCapacity {
    #[serde(default)]
    pub deep_tasks_max: Option<u32>,
    #[serde(default)]
    pub light_tasks_max: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanningTemplate {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub blocks: Vec<PlanningBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanningBlock {
    pub kind: String,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub duration_minutes: Option<u32>,
}

// ── Context markdown files (`context/*.md`) ──────────────────────

#[derive(Debug, Clone)]
pub struct ContextMarkdown {
    /// Relative path inside the bundle, e.g. `context/personal.md`.
    pub rel_path: String,
    /// Stable slug derived from the file name without extension.
    pub slug: String,
    /// Free-form title.
    pub title: String,
    /// Document type discriminator (best-effort from path or content).
    pub doc_type: String,
    /// Raw markdown body.
    pub content_markdown: String,
}

// ── Preview-only project / roadmap files ─────────────────────────
//
// These are parsed + validated + previewed only in Phase 8.4A. They
// are kept as raw JSON values to preserve the file's structure for
// future Phase 8.4B persistence.

#[derive(Debug, Clone)]
pub struct ProjectFile {
    pub rel_path: String,
    pub slug: String,
    pub raw: JsonValue,
}

#[derive(Debug, Clone)]
pub struct RoadmapFile {
    pub rel_path: String,
    pub slug: String,
    pub raw: JsonValue,
}

/// Any top-level file we noticed but do not understand. Reported as a
/// warning during validation.
#[derive(Debug, Clone)]
pub struct ExtraFile {
    pub rel_path: String,
    pub reason: String,
}

// Stage 0 stub — Unit B will add validation + dup detection.
#[doc(hidden)]
pub fn _stage0_stub() {}
