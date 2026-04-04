use std::process::Command;

use serde::Serialize;
use typelude_tooling_core::ToolingResult;

use crate::OutputModeArg;
use crate::process::cargo::driver_path;

#[derive(Debug, Serialize)]
pub(crate) struct DoctorReport {
    pub nightly_available: bool,
    pub driver_exists: bool,
    pub required_env_vars: Vec<String>,
    pub supported_flags: Vec<String>,
}

pub(crate) fn run_doctor(output: OutputModeArg) -> ToolingResult<String> {
    let nightly = Command::new("rustup")
        .args(["run", "nightly", "rustc", "-V"])
        .output();
    let nightly_available = nightly.as_ref().is_ok_and(|result| result.status.success());
    let path = driver_path();
    let driver_exists = path.exists();
    let report = DoctorReport {
        nightly_available,
        driver_exists,
        required_env_vars: vec![String::from("RUSTC_WRAPPER")],
        supported_flags: vec![
            String::from("collect"),
            String::from("trace"),
            String::from("solve-tree"),
            String::from("solve-summary"),
            String::from("solve-owner"),
            String::from("solve-impl"),
            String::from("solve-assoc-item"),
            String::from("solve-diff"),
            String::from("graph"),
            String::from("analyze"),
            String::from("-Z dump-mir=all"),
            String::from("-Z time-passes"),
            String::from("-Z print-type-sizes"),
            String::from("-Z self-profile"),
        ],
    };

    match output {
        OutputModeArg::Text => Ok(format!(
            "nightly_available: {}\ndriver_exists: {}\nrequired_env_vars: {}\nsupported_flags: {}",
            report.nightly_available,
            report.driver_exists,
            report.required_env_vars.join(", "),
            report.supported_flags.join(", ")
        )),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&report)?),
    }
}
