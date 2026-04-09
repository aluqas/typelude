#[test]
fn wasm_wat_ui_failures() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/wasm_wat/*.rs");
}
