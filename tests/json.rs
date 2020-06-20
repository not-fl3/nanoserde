#[test]
fn de() {
    use nanoserde::{DeJson, DeJsonErr};

    #[derive(DeJson)]
    struct Test {
        a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
    }

    let json = r#"{
        "a": 1,
        "b": 2.0,
        "d": "hello"
    }"#;

    let test: Test = DeJson::deserialize_json(json).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2.);
    assert_eq!(test.d.unwrap(), "hello");
    assert_eq!(test.c, None);
}
