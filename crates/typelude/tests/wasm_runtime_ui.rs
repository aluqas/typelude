#[test]
fn wasm_runtime_ui_failures() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/wasm_runtime/*.rs");
}
