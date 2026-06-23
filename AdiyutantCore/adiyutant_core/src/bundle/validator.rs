use std::collections::HashSet;

use crate::bundle::bundle_models::AdiyutantBundle;
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
    }
    sections.push(BundleSectionReportDto {
        section: "projects".into(),
        status: "preview_only".into(),
        entity_count: bundle.projects.len(),
        actions: vec![],
        issues: vec![],
    });

    for road in &bundle.roadmaps {
        if !slug_set.insert(road.slug.clone()) {
            errors.push(BundleIssueDto {
                severity: "error".into(),
                section: "roadmaps".into(),
                code: "duplicate_slug".into(),
                message: format!("Duplicate slug in roadmaps: {}", road.slug),
                field: Some(format!("roadmaps.{}", road.slug)),
            });
        }
    }
    sections.push(BundleSectionReportDto {
        section: "roadmaps".into(),
        status: "preview_only".into(),
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

    let status = if errors.is_empty() && warnings.is_empty() {
        BundleStatus::Valid
    } else if errors.is_empty() {
        BundleStatus::ValidWithWarnings
    } else {
        BundleStatus::Invalid
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
