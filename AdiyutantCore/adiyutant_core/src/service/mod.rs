mod basic_entities;
mod checkin;
mod checklists;
mod checkpoints;
mod helpers;
mod journal;
mod planning;
mod suggestions;
mod today;

#[cfg(test)]
pub mod tests;

use std::path::Path;

use crate::bundle::bundle_models::ImportMode;
use crate::bundle::dto::{
    BundleApplyReportDto, BundleExportReportDto, BundlePreviewDto, BundleValidationReportDto,
};
use crate::bundle::source::BundleSource;
use crate::error::{CoreError, CoreResult};
use crate::store::Store;
use crate::time_provider::{RealTimeProvider, TimeProvider};

/// Application-layer facade / use-case boundary.
///
/// All public methods return stable DTOs. Consumers (CLI, Android)
/// must never access Store or domain internals directly.
pub struct AdiyutantCoreService {
    pub(crate) store: Box<dyn Store<Error = CoreError>>,
    pub(crate) time: Box<dyn TimeProvider>,
    pub(crate) bundle: crate::bundle::service::BundleService,
}

impl AdiyutantCoreService {
    pub fn new(store: Box<dyn Store<Error = CoreError>>) -> Self {
        Self {
            store,
            time: Box::new(RealTimeProvider),
            bundle: crate::bundle::service::BundleService::new(),
        }
    }

    pub fn with_time(
        store: Box<dyn Store<Error = CoreError>>,
        time: Box<dyn TimeProvider>,
    ) -> Self {
        Self {
            store,
            time,
            bundle: crate::bundle::service::BundleService::new(),
        }
    }

    // ── Bundle operations ──

    pub fn validate_bundle(&self, source: &BundleSource) -> CoreResult<BundleValidationReportDto> {
        self.bundle.validate(source)
    }

    pub fn preview_bundle_import(
        &self,
        source: &BundleSource,
        mode: ImportMode,
    ) -> CoreResult<BundlePreviewDto> {
        self.bundle.preview(source, mode, &*self.store)
    }

    pub fn apply_bundle_import(
        &self,
        source: &BundleSource,
        mode: ImportMode,
    ) -> CoreResult<BundleApplyReportDto> {
        self.bundle.apply(source, mode, &*self.store)
    }

    pub fn export_bundle(&self, target: &Path) -> CoreResult<BundleExportReportDto> {
        self.bundle.export_bundle(target, &*self.store)
    }
}
