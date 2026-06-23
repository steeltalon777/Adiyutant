use crate::bundle::bundle_models::{AdiyutantBundle, ImportMode};
use crate::bundle::dto::{BundleActionKind, BundleApplyReportDto, BundleStatus};
use crate::bundle::planner::compute_import_plan;
use crate::datetime::AdiyutantDateTime;
use crate::error::{CoreError, CoreResult};
use crate::id::Id;
use crate::model::import_run::ImportRun;
use crate::store::Store;

/// Apply a validated bundle's supported sections to the store.
///
/// Must use the same internal planner as `preview`.
/// Applies supported sections in a single transaction.
pub fn apply_bundle(
    bundle: &AdiyutantBundle,
    mode: ImportMode,
    store: &dyn Store<Error = CoreError>,
) -> CoreResult<BundleApplyReportDto> {
    let plan = compute_import_plan(bundle, mode, store)?;

    let has_errors = plan.status == BundleStatus::Invalid
        || plan
            .actions
            .iter()
            .any(|a| a.kind == BundleActionKind::Error);

    if has_errors {
        let import_run_id = Id::<ImportRun>::new().value().to_string();
        return Ok(BundleApplyReportDto {
            bundle_id: plan.bundle_id,
            schema_version: plan.schema_version,
            title: plan.title,
            status: BundleStatus::Failed,
            errors: plan.errors,
            warnings: plan.warnings,
            sections: plan.sections,
            actions: plan.actions,
            mode: plan.mode,
            import_run_id,
        });
    }

    let bundle_id = plan.bundle_id.clone();
    let schema_version = plan.schema_version.clone();
    let title = plan.title.clone();
    let all_actions = plan.actions.clone();
    let all_sections = plan.sections.clone();
    let all_warnings = plan.warnings.clone();
    let mode_str = plan.mode.clone();

    let run = ImportRun {
        id: Id::new(),
        bundle_id: bundle_id.clone(),
        bundle_title: title.clone(),
        schema_version: schema_version.clone(),
        mode: mode_str.clone(),
        status: "applied".into(),
        started_at: AdiyutantDateTime::now(),
        finished_at: AdiyutantDateTime::now(),
        summary_json: serde_json::to_string(&all_actions).unwrap_or_default(),
    };
    let import_run_id = run.id.value().to_string();
    store.bundle_apply_composite(bundle, mode, &run)?;

    Ok(BundleApplyReportDto {
        bundle_id,
        schema_version,
        title,
        status: BundleStatus::Applied,
        errors: vec![],
        warnings: all_warnings,
        sections: all_sections,
        actions: all_actions,
        mode: mode_str,
        import_run_id,
    })
}
