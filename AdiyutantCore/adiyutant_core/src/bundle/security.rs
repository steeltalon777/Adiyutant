use std::path::Path;

use crate::bundle::dto::BundleIssueDto;
use crate::bundle::manifest::Manifest;

pub const MAX_FILE_COUNT: usize = 200;
pub const MAX_UNCOMPRESSED_SIZE: u64 = 10 * 1024 * 1024;
pub const MAX_YAML_FILE_SIZE: u64 = 1024 * 1024;
pub const MAX_MD_FILE_SIZE: u64 = 2 * 1024 * 1024;
pub const ALLOWED_EXTENSIONS: &[&str] = &[".yaml", ".yml", ".md"];

pub fn validate_zip_entries(entries: &[(String, u64, bool)]) -> Vec<BundleIssueDto> {
    let mut issues = Vec::new();
    let mut total_size: u64 = 0;

    if entries.len() > MAX_FILE_COUNT {
        issues.push(BundleIssueDto {
            severity: "error".into(),
            section: "archive".into(),
            code: "too_many_files".into(),
            message: format!(
                "Zip contains {} files (max {MAX_FILE_COUNT})",
                entries.len()
            ),
            field: None,
        });
    }

    for (path, size, is_dir) in entries {
        total_size += size;

        if path.starts_with('/') {
            issues.push(BundleIssueDto {
                severity: "error".into(),
                section: "archive".into(),
                code: "unsafe_path".into(),
                message: format!("Absolute path not allowed: {path}"),
                field: Some(path.clone()),
            });
        }

        if path.contains("..") {
            issues.push(BundleIssueDto {
                severity: "error".into(),
                section: "archive".into(),
                code: "path_traversal".into(),
                message: format!("Path traversal detected: {path}"),
                field: Some(path.clone()),
            });
        }

        let p = Path::new(path);
        for component in p.components() {
            if let std::path::Component::Normal(name) = component
                && let Some(name_str) = name.to_str()
                && name_str.starts_with('.')
                && name_str != "."
                && name_str != ".."
            {
                issues.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "archive".into(),
                    code: "hidden_file".into(),
                    message: format!("Hidden file not allowed: {path}"),
                    field: Some(path.clone()),
                });
            }
        }

        if !is_dir {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();

            if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
                issues.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "archive".into(),
                    code: "disallowed_extension".into(),
                    message: format!("File extension '{ext}' not allowed for: {path}"),
                    field: Some(path.clone()),
                });
            }

            if (ext == ".yaml" || ext == ".yml") && *size > MAX_YAML_FILE_SIZE {
                issues.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "archive".into(),
                    code: "file_too_large".into(),
                    message: format!("YAML file exceeds {MAX_YAML_FILE_SIZE} bytes: {path}"),
                    field: Some(path.clone()),
                });
            }

            if ext == ".md" && *size > MAX_MD_FILE_SIZE {
                issues.push(BundleIssueDto {
                    severity: "error".into(),
                    section: "archive".into(),
                    code: "file_too_large".into(),
                    message: format!("MD file exceeds {MAX_MD_FILE_SIZE} bytes: {path}"),
                    field: Some(path.clone()),
                });
            }
        }
    }

    if total_size > MAX_UNCOMPRESSED_SIZE {
        issues.push(BundleIssueDto {
            severity: "error".into(),
            section: "archive".into(),
            code: "total_size_exceeded".into(),
            message: format!(
                "Total uncompressed size {total_size} exceeds max {MAX_UNCOMPRESSED_SIZE}"
            ),
            field: None,
        });
    }

    issues
}

pub fn check_bundle_structure(bundle_dir: &Path, manifest: &Manifest) -> Vec<BundleIssueDto> {
    let mut issues = Vec::new();

    let manifest_path = bundle_dir.join("manifest.yaml");
    if !manifest_path.exists() {
        issues.push(BundleIssueDto {
            severity: "error".into(),
            section: "archive".into(),
            code: "missing_manifest".into(),
            message: "Bundle directory does not contain manifest.yaml".into(),
            field: None,
        });
    }

    for (name, section_path_str) in &manifest.sections {
        let full_path = bundle_dir.join(section_path_str);
        if !full_path.exists() {
            issues.push(BundleIssueDto {
                severity: "warning".into(),
                section: name.clone(),
                code: "missing_section".into(),
                message: format!("Declared section '{name}' not found at path: {section_path_str}"),
                field: Some(section_path_str.clone()),
            });
        }
    }

    issues
}
