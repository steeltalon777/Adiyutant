use std::collections::HashMap;

use crate::bundle::bundle_models::BundleChecklistTemplate;
use crate::bundle::bundle_models::ImportMode;
use crate::bundle::bundle_models::{
    AdiyutantBundle, ChecklistsSection, LifeCoreContextDoc, LifeCoreSection, PlanningSection,
    RoutineEntry, RoutinesSection,
};
use crate::bundle::dto::BundleActionKind;
use crate::bundle::export::export_bundle;
use crate::bundle::manifest::{BUNDLE_SCHEMA_VERSION, Manifest};
use crate::bundle::planner::compute_import_plan;
use crate::bundle::security::{
    MAX_FILE_COUNT, MAX_MD_FILE_SIZE, MAX_YAML_FILE_SIZE, validate_zip_entries,
};
use crate::bundle::validator::validate_bundle;
use crate::model::checklist_template::ChecklistTemplate;
use crate::model::context_document::{ContextDocument, ContextDocumentType};
use crate::service::tests::mock_store::MockStore;
use crate::store::Store;

// ── Helpers ──

fn valid_manifest() -> Manifest {
    Manifest {
        schema_version: BUNDLE_SCHEMA_VERSION.to_string(),
        bundle_id: "test-bundle".into(),
        title: "Test Bundle".into(),
        created_at: "2026-06-22T10:00:00Z".into(),
        capabilities: HashMap::new(),
        sections: HashMap::new(),
    }
}

// ================================================================
// 1. Manifest parsing tests
// ================================================================

#[test]
fn test_manifest_valid() {
    let yaml = r#"
schema_version: "adiyutant.bundle.v1"
bundle_id: "test-1"
title: "Test Bundle"
created_at: "2026-06-22T10:00:00Z"
sections: {}
"#;
    let manifest = Manifest::from_yaml(yaml).unwrap();
    assert_eq!(manifest.bundle_id, "test-1");
    assert_eq!(manifest.schema_version, "adiyutant.bundle.v1");
    assert_eq!(manifest.title, "Test Bundle");
}

#[test]
fn test_manifest_missing_bundle_id() {
    let yaml = r#"
schema_version: "adiyutant.bundle.v1"
title: "Test Bundle"
created_at: "2026-06-22T10:00:00Z"
sections: {}
"#;
    let err = Manifest::from_yaml(yaml).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("bundle_id"),
        "expected bundle_id error, got: {msg}"
    );
}

#[test]
fn test_manifest_unsupported_schema_version() {
    let yaml = r#"
schema_version: "v0"
bundle_id: "test-1"
title: "Test Bundle"
created_at: "2026-06-22T10:00:00Z"
sections: {}
"#;
    let err = Manifest::from_yaml(yaml).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unsupported schema version"),
        "expected version error, got: {msg}"
    );
}

#[test]
fn test_manifest_unknown_field_warning() {
    let yaml = r#"
schema_version: "adiyutant.bundle.v1"
bundle_id: "test-1"
title: "Test Bundle"
created_at: "2026-06-22T10:00:00Z"
sections: {}
unknown_field: "oops"
another_unknown: "bad"
"#;
    let (_manifest, warnings) = Manifest::from_raw_yaml(yaml).unwrap();
    let codes: Vec<&str> = warnings.iter().map(|w| w.code.as_str()).collect();
    assert!(
        codes.contains(&"unknown_field"),
        "expected unknown_field warning, got: {codes:?}"
    );
    assert_eq!(warnings.len(), 2, "expected 2 unknown field warnings");
}

// ================================================================
// 2. Section validation tests
// ================================================================

#[test]
fn test_check_bundle_structure_missing_section_warning() {
    let dir = std::env::temp_dir().join(format!("adiyutant_test_struct_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Write manifest.yaml
    let manifest_yaml = r#"
schema_version: "adiyutant.bundle.v1"
bundle_id: "test-struct"
title: "Test"
created_at: "2026-06-22T10:00:00Z"
sections:
  life_core: "life-core.yaml"
  missing_section: "nonexistent.yaml"
"#;
    std::fs::write(dir.join("manifest.yaml"), manifest_yaml).unwrap();
    std::fs::write(
        dir.join("life-core.yaml"),
        "profile: {}\ncontext_documents: []\n",
    )
    .unwrap();

    let manifest = Manifest::from_yaml(manifest_yaml).unwrap();
    let issues = crate::bundle::security::check_bundle_structure(&dir, &manifest);

    let missing: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "missing_section")
        .collect();
    assert_eq!(
        missing.len(),
        1,
        "expected 1 missing_section issue, got {len}",
        len = missing.len()
    );
    assert!(missing[0].message.contains("nonexistent.yaml"));

    let _ = std::fs::remove_dir_all(&dir);
}

// ================================================================
// 3. Unknown field warning behavior (already covered by test_manifest_unknown_field_warning)
// ================================================================

#[test]
fn test_from_raw_yaml_preserves_unknown_as_warning_not_error() {
    let yaml = r#"
schema_version: "adiyutant.bundle.v1"
bundle_id: "test-1"
title: "Test"
created_at: "2026-06-22T10:00:00Z"
sections: {}
bogus: "value"
"#;
    let result = Manifest::from_raw_yaml(yaml);
    assert!(result.is_ok(), "expected Ok for unknown fields, got Err");
    let (_manifest, warnings) = result.unwrap();
    assert!(!warnings.is_empty(), "expected warnings for unknown fields");
}

// ================================================================
// 4. Duplicate slug detection
// ================================================================

#[test]
fn test_duplicate_slug_life_core_context_documents() {
    let manifest = valid_manifest();
    let bundle = AdiyutantBundle {
        life_core: Some(LifeCoreSection {
            profile: None,
            context_documents: vec![
                LifeCoreContextDoc {
                    slug: Some("dup-slug".into()),
                    title: "Doc 1".into(),
                    doc_type: "core".into(),
                    content: "a".into(),
                },
                LifeCoreContextDoc {
                    slug: Some("dup-slug".into()),
                    title: "Doc 2".into(),
                    doc_type: "core".into(),
                    content: "b".into(),
                },
            ],
        }),
        routines: None,
        rules: None,
        checklists: None,
        planning: None,
        context_docs: vec![],
        projects: vec![],
        roadmaps: vec![],
        unknown_files: vec![],
        manifest,
    };

    let report = validate_bundle(&bundle).unwrap();
    let dup_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.code == "duplicate_slug")
        .collect();
    assert_eq!(
        dup_errors.len(),
        1,
        "expected 1 duplicate slug error, got {len}",
        len = dup_errors.len()
    );
    assert!(dup_errors[0].message.contains("dup-slug"));
}

#[test]
fn test_duplicate_slug_checklists() {
    let manifest = valid_manifest();
    let bundle = AdiyutantBundle {
        life_core: None,
        routines: None,
        rules: None,
        checklists: Some(ChecklistsSection {
            templates: vec![
                BundleChecklistTemplate {
                    slug: "dup-template".into(),
                    title: "T1".into(),
                    category: "morning".into(),
                    items: vec![],
                },
                BundleChecklistTemplate {
                    slug: "dup-template".into(),
                    title: "T2".into(),
                    category: "evening".into(),
                    items: vec![],
                },
            ],
        }),
        planning: None,
        context_docs: vec![],
        projects: vec![],
        roadmaps: vec![],
        unknown_files: vec![],
        manifest,
    };

    let report = validate_bundle(&bundle).unwrap();
    let dup_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.code == "duplicate_slug")
        .collect();
    assert_eq!(
        dup_errors.len(),
        1,
        "expected 1 duplicate slug error, got {len}",
        len = dup_errors.len()
    );
}

#[test]
fn test_duplicate_slug_routines() {
    let manifest = valid_manifest();
    let bundle = AdiyutantBundle {
        life_core: None,
        routines: Some(RoutinesSection {
            routines: vec![
                RoutineEntry {
                    slug: Some("dup-routine".into()),
                    title: "R1".into(),
                    description: None,
                    steps: vec![],
                },
                RoutineEntry {
                    slug: Some("dup-routine".into()),
                    title: "R2".into(),
                    description: None,
                    steps: vec![],
                },
            ],
        }),
        rules: None,
        checklists: None,
        planning: None,
        context_docs: vec![],
        projects: vec![],
        roadmaps: vec![],
        unknown_files: vec![],
        manifest,
    };

    let report = validate_bundle(&bundle).unwrap();
    let dup_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.code == "duplicate_slug")
        .collect();
    assert_eq!(
        dup_errors.len(),
        1,
        "expected 1 duplicate slug error, got {len}",
        len = dup_errors.len()
    );
}

#[test]
fn test_duplicate_slug_context_docs() {
    let manifest = valid_manifest();
    let bundle = AdiyutantBundle {
        life_core: None,
        routines: None,
        rules: None,
        checklists: None,
        planning: None,
        context_docs: vec![
            crate::bundle::bundle_models::ContextDocEntry {
                slug: Some("ctx-dup".into()),
                title: "C1".into(),
                content: "a".into(),
            },
            crate::bundle::bundle_models::ContextDocEntry {
                slug: Some("ctx-dup".into()),
                title: "C2".into(),
                content: "b".into(),
            },
        ],
        projects: vec![],
        roadmaps: vec![],
        unknown_files: vec![],
        manifest,
    };

    let report = validate_bundle(&bundle).unwrap();
    let dup_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.code == "duplicate_slug")
        .collect();
    assert_eq!(
        dup_errors.len(),
        1,
        "expected 1 duplicate slug error, got {len}",
        len = dup_errors.len()
    );
}

// ================================================================
// 5. Planning concrete-day deserialisation
// ================================================================

#[test]
fn test_planning_concrete_day_fields_are_dropped() {
    let yaml = r#"
default_mode: "deep"
daily_capacity:
  deep_tasks_max: 2
  light_tasks_max: 3
templates: []
candidate_hints: []
date: "2026-06-23"
tasks:
  - id: "t1"
    title: "concrete"
"#;
    let section: PlanningSection = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(section.default_mode.as_deref(), Some("deep"));
    // The struct has no date/tasks fields, so they are silently dropped.
    // Verify that the parsed result is a valid PlanningSection without concrete-day data.
    assert!(section.templates.is_empty());
    assert!(section.candidate_hints.is_empty());
}

// ================================================================
// 6. Zip path / security validation
// ================================================================

#[test]
fn test_security_max_file_count() {
    let entries: Vec<(String, u64, bool)> = (0..MAX_FILE_COUNT + 1)
        .map(|i| (format!("file{i}.yaml"), 10, false))
        .collect();
    let issues = validate_zip_entries(&entries);
    let too_many: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "too_many_files")
        .collect();
    assert_eq!(too_many.len(), 1, "expected too_many_files error");
}

#[test]
fn test_security_absolute_path_rejected() {
    let entries = vec![("/etc/passwd".to_string(), 100, false)];
    let issues = validate_zip_entries(&entries);
    let unsafe_paths: Vec<_> = issues.iter().filter(|i| i.code == "unsafe_path").collect();
    assert_eq!(unsafe_paths.len(), 1, "expected unsafe_path error");
}

#[test]
fn test_security_path_traversal_rejected() {
    let entries = vec![("../../foo.yaml".to_string(), 100, false)];
    let issues = validate_zip_entries(&entries);
    let traversals: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "path_traversal")
        .collect();
    assert_eq!(traversals.len(), 1, "expected path_traversal error");
}

#[test]
fn test_security_hidden_file_rejected() {
    let entries = vec![(".secret.yaml".to_string(), 100, false)];
    let issues = validate_zip_entries(&entries);
    let hidden: Vec<_> = issues.iter().filter(|i| i.code == "hidden_file").collect();
    assert_eq!(
        hidden.len(),
        1,
        "expected hidden_file error, got {len}",
        len = hidden.len()
    );
}

#[test]
fn test_security_disallowed_extension() {
    let entries = vec![("script.exe".to_string(), 100, false)];
    let issues = validate_zip_entries(&entries);
    let disallowed: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "disallowed_extension")
        .collect();
    assert_eq!(disallowed.len(), 1, "expected disallowed_extension error");
}

#[test]
fn test_security_yaml_file_size_limit() {
    let entries = vec![("big.yaml".to_string(), MAX_YAML_FILE_SIZE + 1, false)];
    let issues = validate_zip_entries(&entries);
    let oversized: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "file_too_large")
        .collect();
    assert_eq!(oversized.len(), 1, "expected file_too_large for yaml");
}

#[test]
fn test_security_md_file_size_limit() {
    let entries = vec![("big.md".to_string(), MAX_MD_FILE_SIZE + 1, false)];
    let issues = validate_zip_entries(&entries);
    let oversized: Vec<_> = issues
        .iter()
        .filter(|i| i.code == "file_too_large")
        .collect();
    assert_eq!(oversized.len(), 1, "expected file_too_large for md");
}

#[test]
fn test_security_clean_entry_passes() {
    let entries = vec![("manifest.yaml".to_string(), 100, false)];
    let issues = validate_zip_entries(&entries);
    assert!(
        issues.is_empty(),
        "expected no issues for clean entry, got {len}",
        len = issues.len()
    );
}

// ================================================================
// 7. Import planner tests
// ================================================================

fn make_test_bundle_for_planner(lc_slug: Option<&str>) -> AdiyutantBundle {
    let mut sections = HashMap::new();
    sections.insert("life_core".into(), "life-core.yaml".into());
    let manifest = Manifest {
        schema_version: BUNDLE_SCHEMA_VERSION.to_string(),
        bundle_id: "planner-test".into(),
        title: "Planner Test".into(),
        created_at: "2026-06-22T10:00:00Z".into(),
        capabilities: HashMap::new(),
        sections,
    };
    let context_docs = if let Some(slug) = lc_slug {
        vec![LifeCoreContextDoc {
            slug: Some(slug.into()),
            title: slug.into(),
            doc_type: "core".into(),
            content: "test content".into(),
        }]
    } else {
        vec![]
    };
    AdiyutantBundle {
        life_core: Some(LifeCoreSection {
            profile: None,
            context_documents: context_docs,
        }),
        routines: None,
        rules: None,
        checklists: None,
        planning: None,
        context_docs: vec![],
        projects: vec![],
        roadmaps: vec![],
        unknown_files: vec![],
        manifest,
    }
}

fn seed_store_with_slug(store: &MockStore, slug: &str) {
    let doc = ContextDocument::with_source(
        ContextDocumentType::Core,
        slug.into(),
        "existing content".into(),
        Some(slug.into()),
        None,
    );
    store.insert_context_document(&doc).unwrap();
}

#[test]
fn test_planner_append_all_create() {
    let store = MockStore::new();
    let bundle = make_test_bundle_for_planner(Some("new-doc"));
    let plan = compute_import_plan(&bundle, ImportMode::Append, &store).unwrap();
    let create_actions: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == BundleActionKind::Create)
        .collect();
    assert_eq!(
        create_actions.len(),
        1,
        "expected 1 create in append for new slug, got {len}",
        len = create_actions.len()
    );
}

#[test]
fn test_planner_append_existing_slug_skip() {
    let store = MockStore::new();
    seed_store_with_slug(&store, "existing-doc");
    let bundle = make_test_bundle_for_planner(Some("existing-doc"));
    let plan = compute_import_plan(&bundle, ImportMode::Append, &store).unwrap();
    let skip_actions: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == BundleActionKind::SkipExisting)
        .collect();
    assert_eq!(
        skip_actions.len(),
        1,
        "expected 1 skip_existing in append for existing slug, got {len}",
        len = skip_actions.len()
    );
}

#[test]
fn test_planner_merge_existing_slug_update() {
    let store = MockStore::new();
    seed_store_with_slug(&store, "existing-doc");
    let bundle = make_test_bundle_for_planner(Some("existing-doc"));
    let plan = compute_import_plan(&bundle, ImportMode::Merge, &store).unwrap();
    let update_actions: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == BundleActionKind::Update)
        .collect();
    assert_eq!(
        update_actions.len(),
        1,
        "expected 1 update in merge for existing slug, got {len}",
        len = update_actions.len()
    );
}

#[test]
fn test_planner_merge_new_slug_create() {
    let store = MockStore::new();
    let bundle = make_test_bundle_for_planner(Some("brand-new-doc"));
    let plan = compute_import_plan(&bundle, ImportMode::Merge, &store).unwrap();
    let create_actions: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == BundleActionKind::Create)
        .collect();
    assert_eq!(
        create_actions.len(),
        1,
        "expected 1 create in merge for new slug, got {len}",
        len = create_actions.len()
    );
}

// ================================================================
// 8. Export serialisation tests
// ================================================================

#[test]
fn test_export_manifest_is_valid_yaml() {
    let store = MockStore::new();
    let dir = std::env::temp_dir().join(format!(
        "adiyutant_test_export_manifest_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let report = export_bundle(&dir, &store).unwrap();
    assert_eq!(report.status, crate::bundle::dto::BundleStatus::Valid);

    let manifest_path = dir.join("manifest.yaml");
    assert!(
        manifest_path.exists(),
        "manifest.yaml should exist after export"
    );

    let manifest_str = std::fs::read_to_string(&manifest_path).unwrap();
    let parsed: Manifest = Manifest::from_yaml(&manifest_str).unwrap();
    assert_eq!(parsed.schema_version, BUNDLE_SCHEMA_VERSION);
    assert!(!parsed.bundle_id.is_empty());
    assert!(!parsed.title.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_export_section_count_matches() {
    let store = MockStore::new();

    // Add context docs and checklist templates so they are counted.
    let doc = ContextDocument::new(
        ContextDocumentType::Core,
        "Test Core".into(),
        "# Core".into(),
    );
    store.insert_context_document(&doc).unwrap();
    let template = ChecklistTemplate::with_slug(
        "Test Checklist".into(),
        "morning".into(),
        Some("test-checklist".into()),
    );
    store.insert_checklist_template(&template).unwrap();

    let dir = std::env::temp_dir().join(format!(
        "adiyutant_test_export_count_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let report = export_bundle(&dir, &store).unwrap();
    // life_core(1) + routines(0→0) + rules(0→0) + checklists(1) + planning(0→0)
    // = context_docs non-empty: 1 + 0 + 3 = ... actually:
    // section_count = (context_docs.is_empty() ? 0 : 1) + (checklist_templates.is_empty() ? 0 : 1) + 3
    // = 1 + 1 + 3 = 5
    assert_eq!(
        report.section_count, 5,
        "expected 5 sections (life_core+checklists+3 always)"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_export_section_files_exist() {
    let store = MockStore::new();
    let dir = std::env::temp_dir().join(format!(
        "adiyutant_test_export_files_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    export_bundle(&dir, &store).unwrap();

    for expected in &[
        "manifest.yaml",
        "life-core.yaml",
        "routines.yaml",
        "rules.yaml",
        "checklists.yaml",
        "planning.yaml",
    ] {
        assert!(
            dir.join(expected).exists(),
            "expected {expected} to exist after export"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}
