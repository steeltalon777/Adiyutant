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
mod tests;

use crate::error::CoreError;
use crate::store::Store;
use crate::time_provider::{RealTimeProvider, TimeProvider};

/// Application-layer facade / use-case boundary.
///
/// All public methods return stable DTOs. Consumers (CLI, Android)
/// must never access Store or domain internals directly.
pub struct AdiyutantCoreService {
    pub(crate) store: Box<dyn Store<Error = CoreError>>,
    pub(crate) time: Box<dyn TimeProvider>,
}

impl AdiyutantCoreService {
    pub fn new(store: Box<dyn Store<Error = CoreError>>) -> Self {
        Self {
            store,
            time: Box::new(RealTimeProvider),
        }
    }

    pub fn with_time(
        store: Box<dyn Store<Error = CoreError>>,
        time: Box<dyn TimeProvider>,
    ) -> Self {
        Self { store, time }
    }
}
