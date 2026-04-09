use std::path::PathBuf;

use typelude_tooling_cargo_profile::{CargoProfileAnalyzer, render_text};
use typelude_tooling_core::ToolingResult;

use crate::OutputModeArg;

pub(crate) fn run_cargo_profile(
    chrome_profiler_json: Option<PathBuf>,
    summarize_json: Option<PathBuf>,
    self_profile_prefix: Option<PathBuf>,
    top: usize,
    output: OutputModeArg,
) -> ToolingResult<String> {
    let analysis = CargoProfileAnalyzer::new().analyze_paths(
        chrome_profiler_json.as_deref(),
        summarize_json.as_deref(),
        self_profile_prefix.as_deref(),
        top,
    )?;

    match output {
        OutputModeArg::Text => Ok(render_text(&analysis)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&analysis)?),
    }
}
