use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextDocumentType {
    Core,
    Goals,
    Rules,
    Routine,
    Health,
    Work,
    Custom,
    // LifeCoreProfile type tags (same ContentDocument model, new doc_type discriminators)
    LifeCore,
    RecoveryProtocol,
    Tone,
    PlanningPreferences,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextDocument {
    pub id: Id<Self>,
    pub doc_type: ContextDocumentType,
    pub title: String,
    /// Stable source slug used by portable bundle import for merge semantics.
    /// Optional for backward compatibility with pre-8.4A rows.
    pub source_slug: Option<String>,
    pub content_markdown: String,
    pub version: u32,
    pub is_active: bool,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl ContextDocument {
    pub fn new(doc_type: ContextDocumentType, title: String, content_markdown: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            doc_type,
            title,
            source_slug: None,
            content_markdown,
            version: 1,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_document() {
        let doc = ContextDocument::new(
            ContextDocumentType::Core,
            "My Core".into(),
            "# Values\n\nBe kind.".into(),
        );
        assert_eq!(doc.doc_type, ContextDocumentType::Core);
        assert_eq!(doc.title, "My Core");
        assert_eq!(doc.content_markdown, "# Values\n\nBe kind.");
    }

    #[test]
    fn version_defaults_to_one() {
        let doc =
            ContextDocument::new(ContextDocumentType::Goals, "Goals".into(), "content".into());
        assert_eq!(doc.version, 1);
    }

    #[test]
    fn document_active_by_default() {
        let doc =
            ContextDocument::new(ContextDocumentType::Rules, "Rules".into(), "content".into());
        assert!(doc.is_active);
    }

    #[test]
    fn serde_round_trip_with_markdown() {
        let doc = ContextDocument::new(
            ContextDocumentType::Custom,
            "Doc".into(),
            "# Hello\n\nThis is **markdown**.".into(),
        );
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: ContextDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc.id, deserialized.id);
        assert_eq!(doc.content_markdown, deserialized.content_markdown);
    }
}
