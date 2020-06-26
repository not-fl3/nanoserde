use nanoserde::DeRon;

#[test]
fn ron_de() {
    #[derive(DeRon)]
    pub struct Test {
        a: i32,
        b: f32,
        d: Option<String>,
    }

    let ron = r#"(
        a: 1,
        b: 2.0,
        d: "hello",
    )"#;

    let test: Test = DeRon::deserialize_ron(ron).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2.);
    assert_eq!(test.d.unwrap(), "hello");
}
