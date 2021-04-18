#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/incorrect/*.rs");
    t.pass("tests/ui/correct/*.rs");
}
