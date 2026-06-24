use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

/// Project status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectStatus {
    Active,
    Paused,
    Done,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Paused => "paused",
            ProjectStatus::Done => "done",
            ProjectStatus::Archived => "archived",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(ProjectStatus::Active),
            "paused" => Some(ProjectStatus::Paused),
            "done" => Some(ProjectStatus::Done),
            "archived" => Some(ProjectStatus::Archived),
            _ => None,
        }
    }
}

/// A project (Adiyutant first-class entity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Id<Project>,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub priority: u8,
    pub why: Option<String>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl Project {
    pub fn new(slug: String, title: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            slug,
            title,
            description: None,
            status: ProjectStatus::Active,
            priority: 5,
            why: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_new_creates_with_slug_and_title() {
        let p = Project::new("test-project".into(), "Test Project".into());
        assert_eq!(p.slug, "test-project");
        assert_eq!(p.title, "Test Project");
        assert_eq!(p.status, ProjectStatus::Active);
        assert_eq!(p.priority, 5);
    }

    #[test]
    fn project_status_round_trip() {
        for (s, str) in &[
            (ProjectStatus::Active, "active"),
            (ProjectStatus::Paused, "paused"),
            (ProjectStatus::Done, "done"),
            (ProjectStatus::Archived, "archived"),
        ] {
            assert_eq!(s.as_str(), *str);
            assert_eq!(ProjectStatus::from_str(str), Some(*s));
        }
        assert_eq!(ProjectStatus::from_str("unknown"), None);
    }

    #[test]
    fn project_serde_round_trip() {
        let p = Project::new("test".into(), "Test".into());
        let json = serde_json::to_string(&p).unwrap();
        let back: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, back.id);
        assert_eq!(p.slug, back.slug);
    }
}
