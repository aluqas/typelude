use std::{path::PathBuf, process::Command};

use typelude_tooling_core::{ToolingError, ToolingResult};

#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root should exist")
        .to_path_buf()
}

#[must_use]
pub fn driver_path() -> PathBuf {
    let root = workspace_root();
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    target_dir.join("debug").join(format!("typelude-rustc-driver{}", std::env::consts::EXE_SUFFIX))
}

pub fn ensure_driver(toolchain: &str, rebuild: bool) -> ToolingResult<PathBuf> {
    let path = driver_path();
    if path.exists() && !rebuild {
        return Ok(path);
    }

    let root = workspace_root();
    let status = Command::new("cargo")
        .current_dir(&root)
        .arg(format!("+{toolchain}"))
        .args(["build", "-p", "typelude-tooling-rustc-private", "--bin", "typelude-rustc-driver"])
        .status()?;
    if !status.success() {
        return Err(ToolingError::Command(String::from("failed to build typelude-rustc-driver")));
    }
    Ok(driver_path())
}
