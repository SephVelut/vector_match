#[test]
fn test_generate_match() {
    let t = trybuild::TestCases::new();
    t.pass("tests/vector_match.rs");
}
