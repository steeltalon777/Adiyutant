use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum BundleSource {
    Directory(PathBuf),
    ZipFile(PathBuf),
    ZipBytes(Vec<u8>),
}

impl BundleSource {
    pub fn resolve_path(&self) -> Option<&Path> {
        match self {
            BundleSource::Directory(p) | BundleSource::ZipFile(p) => Some(p.as_path()),
            BundleSource::ZipBytes(_) => None,
        }
    }

    pub fn read_zip_bytes(&self) -> Option<Vec<u8>> {
        match self {
            BundleSource::ZipBytes(bytes) => Some(bytes.clone()),
            BundleSource::ZipFile(path) => std::fs::read(path).ok(),
            BundleSource::Directory(_) => None,
        }
    }
}
