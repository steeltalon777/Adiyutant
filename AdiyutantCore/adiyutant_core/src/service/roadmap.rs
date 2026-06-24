use crate::dto::PlanItemDto;
use crate::dto::{
    RoadmapDetailDto, RoadmapDto, RoadmapItemDto, RoadmapPhaseDto, RoadmapPhaseWithItemsDto,
    RoadmapPlanLinkDto, RoadmapTakeItemResultDto,
};
use crate::error::{CoreError, CoreResult};
use crate::model::daily_log::DailyLog;
use crate::model::day_plan::{PlanItem, PlanningMode};
use crate::model::roadmap::{
    Roadmap, RoadmapItem, RoadmapItemStatus, RoadmapPhase, RoadmapPlanLink,
};
use crate::model::task_checkpoint::{CheckpointKind, TaskCheckpoint};
use crate::service::selectors::{parse_roadmap_item_selector, parse_roadmap_selector};
use crate::store::{Store, TakeRoadmapItemInput, TakeRoadmapItemResult};
use chrono::NaiveDate;

/// Map a Roadmap domain model to RoadmapDto.
pub fn roadmap_to_dto(r: &Roadmap) -> RoadmapDto {
    RoadmapDto {
        id: r.id.value().to_string(),
        slug: r.slug.clone(),
        project_slug: r.project_id.value().to_string(), // caller should fix if project slug available
        title: r.title.clone(),
        description: r.description.as_deref().unwrap_or("").to_string(),
        horizon: r.horizon.as_str().to_string(),
        status: r.status.as_str().to_string(),
        created_at: r.created_at.inner().to_rfc3339(),
        updated_at: r.updated_at.inner().to_rfc3339(),
    }
}

/// Map a RoadmapPhase to DTO.
pub fn phase_to_dto(ph: &RoadmapPhase, roadmap_slug: &str) -> RoadmapPhaseDto {
    RoadmapPhaseDto {
        id: ph.id.value().to_string(),
        slug: ph.slug.clone(),
        roadmap_slug: roadmap_slug.to_string(),
        title: ph.title.clone(),
        order_index: ph.order_index,
        status: ph.status.as_str().to_string(),
    }
}

/// Map a RoadmapItem to DTO.
pub fn item_to_dto(item: &RoadmapItem, phase_slug: &str) -> RoadmapItemDto {
    RoadmapItemDto {
        id: item.id.value().to_string(),
        slug: item.slug.clone(),
        phase_slug: phase_slug.to_string(),
        title: item.title.clone(),
        description: item.description.as_deref().unwrap_or("").to_string(),
        status: item.status.as_str().to_string(),
        priority: item.priority,
        acceptance_criteria: item.acceptance_criteria.clone(),
        depends_on: item.depends_on.clone(),
        links: item.links.clone(),
        created_at: item.created_at.inner().to_rfc3339(),
        updated_at: item.updated_at.inner().to_rfc3339(),
    }
}

/// Resolve a roadmap by selector. Returns error on ambiguity.
pub fn resolve_roadmap(
    store: &dyn Store<Error = CoreError>,
    selector: &str,
) -> CoreResult<(Roadmap, String)> {
    let parsed = parse_roadmap_selector(selector)?;
    match parsed {
        crate::service::selectors::RoadmapSelector::Unqualified { slug } => {
            // Try to find unambiguous match
            let all = store.list_roadmaps()?;
            let matches: Vec<&Roadmap> = all.iter().filter(|r| r.slug == slug).collect();
            match matches.len() {
                0 => Err(CoreError::NotFound(format!("roadmap '{slug}'"))),
                1 => {
                    // Get project slug
                    let project = store
                        .get_project(matches[0].project_id)?
                        .ok_or_else(|| CoreError::Internal("orphan roadmap".into()))?;
                    Ok((matches[0].clone(), project.slug))
                }
                _ => Err(CoreError::InvalidInput(format!(
                    "roadmap slug '{slug}' is ambiguous. Use qualified form: project-slug/roadmap-slug"
                ))),
            }
        }
        crate::service::selectors::RoadmapSelector::Qualified {
            project_slug,
            roadmap_slug,
        } => {
            let project = store
                .get_project_by_slug(&project_slug)?
                .ok_or_else(|| CoreError::NotFound(format!("project '{project_slug}'")))?;
            let roadmap = store
                .get_roadmap_by_project_and_slug(project.id, &roadmap_slug)?
                .ok_or_else(|| {
                    CoreError::NotFound(format!(
                        "roadmap '{roadmap_slug}' under project '{project_slug}'"
                    ))
                })?;
            Ok((roadmap, project_slug))
        }
    }
}

/// List roadmaps, optionally filtered by project slug.
pub fn list_roadmaps(
    store: &dyn Store<Error = CoreError>,
    project_slug: Option<&str>,
) -> CoreResult<Vec<RoadmapDto>> {
    let roadmaps = if let Some(slug) = project_slug {
        let project = store
            .get_project_by_slug(slug)?
            .ok_or_else(|| CoreError::NotFound(format!("project '{slug}'")))?;
        store.list_roadmaps_by_project(project.id)?
    } else {
        store.list_roadmaps()?
    };

    Ok(roadmaps
        .iter()
        .map(|r| {
            let mut dto = roadmap_to_dto(r);
            // Fix project_slug if we have it
            if let Ok(Some(p)) = store.get_project(r.project_id) {
                dto.project_slug = p.slug;
            }
            dto
        })
        .collect())
}

/// Get roadmap detail with phases and items.
pub fn get_roadmap(
    store: &dyn Store<Error = CoreError>,
    selector: &str,
) -> CoreResult<RoadmapDetailDto> {
    let (roadmap, project_slug) = resolve_roadmap(store, selector)?;
    let phases = store.list_roadmap_phases(roadmap.id)?;

    let mut phase_dtos = Vec::new();
    for ph in &phases {
        let items = store.list_roadmap_items(ph.id)?;
        let item_dtos: Vec<RoadmapItemDto> =
            items.iter().map(|i| item_to_dto(i, &ph.slug)).collect();
        phase_dtos.push(RoadmapPhaseWithItemsDto {
            phase: phase_to_dto(ph, &roadmap.slug),
            items: item_dtos,
        });
    }

    let mut dto = roadmap_to_dto(&roadmap);
    dto.project_slug = project_slug.clone();

    Ok(RoadmapDetailDto {
        roadmap: dto,
        project_slug,
        phases: phase_dtos,
    })
}

/// List roadmap items filtered by a roadmap or phase selector.
pub fn list_roadmap_items(
    store: &dyn Store<Error = CoreError>,
    selector: &str,
) -> CoreResult<Vec<RoadmapItemDto>> {
    let (roadmap, _project_slug) = resolve_roadmap(store, selector)?;
    let phases = store.list_roadmap_phases(roadmap.id)?;

    let mut all_items = Vec::new();
    for ph in &phases {
        let items = store.list_roadmap_items(ph.id)?;
        for item in &items {
            all_items.push(item_to_dto(item, &ph.slug));
        }
    }
    Ok(all_items)
}

/// Resolve a roadmap item by selector.
fn resolve_roadmap_item(
    store: &dyn Store<Error = CoreError>,
    selector: &str,
) -> CoreResult<RoadmapItem> {
    let parsed = parse_roadmap_item_selector(selector)?;
    match parsed {
        crate::service::selectors::RoadmapItemSelector::Unqualified { slug } => {
            let all = store.list_all_roadmap_items()?;
            let matches: Vec<&RoadmapItem> = all.iter().filter(|i| i.slug == slug).collect();
            match matches.len() {
                0 => Err(CoreError::NotFound(format!("roadmap item '{slug}'"))),
                1 => Ok(matches[0].clone()),
                _ => Err(CoreError::InvalidInput(format!(
                    "item slug '{slug}' is ambiguous. Use qualified form: phase-slug/item-slug or roadmap-slug/phase-slug/item-slug"
                ))),
            }
        }
        crate::service::selectors::RoadmapItemSelector::PhaseQualified {
            phase_slug,
            item_slug,
        } => {
            let all_phases = store.list_all_roadmap_phases()?;
            let phase_matches: Vec<&RoadmapPhase> =
                all_phases.iter().filter(|p| p.slug == phase_slug).collect();
            match phase_matches.len() {
                0 => Err(CoreError::NotFound(format!("phase '{phase_slug}'"))),
                1 => {
                    let items = store.list_roadmap_items(phase_matches[0].id)?;
                    items
                        .into_iter()
                        .find(|i| i.slug == item_slug)
                        .ok_or_else(|| {
                            CoreError::NotFound(format!(
                                "item '{item_slug}' in phase '{phase_slug}'"
                            ))
                        })
                }
                _ => Err(CoreError::InvalidInput(format!(
                    "phase slug '{phase_slug}' is ambiguous. Use fully qualified: roadmap-slug/{phase_slug}/{item_slug}"
                ))),
            }
        }
        crate::service::selectors::RoadmapItemSelector::FullyQualified {
            roadmap_slug,
            phase_slug,
            item_slug,
        } => {
            let all_roadmaps = store.list_roadmaps()?;
            let rm_matches: Vec<&Roadmap> = all_roadmaps
                .iter()
                .filter(|r| r.slug == roadmap_slug)
                .collect();
            match rm_matches.len() {
                0 => Err(CoreError::NotFound(format!("roadmap '{roadmap_slug}'"))),
                1 => {
                    let phases = store.list_roadmap_phases(rm_matches[0].id)?;
                    let ph = phases
                        .into_iter()
                        .find(|p| p.slug == phase_slug)
                        .ok_or_else(|| {
                            CoreError::NotFound(format!(
                                "phase '{phase_slug}' in roadmap '{roadmap_slug}'"
                            ))
                        })?;
                    let items = store.list_roadmap_items(ph.id)?;
                    items
                        .into_iter()
                        .find(|i| i.slug == item_slug)
                        .ok_or_else(|| {
                            CoreError::NotFound(format!(
                                "item '{item_slug}' in phase '{phase_slug}'"
                            ))
                        })
                }
                _ => Err(CoreError::InvalidInput(format!(
                    "roadmap slug '{roadmap_slug}' is ambiguous"
                ))),
            }
        }
    }
}

pub fn plan_item_to_dto(pi: &PlanItem) -> PlanItemDto {
    PlanItemDto {
        id: pi.id.value().to_string(),
        title: pi.title.clone(),
        description: pi.description.as_deref().unwrap_or("").to_string(),
        quadrant: pi.quadrant.as_str().to_string(),
        planned_start: pi.planned_start.map(|t| t.to_string()).unwrap_or_default(),
        planned_end: pi.planned_end.map(|t| t.to_string()).unwrap_or_default(),
        status: pi.status.as_str().to_string(),
        priority: pi.priority,
        source: pi.source.as_str().to_string(),
        created_at: pi.created_at.inner().to_rfc3339(),
        updated_at: pi.updated_at.inner().to_rfc3339(),
    }
}

/// Take a roadmap item into a day plan.
/// Idempotent: if a link for this roadmap item + date already exists, returns existing data.
pub fn take_roadmap_item(
    store: &dyn Store<Error = CoreError>,
    item_selector: &str,
    target_date: NaiveDate,
) -> CoreResult<RoadmapTakeItemResultDto> {
    let item = resolve_roadmap_item(store, item_selector)?;

    // Get or create daily log for target date
    let daily_log = store.get_daily_log_by_date(target_date)?;
    let (daily_log_to_create, daily_log_id) = if let Some(ref log) = daily_log {
        (None, log.id)
    } else {
        let new_log = DailyLog::new(target_date);
        let id = new_log.id;
        (Some(new_log), id)
    };

    // Idempotency check: if daily_log already existed, check if this roadmap item
    // is already linked to any plan item in this log
    if let Some(ref log) = daily_log {
        let plan_items = store.list_plan_items_by_log(log.id)?;
        for pi in &plan_items {
            if let Some(existing_link) = store.find_roadmap_plan_link(item.id, pi.id)? {
                let phase = store.get_roadmap_phase(item.phase_id)?;
                let phase_slug = phase.as_ref().map(|p| p.slug.as_str()).unwrap_or("");
                return Ok(RoadmapTakeItemResultDto {
                    roadmap_item: item_to_dto(&item, phase_slug),
                    plan_item: plan_item_to_dto(pi),
                    link: link_to_dto(&existing_link),
                    target_date: target_date.format("%Y-%m-%d").to_string(),
                });
            }
        }
    }

    // Create plan item
    let mut plan_item = PlanItem::new(daily_log_id, item.title.clone());
    plan_item.source = PlanningMode::Manual;
    plan_item.description = item.description.clone();

    // Create initial StartCheck checkpoint
    let checkpoint = TaskCheckpoint::new(plan_item.id, CheckpointKind::StartCheck);

    // Create link
    let link = RoadmapPlanLink::new(item.id, plan_item.id);

    // Determine whether to activate the item
    let activate = item.status == RoadmapItemStatus::Planned;

    let input = TakeRoadmapItemInput {
        roadmap_item_id: item.id,
        target_date,
        daily_log_to_create,
        plan_item,
        initial_checkpoint: checkpoint,
        link,
        activate_item: activate,
    };

    let result: TakeRoadmapItemResult = store.take_roadmap_item_composite(input)?;

    // Fetch created entities for the DTO response
    let created_plan_item = store.get_plan_item(result.plan_item_id)?.ok_or_else(|| {
        CoreError::Internal("plan_item not found after take_roadmap_item_composite".into())
    })?;
    let created_link = store
        .get_roadmap_plan_link(result.link_id)?
        .ok_or_else(|| {
            CoreError::Internal("link not found after take_roadmap_item_composite".into())
        })?;

    // Build item DTO (we need phase_slug)
    let phase = store.get_roadmap_phase(item.phase_id)?;
    let phase_slug = phase.as_ref().map(|p| p.slug.as_str()).unwrap_or("");

    Ok(RoadmapTakeItemResultDto {
        roadmap_item: item_to_dto(&item, phase_slug),
        plan_item: plan_item_to_dto(&created_plan_item),
        link: link_to_dto(&created_link),
        target_date: target_date.format("%Y-%m-%d").to_string(),
    })
}

fn link_to_dto(link: &RoadmapPlanLink) -> RoadmapPlanLinkDto {
    RoadmapPlanLinkDto {
        id: link.id.value().to_string(),
        roadmap_item_id: link.roadmap_item_id.value().to_string(),
        plan_item_id: link.plan_item_id.value().to_string(),
        link_type: link.link_type.clone(),
        created_at: link.created_at.inner().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::tests::mock_store::MockStore;

    #[test]
    fn test_list_projects_empty() {
        let store = MockStore::new();
        let projects = list_roadmaps(&store, None).unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_project_not_found_error() {
        let store = MockStore::new();
        let result = get_roadmap(&store, "nonexistent");
        assert!(result.is_err());
    }
}
