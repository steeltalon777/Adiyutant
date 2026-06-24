use std::collections::HashSet;

use crate::bundle::bundle_models::{AdiyutantBundle, BundleRoadmap};
use crate::bundle::dto::{
    BundleIssueDto, BundleSectionReportDto, BundleStatus, BundleValidationReportDto,
};
use crate::error::CoreResult;

pub fn validate_bundle(bundle: &AdiyutantBundle) -> CoreResult<BundleValidationReportDto> {
    let mut errors: Vec<BundleIssueDto> = Vec::new();
    let mut warnings: Vec<BundleIssueDto> = Vec::new();
    let mut sections: Vec<BundleSectionReportDto> = Vec::new();

    let manifest_issues = bundle.manifest.validate();
    for issue in manifest_issues {
        if issue.severity == "error" {
            errors.push(issue);
        } else {
            warnings.push(issue);
        }
    }

    sections.push(BundleSectionReportDto {
        section: "manifest".into(),
        status: "ok".into(),
        entity_count: 1,
        actions: vec![],
        issues: vec![],
    });

    let mut slug_set: HashSet<String> = HashSet::new();

    if let Some(ref lc) = bundle.life_core {
        for doc in &lc.context_documents {
            if let Some(ref slug) = doc.slug
                && !slug_set.insert(slug.clone())
            {
                errors.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "life_core".into(),
                    code: "duplicate_slug".into(),
                    message: format!("Duplicate slug in life_core context_documents: {slug}"),
                    field: Some(format!("life_core.context_documents.{slug}")),
                });
            }
        }
        sections.push(BundleSectionReportDto {
            section: "life_core".into(),
            status: "ok".into(),
            entity_count: lc.context_documents.len() + if lc.profile.is_some() { 1 } else { 0 },
            actions: vec![],
            issues: vec![],
        });
    }

    if let Some(ref r) = bundle.routines {
        for entry in &r.routines {
            if let Some(ref slug) = entry.slug
                && !slug_set.insert(slug.clone())
            {
                errors.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "routines".into(),
                    code: "duplicate_slug".into(),
                    message: format!("Duplicate slug in routines: {slug}"),
                    field: Some(format!("routines.{slug}")),
                });
            }
        }
        sections.push(BundleSectionReportDto {
            section: "routines".into(),
            status: "ok".into(),
            entity_count: r.routines.len(),
            actions: vec![],
            issues: vec![],
        });
    }

    if let Some(ref r) = bundle.rules {
        for entry in &r.rules {
            if let Some(ref slug) = entry.slug
                && !slug_set.insert(slug.clone())
            {
                errors.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "rules".into(),
                    code: "duplicate_slug".into(),
                    message: format!("Duplicate slug in rules: {slug}"),
                    field: Some(format!("rules.{slug}")),
                });
            }
        }
        sections.push(BundleSectionReportDto {
            section: "rules".into(),
            status: "ok".into(),
            entity_count: r.rules.len(),
            actions: vec![],
            issues: vec![],
        });
    }

    if let Some(ref c) = bundle.checklists {
        for template in &c.templates {
            if !slug_set.insert(template.slug.clone()) {
                errors.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "checklists".into(),
                    code: "duplicate_slug".into(),
                    message: format!("Duplicate slug in checklists: {}", template.slug),
                    field: Some(format!("checklists.{}", template.slug)),
                });
            }
            for item in &template.items {
                if let Some(ref slug) = item.slug
                    && !slug_set.insert(slug.clone())
                {
                    errors.push(BundleIssueDto {
                        severity: "error".into(),
                        section: "checklists".into(),
                        code: "duplicate_slug".into(),
                        message: format!("Duplicate slug in checklist items: {slug}"),
                        field: Some(format!("checklists.items.{slug}")),
                    });
                }
            }
        }
        sections.push(BundleSectionReportDto {
            section: "checklists".into(),
            status: "ok".into(),
            entity_count: c.templates.len(),
            actions: vec![],
            issues: vec![],
        });
    }

    if let Some(ref p) = bundle.planning {
        sections.push(BundleSectionReportDto {
            section: "planning".into(),
            status: "preview_only".into(),
            entity_count: p.templates.len(),
            actions: vec![],
            issues: vec![],
        });
    }

    let mut seen_context_slugs: HashSet<String> = HashSet::new();
    for doc in &bundle.context_docs {
        if let Some(ref slug) = doc.slug
            && !seen_context_slugs.insert(slug.clone())
        {
            errors.push(BundleIssueDto {
                severity: "error".into(),
                section: "context".into(),
                code: "duplicate_slug".into(),
                message: format!("Duplicate context doc slug: {slug}"),
                field: Some(format!("context.{slug}")),
            });
        }
    }
    sections.push(BundleSectionReportDto {
        section: "context".into(),
        status: "ok".into(),
        entity_count: bundle.context_docs.len(),
        actions: vec![],
        issues: vec![],
    });

    for proj in &bundle.projects {
        if !slug_set.insert(proj.slug.clone()) {
            errors.push(BundleIssueDto {
                severity: "error".into(),
                section: "projects".into(),
                code: "duplicate_slug".into(),
                message: format!("Duplicate slug in projects: {}", proj.slug),
                field: Some(format!("projects.{}", proj.slug)),
            });
        }
        if proj.title.is_empty() {
            errors.push(BundleIssueDto {
                severity: "error".into(),
                section: "projects".into(),
                code: "missing_field".into(),
                message: format!("Project '{}' has empty title", proj.slug),
                field: Some(format!("projects.{}.title", proj.slug)),
            });
        }
    }
    sections.push(BundleSectionReportDto {
        section: "projects".into(),
        status: if bundle.projects.is_empty() {
            "skipped"
        } else {
            "ok"
        }
        .into(),
        entity_count: bundle.projects.len(),
        actions: vec![],
        issues: vec![],
    });

    // ── roadmaps ──
    for road in &bundle.roadmaps {
        let rm_key = format!("{}::{}", road.project_slug, road.slug);
        if !slug_set.insert(rm_key.clone()) {
            errors.push(BundleIssueDto {
                severity: "error".into(),
                section: "roadmaps".into(),
                code: "duplicate_slug".into(),
                message: format!("Duplicate roadmap project_slug/slug: {rm_key}"),
                field: Some(format!("roadmaps.{rm_key}")),
            });
        }
        if road.title.is_empty() {
            errors.push(BundleIssueDto {
                severity: "error".into(),
                section: "roadmaps".into(),
                code: "missing_field".into(),
                message: format!("Roadmap '{}' has empty title", road.slug),
                field: Some(format!("roadmaps.{}.title", road.slug)),
            });
        }
        validate_roadmap_phases(road, &mut errors);
    }
    sections.push(BundleSectionReportDto {
        section: "roadmaps".into(),
        status: if bundle.roadmaps.is_empty() {
            "skipped"
        } else {
            "ok"
        }
        .into(),
        entity_count: bundle.roadmaps.len(),
        actions: vec![],
        issues: vec![],
    });

    for unknown in &bundle.unknown_files {
        warnings.push(BundleIssueDto {
            severity: "warning".into(),
            section: "archive".into(),
            code: "unknown_file".into(),
            message: format!("File not declared in manifest: {unknown}"),
            field: Some(unknown.clone()),
        });
    }

    let status = if !errors.is_empty() {
        BundleStatus::Invalid
    } else if !warnings.is_empty() {
        BundleStatus::ValidWithWarnings
    } else {
        BundleStatus::Valid
    };

    Ok(BundleValidationReportDto {
        bundle_id: bundle.manifest.bundle_id.clone(),
        schema_version: bundle.manifest.schema_version.clone(),
        title: bundle.manifest.title.clone(),
        status,
        errors,
        warnings,
        sections,
    })
}

fn validate_roadmap_phases(roadmap: &BundleRoadmap, errors: &mut Vec<BundleIssueDto>) {
    // First pass: collect all item slugs across all phases
    let all_item_slugs: HashSet<String> = roadmap
        .phases
        .iter()
        .flat_map(|ph| ph.items.iter().map(|i| i.slug.clone()))
        .collect();

    let mut phase_slugs: HashSet<String> = HashSet::new();
    for phase in &roadmap.phases {
        if !phase_slugs.insert(phase.slug.clone()) {
            errors.push(BundleIssueDto {
                severity: "error".into(),
                section: "roadmaps".into(),
                code: "duplicate_slug".into(),
                message: format!(
                    "Duplicate phase slug '{}' in roadmap '{}'",
                    phase.slug, roadmap.slug
                ),
                field: Some(format!("roadmaps.{}.phases.{}", roadmap.slug, phase.slug)),
            });
        }
        let mut item_slugs: HashSet<String> = HashSet::new();
        for item in &phase.items {
            if !item_slugs.insert(item.slug.clone()) {
                errors.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "roadmaps".into(),
                    code: "duplicate_slug".into(),
                    message: format!(
                        "Duplicate item slug '{}' in phase '{}' of roadmap '{}'",
                        item.slug, phase.slug, roadmap.slug
                    ),
                    field: Some(format!(
                        "roadmaps.{}.phases.{}.items.{}",
                        roadmap.slug, phase.slug, item.slug
                    )),
                });
            }
            for dep in &item.depends_on {
                if !all_item_slugs.contains(dep) {
                    errors.push(BundleIssueDto {
                        severity: "error".into(),
                        section: "roadmaps".into(),
                        code: "unresolved_dependency".into(),
                        message: format!(
                            "Item '{}' depends on '{}' which does not exist in roadmap '{}'",
                            item.slug, dep, roadmap.slug
                        ),
                        field: Some(format!(
                            "roadmaps.{}.phases.{}.items.{}.depends_on",
                            roadmap.slug, phase.slug, item.slug
                        )),
                    });
                }
            }
        }
    }
}
