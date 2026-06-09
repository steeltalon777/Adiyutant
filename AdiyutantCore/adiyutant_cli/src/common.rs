use std::path::PathBuf;

use adiyutant_core::error::CoreError;
use adiyutant_core::service::AdiyutantCoreService;
use adiyutant_core::store::Store;
use adiyutant_store::SqliteStore;

/// Determine the SQLite database path.
///
/// Priority:
/// 1. `ADIYUTANT_DB_PATH` environment variable
/// 2. `~/.adiyutant/adiyutant.db` (fallback)
pub fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("ADIYUTANT_DB_PATH") {
        return PathBuf::from(p);
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".adiyutant").join("adiyutant.db")
}

/// Initialize the `SqliteStore`: create directory, open DB, run migration.
pub fn init_store() -> Result<SqliteStore, CoreError> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CoreError::Storage(e.to_string()))?;
    }
    let store = SqliteStore::new(path.to_str().unwrap_or("adiyutant.db"))?;
    store.migrate()?;
    Ok(store)
}

/// Build the application facade from a store.
pub fn build_facade(store: SqliteStore) -> AdiyutantCoreService {
    AdiyutantCoreService::new(Box::new(store))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_path_uses_env_var() {
        // SAFETY: test-only env var modification
        unsafe {
            std::env::set_var("ADIYUTANT_DB_PATH", "/tmp/test.db");
        }
        let path = db_path();
        assert_eq!(path, PathBuf::from("/tmp/test.db"));
        // SAFETY: test-only env var removal
        unsafe {
            std::env::remove_var("ADIYUTANT_DB_PATH");
        }
    }

    #[test]
    fn db_path_falls_back_to_home() {
        // SAFETY: test-only env var removal
        unsafe {
            std::env::remove_var("ADIYUTANT_DB_PATH");
        }
        let path = db_path();
        // Should contain ".adiyutant/adiyutant.db"
        assert!(path.to_string_lossy().contains(".adiyutant/adiyutant.db"));
    }
}
