use std::{
    fs,
    path::{Path, PathBuf},
};

use typelude_tooling_core::ToolingResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirArtifactConfig {
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MirArtifactReport {
    pub mir_files: Vec<PathBuf>,
    pub dot_files: Vec<PathBuf>,
}

#[derive(Debug, Default)]
pub struct MirArtifactCollector;

impl MirArtifactCollector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn collect(&self, config: &MirArtifactConfig) -> ToolingResult<MirArtifactReport> {
        let mut report = MirArtifactReport::default();
        visit_dir(&config.root, &mut report)?;
        Ok(report)
    }
}

fn visit_dir(root: &Path, report: &mut MirArtifactReport) -> ToolingResult<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, report)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("mir") {
            report.mir_files.push(path);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("dot") {
            report.dot_files.push(path);
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

    use super::{MirArtifactCollector, MirArtifactConfig};

    #[test]
    fn collects_mir_and_dot_files() {
        let nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("time should advance").as_nanos();
        let root = std::env::temp_dir().join(format!("typelude-tooling-mir-{nanos}"));
        fs::create_dir_all(&root).expect("temp dir should be created");
        fs::write(root.join("a.mir"), "").expect("mir fixture should be created");
        fs::write(root.join("b.dot"), "").expect("dot fixture should be created");

        let report = MirArtifactCollector::new()
            .collect(&MirArtifactConfig {
                root: root.clone(),
            })
            .expect("mir artifacts should collect");
        assert_eq!(report.mir_files.len(), 1);
        assert_eq!(report.dot_files.len(), 1);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }
}
