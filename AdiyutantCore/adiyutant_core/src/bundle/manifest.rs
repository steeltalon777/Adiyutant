//! `BundleManifest` — authoritative bundle metadata.
//!
//! `manifest.yaml` is required and is the only authoritative map of
//! which sections exist in a bundle. Unknown top-level files produce
//! warnings; unknown optional manifest fields produce warnings too.

use serde::{Deserialize, Serialize};

/// Top-level fields required by Adiyutant Bundle v1.
pub const REQUIRED_SCHEMA_VERSION: &str = "adiyutant.bundle.v1";

/// Capability advert: in Phase 8.4A both `projects` and `roadmaps` are
/// `preview-only` and must NOT be applied.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BundleCapabilities {
    #[serde(default = "default_preview_only")]
    pub projects: String,
    #[serde(default = "default_preview_only")]
    pub roadmaps: String,
}

fn default_preview_only() -> String {
    "preview-only".to_string()
}

/// Section manifest — each field points at the relative file or
/// directory inside the bundle.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleSections {
    pub life_core: Option<String>,
    pub routines: Option<String>,
    pub rules: Option<String>,
    pub checklists: Option<String>,
    pub planning: Option<String>,
    pub context_dir: Option<String>,
    pub projects_dir: Option<String>,
    pub roadmaps_dir: Option<String>,
}

/// Authoritative bundle manifest parsed from `manifest.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub schema_version: String,
    pub bundle_id: String,
    pub title: String,
    pub created_at: String,
    #[serde(default)]
    pub capabilities: BundleCapabilities,
    #[serde(default)]
    pub sections: BundleSections,
}

// Stage 0 stub — Unit B will add strict parsing + unknown-field detection.
#[doc(hidden)]
pub fn _stage0_stub() {}
