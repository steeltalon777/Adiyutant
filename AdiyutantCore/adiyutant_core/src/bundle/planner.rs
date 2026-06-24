use crate::bundle::bundle_models::{
    AdiyutantBundle, BundleChecklistTemplate, BundleProject, BundleRoadmap, ContextDocEntry,
    ImportMode, RoutineEntry, RuleEntry,
};
use crate::bundle::dto::{
    BundleActionDto, BundleActionKind, BundleIssueDto, BundlePreviewDto, BundleSectionReportDto,
    BundleStatus, BundleValidationReportDto,
};
use crate::bundle::validator::validate_bundle;
use crate::error::CoreResult;
use crate::store::Store;

/// Compute the import plan for a validated bundle.
///
/// This is the shared planner used by both `preview` and `apply`.
/// Returns the actions that would be taken (or were taken).
pub fn compute_import_plan(
    bundle: &AdiyutantBundle,
    mode: ImportMode,
    store: &dyn Store<Error = crate::error::CoreError>,
) -> CoreResult<BundlePreviewDto> {
    let validation = validate_bundle(bundle)?;

    let all_errors = validation.errors.clone();
    let all_warnings = validation.warnings.clone();

    if validation.status == BundleStatus::Invalid {
        return Ok(BundlePreviewDto {
            bundle_id: validation.bundle_id.clone(),
            schema_version: validation.schema_version.clone(),
            title: validation.title.clone(),
            status: BundleStatus::Invalid,
            errors: all_errors,
            warnings: all_warnings,
            sections: validation.sections,
            actions: vec![],
            mode: mode.to_string(),
        });
    }

    let mut all_actions: Vec<BundleActionDto> = Vec::new();
    let mut section_reports: Vec<BundleSectionReportDto> = Vec::new();

    // ── life_core ──
    if let Some(lc) = &bundle.life_core {
        let mut actions = Vec::new();
        if lc.profile.is_some() {
            let exists = store
                .get_context_document_by_source_slug("profile")
                .ok()
                .flatten()
                .is_some();
            actions.push(make_action(exists, mode, "life_core", "profile"));
        }
        for doc in &lc.context_documents {
            let slug = doc.slug.as_deref().unwrap_or(&doc.title);
            let exists = store
                .get_context_document_by_source_slug(slug)
                .ok()
                .flatten()
                .is_some();
            actions.push(make_action(exists, mode, "life_core", slug));
        }
        all_actions.extend(actions.clone());
        section_reports.push(BundleSectionReportDto {
            section: "life_core".into(),
            status: if actions.is_empty() { "skipped" } else { "ok" }.into(),
            entity_count: actions.len(),
            actions,
            issues: section_issues(&validation, "life_core"),
        });
    }

    // ── routines ──
    if let Some(rs) = &bundle.routines {
        let actions = plan_routines(&rs.routines, mode, store);
        all_actions.extend(actions.iter().cloned());
        section_reports.push(BundleSectionReportDto {
            section: "routines".into(),
            status: if actions.is_empty() { "skipped" } else { "ok" }.into(),
            entity_count: actions.len(),
            actions,
            issues: section_issues(&validation, "routines"),
        });
    }

    // ── rules ──
    if let Some(rs) = &bundle.rules {
        let actions = plan_rules(&rs.rules, mode, store);
        all_actions.extend(actions.iter().cloned());
        section_reports.push(BundleSectionReportDto {
            section: "rules".into(),
            status: if actions.is_empty() { "skipped" } else { "ok" }.into(),
            entity_count: actions.len(),
            actions,
            issues: section_issues(&validation, "rules"),
        });
    }

    // ── checklists ──
    if let Some(cs) = &bundle.checklists {
        let actions = plan_checklists(&cs.templates, mode, store);
        all_actions.extend(actions.iter().cloned());
        section_reports.push(BundleSectionReportDto {
            section: "checklists".into(),
            status: if actions.is_empty() { "skipped" } else { "ok" }.into(),
            entity_count: actions.len(),
            actions,
            issues: section_issues(&validation, "checklists"),
        });
    }

    // ── planning ──
    if bundle.planning.is_some() {
        let exists = store
            .get_context_document_by_source_slug("planning")
            .ok()
            .flatten()
            .is_some();
        let action = make_action(exists, mode, "planning", "default");
        all_actions.push(action.clone());
        section_reports.push(BundleSectionReportDto {
            section: "planning".into(),
            status: "ok".into(),
            entity_count: 1,
            actions: vec![action],
            issues: section_issues(&validation, "planning"),
        });
    }

    // ── context ──
    let actions = plan_context_docs(&bundle.context_docs, mode, store);
    all_actions.extend(actions.iter().cloned());
    section_reports.push(BundleSectionReportDto {
        section: "context".into(),
        status: if actions.is_empty() { "skipped" } else { "ok" }.into(),
        entity_count: actions.len(),
        actions,
        issues: section_issues(&validation, "context"),
    });

    // ── projects ──
    let project_actions = plan_projects(&bundle.projects, mode, store);
    all_actions.extend(project_actions.iter().cloned());
    section_reports.push(BundleSectionReportDto {
        section: "projects".into(),
        status: if project_actions.is_empty() {
            "skipped"
        } else {
            "ok"
        }
        .into(),
        entity_count: project_actions.len(),
        actions: project_actions,
        issues: section_issues(&validation, "projects"),
    });

    // ── roadmaps ──
    let roadmap_actions = plan_roadmaps(&bundle.roadmaps, mode, store);
    all_actions.extend(roadmap_actions.iter().cloned());
    section_reports.push(BundleSectionReportDto {
        section: "roadmaps".into(),
        status: if roadmap_actions.is_empty() {
            "skipped"
        } else {
            "ok"
        }
        .into(),
        entity_count: roadmap_actions.len(),
        actions: roadmap_actions,
        issues: section_issues(&validation, "roadmaps"),
    });

    let has_errors = all_actions
        .iter()
        .any(|a| a.kind == BundleActionKind::Error);

    Ok(BundlePreviewDto {
        bundle_id: validation.bundle_id,
        schema_version: validation.schema_version,
        title: validation.title,
        status: if has_errors {
            BundleStatus::Invalid
        } else if !all_warnings.is_empty() {
            BundleStatus::ValidWithWarnings
        } else {
            BundleStatus::Valid
        },
        errors: all_errors,
        warnings: all_warnings,
        sections: section_reports,
        actions: all_actions,
        mode: mode.to_string(),
    })
}

fn make_action(exists: bool, mode: ImportMode, section: &str, identifier: &str) -> BundleActionDto {
    let (kind, msg) = match mode {
        ImportMode::Append => {
            if exists {
                (
                    BundleActionKind::SkipExisting,
                    format!("Slug '{identifier}' already exists, skipping"),
                )
            } else {
                (BundleActionKind::Create, format!("Create '{identifier}'"))
            }
        }
        ImportMode::Merge => {
            if exists {
                (BundleActionKind::Update, format!("Update '{identifier}'"))
            } else {
                (BundleActionKind::Create, format!("Create '{identifier}'"))
            }
        }
    };
    BundleActionDto {
        kind,
        section: section.to_string(),
        entity_identifier: identifier.to_string(),
        message: msg,
    }
}

fn plan_routines(
    items: &[RoutineEntry],
    mode: ImportMode,
    store: &dyn Store<Error = crate::error::CoreError>,
) -> Vec<BundleActionDto> {
    items
        .iter()
        .map(|e| {
            let ident = e.slug.as_deref().unwrap_or(&e.title);
            let exists = store
                .get_context_document_by_source_slug(ident)
                .ok()
                .flatten()
                .is_some();
            make_action(exists, mode, "routines", ident)
        })
        .collect()
}

fn plan_rules(
    items: &[RuleEntry],
    mode: ImportMode,
    store: &dyn Store<Error = crate::error::CoreError>,
) -> Vec<BundleActionDto> {
    items
        .iter()
        .map(|e| {
            let ident = e.slug.as_deref().unwrap_or(&e.title);
            let exists = store
                .get_context_document_by_source_slug(ident)
                .ok()
                .flatten()
                .is_some();
            make_action(exists, mode, "rules", ident)
        })
        .collect()
}

fn plan_checklists(
    items: &[BundleChecklistTemplate],
    mode: ImportMode,
    store: &dyn Store<Error = crate::error::CoreError>,
) -> Vec<BundleActionDto> {
    items
        .iter()
        .map(|t| {
            let exists = store
                .get_checklist_template_by_slug(&t.slug)
                .ok()
                .flatten()
                .is_some();
            make_action(exists, mode, "checklists", &t.slug)
        })
        .collect()
}

fn plan_context_docs(
    items: &[ContextDocEntry],
    mode: ImportMode,
    store: &dyn Store<Error = crate::error::CoreError>,
) -> Vec<BundleActionDto> {
    items
        .iter()
        .map(|d| {
            let ident = d.slug.as_deref().unwrap_or(&d.title);
            let exists = store
                .get_context_document_by_source_slug(ident)
                .ok()
                .flatten()
                .is_some();
            make_action(exists, mode, "context", ident)
        })
        .collect()
}

fn plan_projects(
    items: &[BundleProject],
    mode: ImportMode,
    store: &dyn Store<Error = crate::error::CoreError>,
) -> Vec<BundleActionDto> {
    items
        .iter()
        .map(|bp| {
            let exists = store.get_project_by_slug(&bp.slug).ok().flatten().is_some();
            make_action(exists, mode, "projects", &bp.slug)
        })
        .collect()
}

fn plan_roadmaps(
    items: &[BundleRoadmap],
    mode: ImportMode,
    store: &dyn Store<Error = crate::error::CoreError>,
) -> Vec<BundleActionDto> {
    let mut actions = Vec::new();
    for br in items {
        let ident = format!("{}/{}", br.project_slug, br.slug);
        let project = store.get_project_by_slug(&br.project_slug).ok().flatten();
        let road_exists = project
            .as_ref()
            .and_then(|proj| {
                store
                    .get_roadmap_by_project_and_slug(proj.id, &br.slug)
                    .ok()
                    .flatten()
            })
            .is_some();
        actions.push(make_action(road_exists, mode, "roadmaps", &ident));
    }
    actions
}

fn section_issues(v: &BundleValidationReportDto, section: &str) -> Vec<BundleIssueDto> {
    let mut issues = Vec::new();
    for s in &v.sections {
        if s.section == section {
            issues.extend(s.issues.clone());
        }
    }
    issues
}
