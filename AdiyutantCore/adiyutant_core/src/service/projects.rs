use crate::dto::{ProjectDetailDto, ProjectDto, RoadmapDto};
use crate::error::{CoreError, CoreResult};
use crate::model::project::Project;
use crate::store::Store;

/// Map a Project domain model to ProjectDto.
pub fn project_to_dto(p: &Project) -> ProjectDto {
    ProjectDto {
        id: p.id.value().to_string(),
        slug: p.slug.clone(),
        title: p.title.clone(),
        description: p.description.as_deref().unwrap_or("").to_string(),
        status: p.status.as_str().to_string(),
        priority: p.priority,
        why: p.why.as_deref().unwrap_or("").to_string(),
        created_at: p.created_at.inner().to_rfc3339(),
        updated_at: p.updated_at.inner().to_rfc3339(),
    }
}

/// List all projects.
pub fn list_projects(store: &dyn Store<Error = CoreError>) -> CoreResult<Vec<ProjectDto>> {
    let projects = store.list_projects()?;
    Ok(projects.iter().map(project_to_dto).collect())
}

/// Get a single project by slug, with its roadmaps.
pub fn get_project(
    store: &dyn Store<Error = CoreError>,
    slug: &str,
) -> CoreResult<ProjectDetailDto> {
    let project = store
        .get_project_by_slug(slug)?
        .ok_or_else(|| CoreError::NotFound(format!("project '{slug}'")))?;

    let roadmaps = store.list_roadmaps_by_project(project.id)?;
    let roadmap_dtos: Vec<RoadmapDto> = roadmaps
        .iter()
        .map(super::roadmap::roadmap_to_dto)
        .collect();

    Ok(ProjectDetailDto {
        project: project_to_dto(&project),
        roadmaps: roadmap_dtos,
    })
}
