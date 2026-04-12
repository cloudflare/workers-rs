#[test]
fn event_invalid_signatures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

#[test]
fn event_valid_signatures() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
}
