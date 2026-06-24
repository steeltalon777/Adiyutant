use std::io::Write;
use std::path::Path;

use crate::bundle::dto::{BundleExportReportDto, BundleIssueDto, BundleStatus};
use crate::bundle::manifest::{BUNDLE_SCHEMA_VERSION, Manifest};
use crate::datetime::AdiyutantDateTime;
use crate::error::{CoreError, CoreResult};
use crate::model::checklist_template::ChecklistTemplate;
use crate::model::context_document::{ContextDocument, ContextDocumentType};
use crate::model::project::Project;
use crate::model::roadmap::Roadmap;
use crate::store::Store;

/// Export supported sections from the store into a bundle directory or zip archive.
pub fn export_bundle(
    target: &Path,
    store: &dyn Store<Error = CoreError>,
) -> CoreResult<BundleExportReportDto> {
    let bundle_id = format!("export-{}", uuid::Uuid::new_v4());
    let now = AdiyutantDateTime::now();
    let created_at = now.inner().to_rfc3339();

    let errors: Vec<BundleIssueDto> = Vec::new();
    let mut warnings: Vec<BundleIssueDto> = Vec::new();

    // ── Read data from store ──
    let context_docs = store.list_context_documents().unwrap_or_else(|e| {
        warnings.push(BundleIssueDto {
            severity: "warning".into(),
            section: "life_core".into(),
            code: "read_failed".into(),
            message: format!("Failed to list context documents: {e}"),
            field: None,
        });
        vec![]
    });

    let checklist_templates = store.list_checklist_templates().unwrap_or_else(|e| {
        warnings.push(BundleIssueDto {
            severity: "warning".into(),
            section: "checklists".into(),
            code: "read_failed".into(),
            message: format!("Failed to list checklist templates: {e}"),
            field: None,
        });
        vec![]
    });

    let projects = store.list_projects().unwrap_or_else(|e| {
        warnings.push(BundleIssueDto {
            severity: "warning".into(),
            section: "projects".into(),
            code: "read_failed".into(),
            message: format!("Failed to list projects: {e}"),
            field: None,
        });
        vec![]
    });

    let roadmaps = store.list_roadmaps().unwrap_or_else(|e| {
        warnings.push(BundleIssueDto {
            severity: "warning".into(),
            section: "roadmaps".into(),
            code: "read_failed".into(),
            message: format!("Failed to list roadmaps: {e}"),
            field: None,
        });
        vec![]
    });

    // ── Build manifest ──
    let manifest = Manifest {
        schema_version: BUNDLE_SCHEMA_VERSION.to_string(),
        bundle_id: bundle_id.clone(),
        title: format!("Adiyutant Export {}", now.inner().format("%Y-%m-%d")),
        created_at: created_at.clone(),
        capabilities: [
            ("projects".into(), "apply-export".into()),
            ("roadmaps".into(), "apply-export".into()),
        ]
        .into(),
        sections: [
            ("life_core".into(), "life-core.yaml".into()),
            ("routines".into(), "routines.yaml".into()),
            ("rules".into(), "rules.yaml".into()),
            ("checklists".into(), "checklists.yaml".into()),
            ("planning".into(), "planning.yaml".into()),
            ("projects".into(), "projects/".into()),
            ("roadmaps".into(), "roadmaps/".into()),
        ]
        .into(),
    };

    let manifest_yaml = serde_yaml::to_string(&manifest)
        .map_err(|e| CoreError::Internal(format!("Failed to serialize manifest: {e}")))?;

    // ── Build section YAMLs ──
    let life_core_yaml = build_life_core_yaml(&context_docs);
    let routines_yaml = build_routines_yaml(&context_docs);
    let rules_yaml = build_rules_yaml(&context_docs);
    let checklists_yaml = build_checklists_yaml(&checklist_templates);
    let planning_yaml = build_planning_yaml();
    let projects_yaml = build_projects_yaml(&projects);
    let roadmaps_yaml = build_roadmaps_yaml(&roadmaps, store);

    // ── Write bundle ──
    let is_zip = target.extension().map(|ext| ext == "zip").unwrap_or(false);

    if is_zip {
        write_zip(
            target,
            &manifest_yaml,
            &life_core_yaml,
            &routines_yaml,
            &rules_yaml,
            &checklists_yaml,
            &planning_yaml,
            &context_docs,
            &projects_yaml,
            &roadmaps_yaml,
        )?;
    } else {
        write_directory(
            target,
            &manifest_yaml,
            &life_core_yaml,
            &routines_yaml,
            &rules_yaml,
            &checklists_yaml,
            &planning_yaml,
            &context_docs,
            &projects_yaml,
            &roadmaps_yaml,
        )?;
    }

    let section_count = if context_docs.is_empty() { 0usize } else { 1 }
        + if checklist_templates.is_empty() { 0 } else { 1 }
        + if projects.is_empty() { 0 } else { 1 }
        + if roadmaps.is_empty() { 0 } else { 1 }
        + 3; // routines, rules, planning always present

    Ok(BundleExportReportDto {
        bundle_id,
        schema_version: BUNDLE_SCHEMA_VERSION.to_string(),
        title: manifest.title,
        status: if errors.is_empty() {
            BundleStatus::Valid
        } else {
            BundleStatus::ValidWithWarnings
        },
        target_path: target.to_string_lossy().to_string(),
        section_count,
        errors,
        warnings,
    })
}

fn build_life_core_yaml(docs: &[ContextDocument]) -> String {
    let profile = serde_json::json!({
        "name": null,
        "bio": null,
        "values": [],
        "goals": [],
    });

    let context_docs: Vec<serde_json::Value> = docs
        .iter()
        .filter(|d| {
            matches!(
                d.doc_type,
                ContextDocumentType::Core
                    | ContextDocumentType::LifeCore
                    | ContextDocumentType::Goals
                    | ContextDocumentType::Tone
                    | ContextDocumentType::RecoveryProtocol
            )
        })
        .map(|d| {
            serde_json::json!({
                "slug": doc_slug(d),
                "title": d.title,
                "doc_type": doc_type_str(&d.doc_type),
                "content": d.content_markdown,
            })
        })
        .collect();

    let yaml_value = serde_json::json!({
        "profile": profile,
        "context_documents": context_docs,
    });

    serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| "profile:\n  name: null\n  bio: null\n  values: []\n  goals: []\ncontext_documents: []\n".into())
}

fn build_routines_yaml(docs: &[ContextDocument]) -> String {
    let routines: Vec<serde_json::Value> = docs
        .iter()
        .filter(|d| d.doc_type == ContextDocumentType::Routine)
        .map(|d| {
            serde_json::json!({
                "slug": doc_slug(d),
                "title": d.title,
                "description": null,
                "steps": [],
            })
        })
        .collect();

    let yaml_value = serde_json::json!({ "routines": routines });
    serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| "routines: []\n".into())
}

fn build_rules_yaml(docs: &[ContextDocument]) -> String {
    let rules: Vec<serde_json::Value> = docs
        .iter()
        .filter(|d| d.doc_type == ContextDocumentType::Rules)
        .map(|d| {
            serde_json::json!({
                "slug": doc_slug(d),
                "title": d.title,
                "description": d.content_markdown,
                "category": "rules",
            })
        })
        .collect();

    let yaml_value = serde_json::json!({ "rules": rules });
    serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| "rules: []\n".into())
}

fn build_checklists_yaml(templates: &[ChecklistTemplate]) -> String {
    let items: Vec<serde_json::Value> = templates
        .iter()
        .map(|t| {
            let checklist_items: Vec<serde_json::Value> = t
                .items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "slug": i.id.value().to_string(),
                        "question": i.question,
                        "kind": format!("{:?}", i.kind),
                        "options": i.options,
                        "order": i.order,
                    })
                })
                .collect();
            serde_json::json!({
                "slug": t.id.value().to_string(),
                "title": t.title,
                "category": t.category,
                "items": checklist_items,
            })
        })
        .collect();

    let yaml_value = serde_json::json!({ "templates": items });
    serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| "templates: []\n".into())
}

fn build_planning_yaml() -> String {
    let yaml_value = serde_json::json!({
        "default_mode": "light",
        "daily_capacity": {
            "deep_tasks_max": 3,
            "light_tasks_max": 5,
        },
        "templates": [],
        "candidate_hints": [],
    });
    serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| "default_mode: light\ndaily_capacity:\n  deep_tasks_max: 3\n  light_tasks_max: 5\ntemplates: []\ncandidate_hints: []\n".into())
}

fn build_projects_yaml(projects: &[Project]) -> Vec<(String, String)> {
    projects
        .iter()
        .map(|p| {
            let yaml_value = serde_json::json!({
                "slug": p.slug,
                "title": p.title,
                "description": p.description,
                "status": p.status.as_str(),
                "priority": p.priority,
                "why": p.why,
            });
            let yaml = serde_yaml::to_string(&yaml_value)
                .unwrap_or_else(|_| format!("slug: {}\ntitle: {}\n", p.slug, p.title));
            (p.slug.clone(), yaml)
        })
        .collect()
}

fn build_roadmaps_yaml(
    roadmaps: &[Roadmap],
    store: &dyn Store<Error = CoreError>,
) -> Vec<(String, String)> {
    roadmaps
        .iter()
        .map(|r| {
            let project_slug = store
                .get_project(r.project_id)
                .ok()
                .flatten()
                .map(|proj| proj.slug)
                .unwrap_or_default();
            let phases = store.list_roadmap_phases(r.id).unwrap_or_default();
            let bundle_phases: Vec<serde_json::Value> = phases
                .iter()
                .map(|ph| {
                    let items = store.list_roadmap_items(ph.id).unwrap_or_default();
                    let bundle_items: Vec<serde_json::Value> = items
                        .iter()
                        .map(|i| {
                            serde_json::json!({
                                "slug": i.slug,
                                "title": i.title,
                                "description": i.description,
                                "status": i.status.as_str(),
                                "priority": i.priority,
                                "acceptance_criteria": i.acceptance_criteria,
                                "depends_on": i.depends_on,
                                "links": i.links,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "slug": ph.slug,
                        "title": ph.title,
                        "order_index": ph.order_index,
                        "status": ph.status.as_str(),
                        "items": bundle_items,
                    })
                })
                .collect();
            let yaml_value = serde_json::json!({
                "slug": r.slug,
                "project_slug": project_slug,
                "title": r.title,
                "description": r.description,
                "horizon": r.horizon.as_str(),
                "status": r.status.as_str(),
                "phases": bundle_phases,
            });
            let yaml = serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| {
                format!(
                    "slug: {}\ntitle: {}\nproject_slug: {}\n",
                    r.slug, r.title, project_slug
                )
            });
            (r.slug.clone(), yaml)
        })
        .collect()
}

// ── File writers ──

#[allow(clippy::too_many_arguments)]
fn write_directory(
    base: &Path,
    manifest_yaml: &str,
    life_core_yaml: &str,
    routines_yaml: &str,
    rules_yaml: &str,
    checklists_yaml: &str,
    planning_yaml: &str,
    context_docs: &[ContextDocument],
    projects_yaml: &[(String, String)],
    roadmaps_yaml: &[(String, String)],
) -> Result<(), CoreError> {
    std::fs::create_dir_all(base)
        .map_err(|e| CoreError::Storage(format!("Cannot create target dir: {e}")))?;
    std::fs::create_dir_all(base.join("context"))
        .map_err(|e| CoreError::Storage(format!("Cannot create context dir: {e}")))?;

    write_file(base, "manifest.yaml", manifest_yaml)?;
    write_file(base, "life-core.yaml", life_core_yaml)?;
    write_file(base, "routines.yaml", routines_yaml)?;
    write_file(base, "rules.yaml", rules_yaml)?;
    write_file(base, "checklists.yaml", checklists_yaml)?;
    write_file(base, "planning.yaml", planning_yaml)?;

    for doc in context_docs {
        let filename = format!("{}.md", doc_slug(doc));
        let path = base.join("context").join(&filename);
        write_file_content(&path, &doc.content_markdown)?;
    }

    if !projects_yaml.is_empty() {
        std::fs::create_dir_all(base.join("projects"))
            .map_err(|e| CoreError::Storage(format!("Cannot create projects dir: {e}")))?;
        for (slug, yaml) in projects_yaml {
            let path = base.join("projects").join(format!("{slug}.yaml"));
            write_file_content(&path, yaml)?;
        }
    }

    if !roadmaps_yaml.is_empty() {
        std::fs::create_dir_all(base.join("roadmaps"))
            .map_err(|e| CoreError::Storage(format!("Cannot create roadmaps dir: {e}")))?;
        for (slug, yaml) in roadmaps_yaml {
            let path = base.join("roadmaps").join(format!("{slug}.yaml"));
            write_file_content(&path, yaml)?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_zip(
    target: &Path,
    manifest_yaml: &str,
    life_core_yaml: &str,
    routines_yaml: &str,
    rules_yaml: &str,
    checklists_yaml: &str,
    planning_yaml: &str,
    context_docs: &[ContextDocument],
    projects_yaml: &[(String, String)],
    roadmaps_yaml: &[(String, String)],
) -> Result<(), CoreError> {
    let file = std::fs::File::create(target)
        .map_err(|e| CoreError::Storage(format!("Cannot create zip file: {e}")))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_to_zip(
        &mut zip,
        "manifest.yaml",
        manifest_yaml.as_bytes(),
        &options,
    )?;
    add_to_zip(
        &mut zip,
        "life-core.yaml",
        life_core_yaml.as_bytes(),
        &options,
    )?;
    add_to_zip(
        &mut zip,
        "routines.yaml",
        routines_yaml.as_bytes(),
        &options,
    )?;
    add_to_zip(&mut zip, "rules.yaml", rules_yaml.as_bytes(), &options)?;
    add_to_zip(
        &mut zip,
        "checklists.yaml",
        checklists_yaml.as_bytes(),
        &options,
    )?;
    add_to_zip(
        &mut zip,
        "planning.yaml",
        planning_yaml.as_bytes(),
        &options,
    )?;

    for doc in context_docs {
        let filename = format!("context/{}.md", doc_slug(doc));
        add_to_zip(
            &mut zip,
            &filename,
            doc.content_markdown.as_bytes(),
            &options,
        )?;
    }

    for (slug, yaml) in projects_yaml {
        let path = format!("projects/{slug}.yaml");
        add_to_zip(&mut zip, &path, yaml.as_bytes(), &options)?;
    }

    for (slug, yaml) in roadmaps_yaml {
        let path = format!("roadmaps/{slug}.yaml");
        add_to_zip(&mut zip, &path, yaml.as_bytes(), &options)?;
    }

    zip.finish()
        .map_err(|e| CoreError::Storage(format!("Failed to finalize zip: {e}")))?;
    Ok(())
}

fn add_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    name: &str,
    data: &[u8],
    options: &zip::write::SimpleFileOptions,
) -> Result<(), CoreError> {
    let opts = *options;
    zip.start_file(name, opts)
        .map_err(|e| CoreError::Storage(format!("Zip write error for '{name}': {e}")))?;
    zip.write_all(data)
        .map_err(|e| CoreError::Storage(format!("Zip write error for '{name}': {e}")))?;
    Ok(())
}

fn write_file(base: &Path, name: &str, content: &str) -> Result<(), CoreError> {
    let path = base.join(name);
    std::fs::write(&path, content)
        .map_err(|e| CoreError::Storage(format!("Cannot write {name}: {e}")))
}

fn write_file_content(path: &Path, content: &str) -> Result<(), CoreError> {
    std::fs::write(path, content)
        .map_err(|e| CoreError::Storage(format!("Cannot write {}: {e}", path.display())))
}

fn doc_slug(doc: &ContextDocument) -> String {
    let slug: String = doc
        .title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    slug.trim_matches('-').to_string()
}

fn doc_type_str(dt: &ContextDocumentType) -> &'static str {
    match dt {
        ContextDocumentType::Core => "core",
        ContextDocumentType::Goals => "goals",
        ContextDocumentType::Rules => "rules",
        ContextDocumentType::Routine => "routine",
        ContextDocumentType::Health => "health",
        ContextDocumentType::Work => "work",
        ContextDocumentType::Custom => "custom",
        ContextDocumentType::LifeCore => "life_core",
        ContextDocumentType::RecoveryProtocol => "recovery_protocol",
        ContextDocumentType::Tone => "tone",
        ContextDocumentType::PlanningPreferences => "planning_preferences",
    }
}
