use nanoserde::{DeJson, DeJsonErr};

#[test]
fn de() {
    #[derive(DeJson)]
    pub struct Test {
        pub a: i32,
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

#[test]
fn doctests() {
    /// This is test
    /// second doc comment
    #[derive(DeJson)]
    pub struct Test {
        /// with documented field
        pub a: i32,
        pub b: f32,
        /// or here
        /// Or here
        c: Option<String>,
        /// more doc comments
        /// and more
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

#[test]
fn empty() {
    #[derive(DeJson)]
    pub struct Empty {}

    let json = r#"{
    }"#;

    let _: Empty = DeJson::deserialize_json(json).unwrap();
}

#[test]
fn array() {
    #[derive(DeJson)]
    pub struct Foo {
        x: i32
    }

    #[derive(DeJson)]
    pub struct Bar {
        foos: Vec<Foo>,
        ints: Vec<i32>
    }

    let json = r#"{
       "foos": [{"x": 1}, {"x": 2}],
       "ints": [1, 2, 3, 4]
    }"#;

    let bar: Bar = DeJson::deserialize_json(json).unwrap();

    assert_eq!(bar.foos.len(), 2);
    assert_eq!(bar.foos[0].x, 1);
    assert_eq!(bar.ints.len(), 4);
    assert_eq!(bar.ints[2], 3);

}
