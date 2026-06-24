use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::bundle::bundle_models::*;
use crate::bundle::manifest::Manifest;
use crate::bundle::security;
use crate::bundle::source::BundleSource;
use crate::error::{CoreError, CoreResult};

pub fn parse_bundle(source: &BundleSource) -> CoreResult<AdiyutantBundle> {
    match source {
        BundleSource::Directory(path) => parse_directory_bundle(path),
        BundleSource::ZipFile(path) => {
            let bytes = fs::read(path)
                .map_err(|e| CoreError::NotFound(format!("Cannot read zip file: {e}")))?;
            parse_zip_bundle(&bytes)
        }
        BundleSource::ZipBytes(bytes) => parse_zip_bundle(bytes),
    }
}

fn parse_directory_bundle(path: &Path) -> CoreResult<AdiyutantBundle> {
    let manifest_yaml = fs::read_to_string(path.join("manifest.yaml"))
        .map_err(|e| CoreError::NotFound(format!("Cannot read manifest.yaml: {e}")))?;
    let manifest = Manifest::from_yaml(&manifest_yaml)?;

    let life_core =
        parse_optional_yaml::<LifeCoreSection>(path, manifest.section_path("life_core"))?;
    let routines = parse_optional_yaml::<RoutinesSection>(path, manifest.section_path("routines"))?;
    let rules = parse_optional_yaml::<RulesSection>(path, manifest.section_path("rules"))?;
    let checklists =
        parse_optional_yaml::<ChecklistsSection>(path, manifest.section_path("checklists"))?;
    let planning = parse_optional_yaml::<PlanningSection>(path, manifest.section_path("planning"))?;

    let context_docs = parse_context_docs_dir(path, manifest.section_path("context_dir"))?;
    let projects =
        parse_yaml_files_dir::<BundleProject>(path, manifest.section_path("projects_dir"))?;
    let roadmaps =
        parse_yaml_files_dir::<BundleRoadmap>(path, manifest.section_path("roadmaps_dir"))?;

    let unknown_files = find_unknown_files_dir(path, &manifest)?;

    Ok(AdiyutantBundle {
        manifest,
        life_core,
        routines,
        rules,
        checklists,
        planning,
        context_docs,
        projects,
        roadmaps,
        unknown_files,
    })
}

fn parse_zip_bundle(bytes: &[u8]) -> CoreResult<AdiyutantBundle> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| CoreError::InvalidInput(format!("invalid zip archive: {e}")))?;

    let mut raw_entries: Vec<(String, Vec<u8>, bool)> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| CoreError::InvalidInput(format!("failed to read zip entry {i}: {e}")))?;
        let name = file.name().to_string();

        if file.is_symlink() {
            return Err(CoreError::InvalidInput(format!(
                "Zip contains symlink which is not allowed: {name}"
            )));
        }

        let is_dir = file.is_dir();
        let mut data = Vec::new();
        let size = file.size();
        data.reserve(size as usize);
        file.read_to_end(&mut data).map_err(|e| {
            CoreError::InvalidInput(format!("failed to read zip entry '{name}': {e}"))
        })?;
        raw_entries.push((name, data, is_dir));
    }

    let security_entries: Vec<(String, u64, bool)> = raw_entries
        .iter()
        .map(|(name, data, is_dir)| (name.clone(), data.len() as u64, *is_dir))
        .collect();
    let sec_issues = security::validate_zip_entries(&security_entries);
    let sec_errors: Vec<&crate::bundle::dto::BundleIssueDto> = sec_issues
        .iter()
        .filter(|i| i.severity == "error")
        .collect();
    if !sec_errors.is_empty() {
        let msgs: Vec<String> = sec_errors.iter().map(|i| i.message.clone()).collect();
        return Err(CoreError::InvalidInput(format!(
            "zip security check failed: {}",
            msgs.join("; ")
        )));
    }

    let mut entry_map: HashMap<String, Vec<u8>> = HashMap::new();
    for (name, data, _is_dir) in raw_entries {
        entry_map.insert(name, data);
    }

    let manifest_data = entry_map
        .get("manifest.yaml")
        .ok_or_else(|| CoreError::InvalidInput("manifest.yaml not found in bundle".into()))?;
    let manifest_str = std::str::from_utf8(manifest_data)
        .map_err(|e| CoreError::InvalidInput(format!("manifest.yaml is not valid UTF-8: {e}")))?;
    let manifest = Manifest::from_yaml(manifest_str)?;

    let life_core =
        parse_entry_yaml::<LifeCoreSection>(&entry_map, manifest.section_path("life_core"))?;
    let routines =
        parse_entry_yaml::<RoutinesSection>(&entry_map, manifest.section_path("routines"))?;
    let rules = parse_entry_yaml::<RulesSection>(&entry_map, manifest.section_path("rules"))?;
    let checklists =
        parse_entry_yaml::<ChecklistsSection>(&entry_map, manifest.section_path("checklists"))?;
    let planning =
        parse_entry_yaml::<PlanningSection>(&entry_map, manifest.section_path("planning"))?;

    let context_docs =
        parse_context_docs_entries(&entry_map, manifest.section_path("context_dir"))?;
    let projects = parse_yaml_collection_entries::<BundleProject>(
        &entry_map,
        manifest.section_path("projects_dir"),
    )?;
    let roadmaps = parse_yaml_collection_entries::<BundleRoadmap>(
        &entry_map,
        manifest.section_path("roadmaps_dir"),
    )?;

    let unknown_files = find_unknown_files_entries(&entry_map, &manifest)?;

    Ok(AdiyutantBundle {
        manifest,
        life_core,
        routines,
        rules,
        checklists,
        planning,
        context_docs,
        projects,
        roadmaps,
        unknown_files,
    })
}

fn parse_optional_yaml<T>(base: &Path, rel_path: Option<&str>) -> CoreResult<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    match rel_path {
        Some(p) => {
            let full_path = base.join(p);
            if full_path.exists() {
                let content = fs::read_to_string(&full_path).map_err(|e| {
                    CoreError::InvalidInput(format!("Cannot read {}: {e}", full_path.display()))
                })?;
                let parsed: T = serde_yaml::from_str(&content).map_err(|e| {
                    CoreError::InvalidInput(format!(
                        "YAML parse error in {}: {e}",
                        full_path.display()
                    ))
                })?;
                Ok(Some(parsed))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn parse_entry_yaml<T>(
    entries: &HashMap<String, Vec<u8>>,
    path: Option<&str>,
) -> CoreResult<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    match path {
        Some(p) => {
            if let Some(data) = entries.get(p) {
                let content = std::str::from_utf8(data)
                    .map_err(|e| CoreError::InvalidInput(format!("{p} is not valid UTF-8: {e}")))?;
                let parsed: T = serde_yaml::from_str(content).map_err(|e| {
                    CoreError::InvalidInput(format!("YAML parse error in {p}: {e}"))
                })?;
                Ok(Some(parsed))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn parse_context_docs_dir(base: &Path, rel_dir: Option<&str>) -> CoreResult<Vec<ContextDocEntry>> {
    let mut docs = Vec::new();
    if let Some(dir) = rel_dir {
        let dir_path = base.join(dir);
        if dir_path.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(&dir_path)
                .map_err(|e| {
                    CoreError::InvalidInput(format!("Cannot read dir {}: {e}", dir_path.display()))
                })?
                .filter_map(|r| r.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                let content = fs::read_to_string(&path).map_err(|e| {
                    CoreError::InvalidInput(format!("Cannot read {}: {e}", path.display()))
                })?;
                let slug = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let title = slug.clone().unwrap_or_default();
                docs.push(ContextDocEntry {
                    slug,
                    title,
                    content,
                });
            }
        }
    }
    Ok(docs)
}

fn parse_context_docs_entries(
    entries: &HashMap<String, Vec<u8>>,
    rel_dir: Option<&str>,
) -> CoreResult<Vec<ContextDocEntry>> {
    let mut docs = Vec::new();
    if let Some(dir) = rel_dir {
        let prefix = dir.trim_end_matches('/').to_string() + "/";
        let mut file_names: Vec<&String> = entries
            .keys()
            .filter(|k| k.starts_with(&prefix) && k.ends_with(".md"))
            .collect();
        file_names.sort();

        for name in file_names {
            if let Some(data) = entries.get(name) {
                let content = std::str::from_utf8(data)
                    .map_err(|e| {
                        CoreError::InvalidInput(format!("{name} is not valid UTF-8: {e}"))
                    })?
                    .to_string();
                let stem = std::path::Path::new(name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let title = stem.clone().unwrap_or_default();
                docs.push(ContextDocEntry {
                    slug: stem,
                    title,
                    content,
                });
            }
        }
    }
    Ok(docs)
}

fn parse_yaml_files_dir<T>(base: &Path, rel_dir: Option<&str>) -> CoreResult<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let mut items = Vec::new();
    if let Some(dir) = rel_dir {
        let dir_path = base.join(dir);
        if dir_path.is_dir() {
            let mut file_paths: Vec<_> = fs::read_dir(&dir_path)
                .map_err(|e| {
                    CoreError::InvalidInput(format!("Cannot read dir {}: {e}", dir_path.display()))
                })?
                .filter_map(|r| r.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "yaml" || ext == "yml")
                        .unwrap_or(false)
                })
                .map(|e| e.path())
                .collect::<Vec<_>>();
            file_paths.sort();

            for path in file_paths {
                let content = fs::read_to_string(&path).map_err(|e| {
                    CoreError::InvalidInput(format!("Cannot read {}: {e}", path.display()))
                })?;
                let parsed: T = serde_yaml::from_str(&content).map_err(|e| {
                    CoreError::InvalidInput(format!("YAML parse error in {}: {e}", path.display()))
                })?;
                items.push(parsed);
            }
        }
    }
    Ok(items)
}

fn parse_yaml_collection_entries<T>(
    entries: &HashMap<String, Vec<u8>>,
    rel_dir: Option<&str>,
) -> CoreResult<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let mut items = Vec::new();
    if let Some(dir) = rel_dir {
        let prefix = dir.trim_end_matches('/').to_string() + "/";
        let mut file_names: Vec<&String> = entries
            .keys()
            .filter(|k| k.starts_with(&prefix) && (k.ends_with(".yaml") || k.ends_with(".yml")))
            .collect();
        file_names.sort();

        for name in file_names {
            if let Some(data) = entries.get(name) {
                let content = std::str::from_utf8(data).map_err(|e| {
                    CoreError::InvalidInput(format!("{name} is not valid UTF-8: {e}"))
                })?;
                let parsed: T = serde_yaml::from_str(content).map_err(|e| {
                    CoreError::InvalidInput(format!("YAML parse error in {name}: {e}"))
                })?;
                items.push(parsed);
            }
        }
    }
    Ok(items)
}

fn find_unknown_files_dir(base: &Path, manifest: &Manifest) -> CoreResult<Vec<String>> {
    let known_prefixes = build_known_prefixes(manifest);
    let mut unknown = Vec::new();
    collect_unknown(base, base, &known_prefixes, &mut unknown)?;
    Ok(unknown)
}

fn build_known_prefixes(manifest: &Manifest) -> Vec<String> {
    let mut prefixes: Vec<String> = Vec::new();
    prefixes.push("manifest.yaml".to_string());
    for path_str in manifest.sections.values() {
        let trimmed = path_str.trim_end_matches('/');
        prefixes.push(path_str.clone());
        // If it's a directory (no recognizable file extension), also add with /
        if !trimmed.contains('.') || path_str.ends_with('/') {
            prefixes.push(format!("{trimmed}/"));
        }
    }
    prefixes
}

fn collect_unknown(
    base: &Path,
    dir: &Path,
    known_prefixes: &[String],
    result: &mut Vec<String>,
) -> CoreResult<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)
            .map_err(|e| CoreError::InvalidInput(format!("Cannot read dir: {e}")))?
        {
            let entry =
                entry.map_err(|e| CoreError::InvalidInput(format!("Cannot read entry: {e}")))?;
            let path = entry.path();
            let rel_path = path
                .strip_prefix(base)
                .map_err(|_| CoreError::Internal("path resolution error".into()))?
                .to_string_lossy()
                .to_string();

            if path.is_dir() {
                collect_unknown(base, &path, known_prefixes, result)?;
            } else {
                let is_known = known_prefixes
                    .iter()
                    .any(|p| rel_path == *p || rel_path.starts_with(p));
                if !is_known {
                    result.push(rel_path);
                }
            }
        }
    }
    Ok(())
}

fn find_unknown_files_entries(
    entries: &HashMap<String, Vec<u8>>,
    manifest: &Manifest,
) -> CoreResult<Vec<String>> {
    let known = build_known_prefixes(manifest);

    let mut unknown: Vec<String> = entries
        .keys()
        .filter(|name| {
            !name.ends_with('/') && !known.iter().any(|p| *name == p || name.starts_with(p))
        })
        .cloned()
        .collect();
    unknown.sort();
    Ok(unknown)
}
