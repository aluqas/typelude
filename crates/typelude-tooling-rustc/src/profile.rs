use std::{
    fs,
    path::{Path, PathBuf},
};

use typelude_tooling_core::{MetricKind, MetricRecord, ToolingResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfProfileConfig {
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelfProfileReport {
    pub artifacts: Vec<PathBuf>,
    pub metrics: Vec<MetricRecord>,
}

#[derive(Debug, Default)]
pub struct SelfProfileCollector;

impl SelfProfileCollector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn collect(&self, config: &SelfProfileConfig) -> ToolingResult<SelfProfileReport> {
        let mut artifacts = Vec::new();
        visit_dir(&config.root, &mut artifacts)?;

        let metrics = vec![MetricRecord::new(
            MetricKind::ArtifactCount,
            "self_profile_artifacts",
            artifacts.len() as f64,
        )];

        Ok(SelfProfileReport {
            artifacts,
            metrics,
        })
    }
}

fn visit_dir(root: &Path, artifacts: &mut Vec<PathBuf>) -> ToolingResult<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, artifacts)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "mm_profdata" || extension == "mm_profraw")
        {
            artifacts.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{SelfProfileCollector, SelfProfileConfig};

    #[test]
    fn collects_self_profile_artifacts() {
        let nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("time should advance").as_nanos();
        let root = std::env::temp_dir().join(format!("typelude-tooling-prof-{nanos}"));
        fs::create_dir_all(&root).expect("temp dir should be created");
        fs::write(root.join("sample.mm_profdata"), "").expect("profile fixture should be created");

        let report = SelfProfileCollector::new()
            .collect(&SelfProfileConfig {
                root: root.clone(),
            })
            .expect("profile artifacts should collect");
        assert_eq!(report.artifacts.len(), 1);
        assert_eq!(report.metrics[0].value, 1.0);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }
}
