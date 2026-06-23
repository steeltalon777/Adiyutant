use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::bundle::dto::BundleIssueDto;
use crate::error::{CoreError, CoreResult};

pub const BUNDLE_SCHEMA_VERSION: &str = "adiyutant.bundle.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    PreviewOnly,
    ApplyExport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: String,
    pub bundle_id: String,
    pub title: String,
    pub created_at: String,
    #[serde(default)]
    pub capabilities: HashMap<String, String>,
    pub sections: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRaw {
    pub schema_version: String,
    pub bundle_id: String,
    pub title: String,
    pub created_at: String,
    #[serde(default)]
    pub capabilities: HashMap<String, String>,
    #[serde(default)]
    pub sections: HashMap<String, String>,
}

impl Manifest {
    pub fn from_yaml(yaml_str: &str) -> CoreResult<Manifest> {
        let raw: ManifestRaw = serde_yaml::from_str(yaml_str)
            .map_err(|e| CoreError::InvalidInput(format!("invalid manifest YAML: {e}")))?;
        let manifest = Manifest {
            schema_version: raw.schema_version,
            bundle_id: raw.bundle_id,
            title: raw.title,
            created_at: raw.created_at,
            capabilities: raw.capabilities,
            sections: raw.sections,
        };
        manifest.ensure_valid()?;
        Ok(manifest)
    }

    pub fn from_raw_yaml(yaml_str: &str) -> CoreResult<(Manifest, Vec<BundleIssueDto>)> {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .map_err(|e| CoreError::InvalidInput(format!("invalid manifest YAML: {e}")))?;

        let mut warnings = Vec::new();

        let expected_fields = [
            "schema_version",
            "bundle_id",
            "title",
            "created_at",
            "capabilities",
            "sections",
        ];

        if let serde_yaml::Value::Mapping(ref map) = value {
            for key in map.keys() {
                if let serde_yaml::Value::String(field_name) = key
                    && !expected_fields.contains(&field_name.as_str())
                {
                    warnings.push(BundleIssueDto {
                        severity: "warning".into(),
                        section: "manifest".into(),
                        code: "unknown_field".into(),
                        message: format!("Unknown manifest field: {field_name}"),
                        field: Some(field_name.clone()),
                    });
                }
            }
        }

        let raw: ManifestRaw = serde_yaml::from_value(value)
            .map_err(|e| CoreError::InvalidInput(format!("invalid manifest YAML: {e}")))?;

        let manifest = Manifest {
            schema_version: raw.schema_version,
            bundle_id: raw.bundle_id,
            title: raw.title,
            created_at: raw.created_at,
            capabilities: raw.capabilities,
            sections: raw.sections,
        };

        let validation_issues = manifest.validate();
        for issue in validation_issues {
            if issue.severity == "warning" {
                warnings.push(issue);
            }
        }

        Ok((manifest, warnings))
    }

    fn ensure_valid(&self) -> CoreResult<()> {
        if self.schema_version != BUNDLE_SCHEMA_VERSION {
            return Err(CoreError::InvalidInput(format!(
                "unsupported schema version: {}, expected {BUNDLE_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        if self.bundle_id.is_empty() {
            return Err(CoreError::InvalidInput(
                "manifest: bundle_id is required".into(),
            ));
        }
        if self.title.is_empty() {
            return Err(CoreError::InvalidInput(
                "manifest: title is required".into(),
            ));
        }
        if self.created_at.is_empty() {
            return Err(CoreError::InvalidInput(
                "manifest: created_at is required".into(),
            ));
        }
        Ok(())
    }

    pub fn validate(&self) -> Vec<BundleIssueDto> {
        let mut issues = Vec::new();

        if self.bundle_id.is_empty() {
            issues.push(BundleIssueDto {
                severity: "error".into(),
                section: "manifest".into(),
                code: "missing_field".into(),
                message: "bundle_id is required".into(),
                field: Some("bundle_id".into()),
            });
        }
        if self.title.is_empty() {
            issues.push(BundleIssueDto {
                severity: "error".into(),
                section: "manifest".into(),
                code: "missing_field".into(),
                message: "title is required".into(),
                field: Some("title".into()),
            });
        }
        if self.created_at.is_empty() {
            issues.push(BundleIssueDto {
                severity: "error".into(),
                section: "manifest".into(),
                code: "missing_field".into(),
                message: "created_at is required".into(),
                field: Some("created_at".into()),
            });
        }
        if self.schema_version != BUNDLE_SCHEMA_VERSION {
            issues.push(BundleIssueDto {
                severity: "error".into(),
                section: "manifest".into(),
                code: "unsupported_version".into(),
                message: format!(
                    "expected {BUNDLE_SCHEMA_VERSION}, got {}",
                    self.schema_version
                ),
                field: Some("schema_version".into()),
            });
        }

        for (key, val) in &self.capabilities {
            let normalized = val.to_lowercase().replace('-', "_");
            match normalized.as_str() {
                "preview_only" | "apply_export" => {}
                _ => {
                    issues.push(BundleIssueDto {
                        severity: "warning".into(),
                        section: "manifest".into(),
                        code: "invalid_capability".into(),
                        message: format!(
                            "Unknown capability value '{val}' for '{key}', expected preview_only or apply_export"
                        ),
                        field: Some(format!("capabilities.{key}")),
                    });
                }
            }
        }

        issues
    }

    pub fn section_path(&self, name: &str) -> Option<&str> {
        self.sections.get(name).map(|s| s.as_str())
    }
}
