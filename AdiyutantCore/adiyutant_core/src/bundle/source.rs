//! `BundleSource` — input form adapters for portable bundle import.
//!
//! All forms parse into the same `AdiyutantBundle`. The Android client
//! will primarily use `ZipBytes`; the CLI and tests primarily use
//! `Directory` and `ZipFile`.

use std::path::PathBuf;

/// Input form for a portable bundle.
///
/// Three adapters are exposed so that:
/// * the CLI can pass a directory or a `.adiyutant.zip` path;
/// * Android can pass raw zip bytes from a file picker without owning
///   the on-disk write step;
/// * tests can pass synthetic bytes.
#[derive(Debug, Clone)]
pub enum BundleSource {
    /// Unpacked bundle directory.
    Directory(PathBuf),
    /// `.adiyutant.zip` file on disk.
    ZipFile(PathBuf),
    /// Raw zip bytes (e.g. from Android file picker).
    ZipBytes(Vec<u8>),
}

impl BundleSource {
    /// Human-readable label for diagnostics.
    pub fn kind_label(&self) -> &'static str {
        match self {
            BundleSource::Directory(_) => "directory",
            BundleSource::ZipFile(_) => "zip_file",
            BundleSource::ZipBytes(_) => "zip_bytes",
        }
    }
}

// Stage 0 stub — Unit B will fill in real parser dispatch.
#[doc(hidden)]
pub fn _stage0_stub() {}
