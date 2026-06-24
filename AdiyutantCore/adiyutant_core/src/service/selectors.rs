use crate::error::CoreError;
use std::fmt;

/// Represents a parsed roadmap selector.
///
/// - Unqualified slug: "my-roadmap" — allowed only when unambiguous across all roadmaps.
/// - Qualified: "my-project/my-roadmap" — project-slug/roadmap-slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoadmapSelector {
    Unqualified {
        slug: String,
    },
    Qualified {
        project_slug: String,
        roadmap_slug: String,
    },
}

/// Represents a parsed roadmap-item selector.
///
/// - Unqualified slug: "my-item" — allowed only when unambiguous.
/// - Qualified: "phase-slug/my-item" — within a phase, or full "roadmap-slug/phase-slug/item-slug".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoadmapItemSelector {
    Unqualified {
        slug: String,
    },
    PhaseQualified {
        phase_slug: String,
        item_slug: String,
    },
    FullyQualified {
        roadmap_slug: String,
        phase_slug: String,
        item_slug: String,
    },
}

impl fmt::Display for RoadmapSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoadmapSelector::Unqualified { slug } => write!(f, "{slug}"),
            RoadmapSelector::Qualified {
                project_slug,
                roadmap_slug,
            } => {
                write!(f, "{project_slug}/{roadmap_slug}")
            }
        }
    }
}

impl fmt::Display for RoadmapItemSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoadmapItemSelector::Unqualified { slug } => write!(f, "{slug}"),
            RoadmapItemSelector::PhaseQualified {
                phase_slug,
                item_slug,
            } => {
                write!(f, "{phase_slug}/{item_slug}")
            }
            RoadmapItemSelector::FullyQualified {
                roadmap_slug,
                phase_slug,
                item_slug,
            } => {
                write!(f, "{roadmap_slug}/{phase_slug}/{item_slug}")
            }
        }
    }
}

/// Parse a roadmap selector string.
///
/// Split on `/`. 1 part → Unqualified, 2 parts → Qualified, 0 or ≥3 parts → error.
/// Empty parts after splitting → error. Whitespace around parts is trimmed.
pub fn parse_roadmap_selector(s: &str) -> Result<RoadmapSelector, CoreError> {
    if s.is_empty() {
        return Err(CoreError::InvalidInput(
            "roadmap selector cannot be empty".into(),
        ));
    }

    let parts: Vec<&str> = s.split('/').collect();

    if parts.is_empty() || parts.len() >= 3 {
        return Err(CoreError::InvalidInput(format!(
            "invalid roadmap selector '{}': expected 1 or 2 parts separated by '/'",
            s
        )));
    }

    let trimmed: Vec<String> = parts.iter().map(|p| p.trim().to_string()).collect();

    if trimmed.iter().any(|p| p.is_empty()) {
        return Err(CoreError::InvalidInput(format!(
            "invalid roadmap selector '{}': empty part",
            s
        )));
    }

    match trimmed.len() {
        1 => Ok(RoadmapSelector::Unqualified {
            slug: trimmed.into_iter().next().unwrap(),
        }),
        2 => {
            let mut iter = trimmed.into_iter();
            Ok(RoadmapSelector::Qualified {
                project_slug: iter.next().unwrap(),
                roadmap_slug: iter.next().unwrap(),
            })
        }
        _ => Err(CoreError::InvalidInput(format!(
            "invalid roadmap selector '{}': expected 1 or 2 parts separated by '/'",
            s
        ))),
    }
}

/// Parse a roadmap-item selector string.
///
/// Split on `/`. 1 part → Unqualified, 2 parts → PhaseQualified,
/// 3 parts → FullyQualified. 0 or ≥4 parts → error.
pub fn parse_roadmap_item_selector(s: &str) -> Result<RoadmapItemSelector, CoreError> {
    if s.is_empty() {
        return Err(CoreError::InvalidInput(
            "roadmap item selector cannot be empty".into(),
        ));
    }

    let parts: Vec<&str> = s.split('/').collect();

    if parts.is_empty() || parts.len() >= 4 {
        return Err(CoreError::InvalidInput(format!(
            "invalid roadmap item selector '{}': expected 1, 2, or 3 parts separated by '/'",
            s
        )));
    }

    let trimmed: Vec<String> = parts.iter().map(|p| p.trim().to_string()).collect();

    if trimmed.iter().any(|p| p.is_empty()) {
        return Err(CoreError::InvalidInput(format!(
            "invalid roadmap item selector '{}': empty part",
            s
        )));
    }

    match trimmed.len() {
        1 => Ok(RoadmapItemSelector::Unqualified {
            slug: trimmed.into_iter().next().unwrap(),
        }),
        2 => {
            let mut iter = trimmed.into_iter();
            Ok(RoadmapItemSelector::PhaseQualified {
                phase_slug: iter.next().unwrap(),
                item_slug: iter.next().unwrap(),
            })
        }
        3 => {
            let mut iter = trimmed.into_iter();
            Ok(RoadmapItemSelector::FullyQualified {
                roadmap_slug: iter.next().unwrap(),
                phase_slug: iter.next().unwrap(),
                item_slug: iter.next().unwrap(),
            })
        }
        _ => Err(CoreError::InvalidInput(format!(
            "invalid roadmap item selector '{}': expected 1, 2, or 3 parts separated by '/'",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roadmap_selector_unqualified() {
        let result = parse_roadmap_selector("my-roadmap").unwrap();
        assert_eq!(
            result,
            RoadmapSelector::Unqualified {
                slug: "my-roadmap".into()
            }
        );
    }

    #[test]
    fn parse_roadmap_selector_qualified() {
        let result = parse_roadmap_selector("my-project/my-roadmap").unwrap();
        assert_eq!(
            result,
            RoadmapSelector::Qualified {
                project_slug: "my-project".into(),
                roadmap_slug: "my-roadmap".into(),
            }
        );
    }

    #[test]
    fn parse_roadmap_selector_too_many_parts() {
        let err = parse_roadmap_selector("a/b/c").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("invalid roadmap selector"));
    }

    #[test]
    fn parse_roadmap_selector_empty_input() {
        let err = parse_roadmap_selector("").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn parse_roadmap_selector_whitespace_trimmed() {
        let result = parse_roadmap_selector("  my-roadmap  ").unwrap();
        assert_eq!(
            result,
            RoadmapSelector::Unqualified {
                slug: "my-roadmap".into()
            }
        );
    }

    #[test]
    fn parse_roadmap_selector_empty_part() {
        let err = parse_roadmap_selector("a//b").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn parse_roadmap_item_selector_unqualified() {
        let result = parse_roadmap_item_selector("my-item").unwrap();
        assert_eq!(
            result,
            RoadmapItemSelector::Unqualified {
                slug: "my-item".into()
            }
        );
    }

    #[test]
    fn parse_roadmap_item_selector_phase_qualified() {
        let result = parse_roadmap_item_selector("phase-slug/my-item").unwrap();
        assert_eq!(
            result,
            RoadmapItemSelector::PhaseQualified {
                phase_slug: "phase-slug".into(),
                item_slug: "my-item".into(),
            }
        );
    }

    #[test]
    fn parse_roadmap_item_selector_fully_qualified() {
        let result = parse_roadmap_item_selector("roadmap-slug/phase-slug/my-item").unwrap();
        assert_eq!(
            result,
            RoadmapItemSelector::FullyQualified {
                roadmap_slug: "roadmap-slug".into(),
                phase_slug: "phase-slug".into(),
                item_slug: "my-item".into(),
            }
        );
    }

    #[test]
    fn parse_roadmap_item_selector_too_many_parts() {
        let err = parse_roadmap_item_selector("a/b/c/d").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn parse_roadmap_item_selector_empty_input() {
        let err = parse_roadmap_item_selector("").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn parse_roadmap_item_selector_empty_part() {
        let err = parse_roadmap_item_selector("a//b").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn round_trip_roadmap_selector_unqualified() {
        let selector = RoadmapSelector::Unqualified {
            slug: "my-roadmap".into(),
        };
        let s = selector.to_string();
        let parsed = parse_roadmap_selector(&s).unwrap();
        assert_eq!(parsed, selector);
    }

    #[test]
    fn round_trip_roadmap_selector_qualified() {
        let selector = RoadmapSelector::Qualified {
            project_slug: "my-project".into(),
            roadmap_slug: "my-roadmap".into(),
        };
        let s = selector.to_string();
        let parsed = parse_roadmap_selector(&s).unwrap();
        assert_eq!(parsed, selector);
    }

    #[test]
    fn round_trip_roadmap_item_selector_unqualified() {
        let selector = RoadmapItemSelector::Unqualified {
            slug: "my-item".into(),
        };
        let s = selector.to_string();
        let parsed = parse_roadmap_item_selector(&s).unwrap();
        assert_eq!(parsed, selector);
    }

    #[test]
    fn round_trip_roadmap_item_selector_phase_qualified() {
        let selector = RoadmapItemSelector::PhaseQualified {
            phase_slug: "phase-1".into(),
            item_slug: "item-1".into(),
        };
        let s = selector.to_string();
        let parsed = parse_roadmap_item_selector(&s).unwrap();
        assert_eq!(parsed, selector);
    }

    #[test]
    fn round_trip_roadmap_item_selector_fully_qualified() {
        let selector = RoadmapItemSelector::FullyQualified {
            roadmap_slug: "rm".into(),
            phase_slug: "ph".into(),
            item_slug: "it".into(),
        };
        let s = selector.to_string();
        let parsed = parse_roadmap_item_selector(&s).unwrap();
        assert_eq!(parsed, selector);
    }
}
