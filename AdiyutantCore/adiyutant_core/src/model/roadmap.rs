use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::day_plan::PlanItem;
use crate::model::project::Project;
use serde::{Deserialize, Serialize};

/// Roadmap horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoadmapHorizon {
    Week,
    Month,
    Quarter,
    Custom,
}

impl RoadmapHorizon {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoadmapHorizon::Week => "week",
            RoadmapHorizon::Month => "month",
            RoadmapHorizon::Quarter => "quarter",
            RoadmapHorizon::Custom => "custom",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "week" => Some(RoadmapHorizon::Week),
            "month" => Some(RoadmapHorizon::Month),
            "quarter" => Some(RoadmapHorizon::Quarter),
            "custom" => Some(RoadmapHorizon::Custom),
            _ => None,
        }
    }
}

/// Phase status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhaseStatus {
    Planned,
    Active,
    Done,
    Skipped,
}

impl PhaseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PhaseStatus::Planned => "planned",
            PhaseStatus::Active => "active",
            PhaseStatus::Done => "done",
            PhaseStatus::Skipped => "skipped",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "planned" => Some(PhaseStatus::Planned),
            "active" => Some(PhaseStatus::Active),
            "done" => Some(PhaseStatus::Done),
            "skipped" => Some(PhaseStatus::Skipped),
            _ => None,
        }
    }
}

/// Roadmap item status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoadmapItemStatus {
    Planned,
    Active,
    Blocked,
    Done,
    Skipped,
}

impl RoadmapItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoadmapItemStatus::Planned => "planned",
            RoadmapItemStatus::Active => "active",
            RoadmapItemStatus::Blocked => "blocked",
            RoadmapItemStatus::Done => "done",
            RoadmapItemStatus::Skipped => "skipped",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "planned" => Some(RoadmapItemStatus::Planned),
            "active" => Some(RoadmapItemStatus::Active),
            "blocked" => Some(RoadmapItemStatus::Blocked),
            "done" => Some(RoadmapItemStatus::Done),
            "skipped" => Some(RoadmapItemStatus::Skipped),
            _ => None,
        }
    }
}

/// A roadmap attached to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roadmap {
    pub id: Id<Roadmap>,
    pub slug: String,
    pub project_id: Id<Project>,
    pub title: String,
    pub description: Option<String>,
    pub horizon: RoadmapHorizon,
    pub status: crate::model::project::ProjectStatus,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl Roadmap {
    pub fn new(slug: String, project_id: Id<Project>, title: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            slug,
            project_id,
            title,
            description: None,
            horizon: RoadmapHorizon::Month,
            status: crate::model::project::ProjectStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A phase within a roadmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapPhase {
    pub id: Id<RoadmapPhase>,
    pub slug: String,
    pub roadmap_id: Id<Roadmap>,
    pub title: String,
    pub order_index: u32,
    pub status: PhaseStatus,
}

impl RoadmapPhase {
    pub fn new(slug: String, roadmap_id: Id<Roadmap>, title: String, order_index: u32) -> Self {
        Self {
            id: Id::new(),
            slug,
            roadmap_id,
            title,
            order_index,
            status: PhaseStatus::Planned,
        }
    }
}

/// A roadmap item within a phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItem {
    pub id: Id<RoadmapItem>,
    pub slug: String,
    pub phase_id: Id<RoadmapPhase>,
    pub title: String,
    pub description: Option<String>,
    pub status: RoadmapItemStatus,
    pub priority: u8,
    pub acceptance_criteria: Vec<String>,
    pub depends_on: Vec<String>,
    pub links: Vec<String>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

impl RoadmapItem {
    pub fn new(slug: String, phase_id: Id<RoadmapPhase>, title: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            slug,
            phase_id,
            title,
            description: None,
            status: RoadmapItemStatus::Planned,
            priority: 5,
            acceptance_criteria: Vec::new(),
            depends_on: Vec::new(),
            links: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Link between a roadmap item and a day-plan item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapPlanLink {
    pub id: Id<RoadmapPlanLink>,
    pub roadmap_item_id: Id<RoadmapItem>,
    pub plan_item_id: Id<PlanItem>,
    pub link_type: String,
    pub created_at: AdiyutantDateTime,
}

impl RoadmapPlanLink {
    pub fn new(roadmap_item_id: Id<RoadmapItem>, plan_item_id: Id<PlanItem>) -> Self {
        Self {
            id: Id::new(),
            roadmap_item_id,
            plan_item_id,
            link_type: "taken_into_day".into(),
            created_at: AdiyutantDateTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roadmap_new_creates_with_slug_and_title() {
        let id = Id::<Project>::new();
        let r = Roadmap::new("test-roadmap".into(), id, "Test Roadmap".into());
        assert_eq!(r.slug, "test-roadmap");
        assert_eq!(r.project_id, id);
        assert_eq!(r.horizon, RoadmapHorizon::Month);
    }

    #[test]
    fn roadmap_horizon_round_trip() {
        for (h, str) in &[
            (RoadmapHorizon::Week, "week"),
            (RoadmapHorizon::Month, "month"),
            (RoadmapHorizon::Quarter, "quarter"),
            (RoadmapHorizon::Custom, "custom"),
        ] {
            assert_eq!(h.as_str(), *str);
            assert_eq!(RoadmapHorizon::from_str(str), Some(*h));
        }
        assert_eq!(RoadmapHorizon::from_str("unknown"), None);
    }

    #[test]
    fn phase_status_round_trip() {
        for (s, str) in &[
            (PhaseStatus::Planned, "planned"),
            (PhaseStatus::Active, "active"),
            (PhaseStatus::Done, "done"),
            (PhaseStatus::Skipped, "skipped"),
        ] {
            assert_eq!(s.as_str(), *str);
            assert_eq!(PhaseStatus::from_str(str), Some(*s));
        }
    }

    #[test]
    fn roadmap_item_status_round_trip() {
        for (s, str) in &[
            (RoadmapItemStatus::Planned, "planned"),
            (RoadmapItemStatus::Active, "active"),
            (RoadmapItemStatus::Blocked, "blocked"),
            (RoadmapItemStatus::Done, "done"),
            (RoadmapItemStatus::Skipped, "skipped"),
        ] {
            assert_eq!(s.as_str(), *str);
            assert_eq!(RoadmapItemStatus::from_str(str), Some(*s));
        }
    }

    #[test]
    fn roadmap_phase_new_creates() {
        let rm_id = Id::<Roadmap>::new();
        let p = RoadmapPhase::new("phase-1".into(), rm_id, "Phase 1".into(), 0);
        assert_eq!(p.slug, "phase-1");
        assert_eq!(p.roadmap_id, rm_id);
        assert_eq!(p.status, PhaseStatus::Planned);
    }

    #[test]
    fn roadmap_item_new_creates() {
        let ph_id = Id::<RoadmapPhase>::new();
        let i = RoadmapItem::new("item-1".into(), ph_id, "Item 1".into());
        assert_eq!(i.slug, "item-1");
        assert_eq!(i.status, RoadmapItemStatus::Planned);
    }

    #[test]
    fn roadmap_plan_link_new_creates() {
        let ri_id = Id::<RoadmapItem>::new();
        let pi_id = Id::<PlanItem>::new();
        let link = RoadmapPlanLink::new(ri_id, pi_id);
        assert_eq!(link.roadmap_item_id, ri_id);
        assert_eq!(link.plan_item_id, pi_id);
        assert_eq!(link.link_type, "taken_into_day");
    }
}
