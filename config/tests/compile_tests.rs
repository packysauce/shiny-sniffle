#[test]
fn all() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cases/fail*.rs");
    t.pass("tests/cases/pass*.rs");
}
