#[test]
fn bad_syntax() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/bad_syntax/*.rs");
}
