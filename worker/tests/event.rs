#[test]
fn event_invalid_signatures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
