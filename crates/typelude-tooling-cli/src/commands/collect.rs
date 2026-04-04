use std::{
    fs,
    path::PathBuf,
    process::Command,
};

use typelude_tooling_core::{ToolingError, ToolingResult, Trace, TraceId};

use crate::{HookArg, OwnerMatchArg};
use crate::process::cargo::ensure_driver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryKindArg {
    Owner,
    Impl,
    AssocItem,
}

pub(crate) fn run_collect(
    output: PathBuf,
    cargo_subcommand: &str,
    package: Option<String>,
    manifest_path: Option<PathBuf>,
    toolchain: &str,
    hooks: &[HookArg],
    subject_filter: Option<String>,
    rebuild_driver: bool,
) -> ToolingResult<String> {
    let trace = collect_trace(
        cargo_subcommand,
        package,
        manifest_path,
        toolchain,
        hooks,
        subject_filter.clone(),
        rebuild_driver,
        None,
        None,
        None,
    )?;
    fs::write(&output, trace.to_json_lines()?)?;

    let hooks_text = if hooks.is_empty() {
        String::from("default")
    } else {
        hooks
            .iter()
            .map(|hook| format!("{:?}", hook.hook_id()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    Ok(format!(
        "collected {} events into {} using hooks: {} subject_filter={}",
        trace.events.len(),
        output.display(),
        hooks_text,
        subject_filter.unwrap_or_else(|| String::from("<none>"))
    ))
}

pub(crate) fn collect_trace(
    cargo_subcommand: &str,
    package: Option<String>,
    manifest_path: Option<PathBuf>,
    toolchain: &str,
    hooks: &[HookArg],
    subject_filter: Option<String>,
    rebuild_driver: bool,
    owner_query: Option<&str>,
    query_kind: Option<QueryKindArg>,
    query_match: Option<OwnerMatchArg>,
) -> ToolingResult<Trace> {
    let driver_path = ensure_driver(toolchain, rebuild_driver)?;
    let mut command = Command::new("cargo");
    command
        .arg(format!("+{toolchain}"))
        .arg(cargo_subcommand)
        .arg("--quiet");
    if let Some(package) = package {
        command.args(["-p", &package]);
    }
    if let Some(manifest_path) = manifest_path {
        command.arg("--manifest-path").arg(manifest_path);
    }
    command.env("RUSTC_WRAPPER", driver_path);
    command.env("TYPELUDE_TOOLING_SUMMARY_ONLY", "0");
    if let Some(subject_filter) = &subject_filter {
        command.env("TYPELUDE_TOOLING_SUBJECT_FILTER", subject_filter);
    }
    if let Some(owner_query) = owner_query {
        command.env("TYPELUDE_TOOLING_QUERY_OWNER", owner_query);
    }
    if let Some(query_kind) = query_kind {
        let value = match query_kind {
            QueryKindArg::Owner => "owner",
            QueryKindArg::Impl => "impl",
            QueryKindArg::AssocItem => "assoc_item",
        };
        command.env("TYPELUDE_TOOLING_QUERY_KIND", value);
    }
    if let Some(query_match) = query_match {
        let value = match query_match {
            OwnerMatchArg::Substring => "substring",
            OwnerMatchArg::Suffix => "suffix",
            OwnerMatchArg::Exact => "exact",
            OwnerMatchArg::DefId => "def_id",
        };
        command.env("TYPELUDE_TOOLING_QUERY_MATCH", value);
    }
    if !hooks.is_empty() {
        let enabled = hooks
            .iter()
            .map(|hook| hook.as_env())
            .collect::<Vec<_>>()
            .join(",");
        command.env("TYPELUDE_TOOLING_HOOKS", enabled);
    }

    let result = command.output()?;
    if !result.status.success() {
        return Err(ToolingError::Command(format!(
            "cargo {cargo_subcommand} failed with status {}",
            result.status
        )));
    }

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let trace = typelude_tooling_core::ingest::trace_from_json_lines(TraceId::new(1), &combined)?;
    if trace.events.is_empty() {
        return Err(ToolingError::Parse(String::from(
            "no trace events were collected from rustc_private output",
        )));
    }
    Ok(trace)
}
