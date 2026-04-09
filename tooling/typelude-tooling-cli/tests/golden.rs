use clap::Parser;

#[test]
fn graph_json_output_from_expr_contains_nodes() {
    let cli = typelude_tooling_cli::Cli::parse_from([
        "typelude-tooling-cli",
        "graph",
        "--expr",
        "EIf<True, U1, U0>",
        "--format",
        "json",
    ]);
    let output = typelude_tooling_cli::run(cli).expect("graph command should succeed");

    let actual: serde_json::Value =
        serde_json::from_str(&output).expect("output should be valid JSON");
    assert!(actual.get("nodes").is_some());
    assert!(actual.get("edges").is_some());
}

#[test]
fn analyze_json_output_from_expr_contains_summary() {
    let cli = typelude_tooling_cli::Cli::parse_from([
        "typelude-tooling-cli",
        "analyze",
        "--expr",
        "EIf<True, U1, U0>",
        "--output",
        "json",
    ]);
    let output = typelude_tooling_cli::run(cli).expect("analyze command should succeed");
    let actual: serde_json::Value =
        serde_json::from_str(&output).expect("output should be valid JSON");

    assert!(actual.get("node_count").is_some());
    assert!(actual.get("distribution").is_some());
}
