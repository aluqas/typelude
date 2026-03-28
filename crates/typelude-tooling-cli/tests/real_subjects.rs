use clap::Parser;
use typelude_tooling_cli::{Cli, run};

fn owners() -> [&'static str; 4] {
    ["RunWriter", "RunState", "OpPush", "OpIf"]
}

#[test]
#[ignore = "requires nightly rustc_private driver and typelude-vm build"]
fn solve_owner_smoke_for_small_real_subjects() {
    for owner in owners() {
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "solve-owner",
            "--package",
            "typelude-vm",
            "--owner",
            owner,
            "--output",
            "text",
            "--view",
            "summary",
        ]);
        let output = run(cli).expect("solve-owner should run for real subject");
        assert!(output.contains("subjects="), "owner={owner} output={output}");
    }
}
