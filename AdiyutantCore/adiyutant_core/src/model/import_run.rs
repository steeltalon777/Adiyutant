use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
// Serialize/Deserialize not needed yet — ImportRun is not a serde DTO.

/// A record of a completed (or failed) bundle import apply operation.
///
/// Only `apply` persists these records. `validate` and `preview` are ephemeral.
#[derive(Debug, Clone)]
pub struct ImportRun {
    pub id: Id<ImportRun>,
    pub bundle_id: String,
    pub bundle_title: String,
    pub schema_version: String,
    pub mode: String,   // "append" or "merge"
    pub status: String, // "applied", "failed"
    pub started_at: AdiyutantDateTime,
    pub finished_at: AdiyutantDateTime,
    /// JSON summary of actions taken.
    pub summary_json: String,
}
