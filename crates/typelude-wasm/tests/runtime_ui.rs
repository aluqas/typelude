#[test]
fn runtime_ui_failures() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("src/tests/ui/runtime/*.rs");
}
