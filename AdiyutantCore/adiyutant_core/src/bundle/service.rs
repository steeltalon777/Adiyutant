use crate::bundle::apply::apply_bundle;
use crate::bundle::bundle_models::ImportMode;
use crate::bundle::dto::{
    BundleApplyReportDto, BundleExportReportDto, BundlePreviewDto, BundleValidationReportDto,
};
use crate::bundle::export::export_bundle;
use crate::bundle::parser::parse_bundle;
use crate::bundle::planner::compute_import_plan;
use crate::bundle::source::BundleSource;
use crate::bundle::validator::validate_bundle;
use crate::error::CoreResult;
use crate::store::Store;
use std::path::Path;

/// Bundle operations facade for `AdiyutantCoreService`.
pub struct BundleService;

impl Default for BundleService {
    fn default() -> Self {
        Self
    }
}

impl BundleService {
    pub fn new() -> Self {
        Self
    }

    /// Validate a bundle without any store access.
    pub fn validate(&self, source: &BundleSource) -> CoreResult<BundleValidationReportDto> {
        let bundle = parse_bundle(source)?;
        validate_bundle(&bundle)
    }

    /// Preview an import without writing.
    pub fn preview(
        &self,
        source: &BundleSource,
        mode: ImportMode,
        store: &dyn Store<Error = crate::error::CoreError>,
    ) -> CoreResult<BundlePreviewDto> {
        let bundle = parse_bundle(source)?;
        let validation = validate_bundle(&bundle)?;
        if validation.status == crate::bundle::dto::BundleStatus::Invalid {
            return Ok(BundlePreviewDto {
                bundle_id: validation.bundle_id,
                schema_version: validation.schema_version,
                title: validation.title,
                status: crate::bundle::dto::BundleStatus::Invalid,
                errors: validation.errors,
                warnings: validation.warnings,
                sections: validation.sections,
                actions: vec![],
                mode: mode.to_string(),
            });
        }
        compute_import_plan(&bundle, mode, store)
    }

    /// Apply (import) a bundle to the store.
    pub fn apply(
        &self,
        source: &BundleSource,
        mode: ImportMode,
        store: &dyn Store<Error = crate::error::CoreError>,
    ) -> CoreResult<BundleApplyReportDto> {
        let bundle = parse_bundle(source)?;
        let validation = validate_bundle(&bundle)?;
        if validation.status == crate::bundle::dto::BundleStatus::Invalid {
            return Ok(BundleApplyReportDto {
                bundle_id: validation.bundle_id,
                schema_version: validation.schema_version,
                title: validation.title,
                status: crate::bundle::dto::BundleStatus::Failed,
                errors: validation.errors,
                warnings: validation.warnings,
                sections: validation.sections,
                actions: vec![],
                mode: mode.to_string(),
                import_run_id: String::new(),
            });
        }
        apply_bundle(&bundle, mode, store)
    }

    /// Export supported sections from the store.
    pub fn export_bundle(
        &self,
        target: &Path,
        store: &dyn Store<Error = crate::error::CoreError>,
    ) -> CoreResult<BundleExportReportDto> {
        export_bundle(target, store)
    }
}
