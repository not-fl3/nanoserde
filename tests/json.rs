use nanoserde::{DeJson, SerJson};

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
fn de_non_exhaustive() {
    #[derive(DeJson)]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
    }

    let json = r#"{
        "some": {
            "export": {
                "target":"."
            }
        },
        "a": 1,
        "b": 2.0,
        "b_": 5.,
        "d": "hello",
        "d__": "this string is going nowhere",
        "e": 1.,
        "extra_array": [1, 2, 3],
        "extra_struct": {"a": 1., "b": [1, {"a": 1}]}
    }"#;

    let test: Test = DeJson::deserialize_json(json).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2.);
    assert_eq!(test.d.unwrap(), "hello");
    assert_eq!(test.c, None);
}

#[test]
fn de_container_default() {
    #[derive(DeJson)]
    #[nserde(default)]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
    }

    let json = r#"{
        "a": 1,
        "d": "hello",
    }"#;

    let test: Test = DeJson::deserialize_json(json).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 0.);
    assert_eq!(test.d.unwrap(), "hello");
    assert_eq!(test.c, None);
}

#[test]
fn rename() {
    #[derive(DeJson, SerJson, PartialEq)]
    #[nserde(default)]
    pub struct Test {
        #[nserde(rename = "fooField")]
        pub a: i32,
        #[nserde(rename = "barField")]
        pub b: i32,
    }

    let json = r#"{
        "fooField": 1,
        "barField": 2,
    }"#;

    let test: Test = DeJson::deserialize_json(json).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2);

    let bytes = SerJson::serialize_json(&test);
    let test_deserialized = DeJson::deserialize_json(&bytes).unwrap();
    assert!(test == test_deserialized);
}

#[test]
fn de_field_default() {
    #[derive(DeJson)]
    struct Foo {
        x: i32,
    }
    impl Default for Foo {
        fn default() -> Foo {
            Foo { x: 23 }
        }
    }

    #[derive(DeJson)]
    pub struct Test {
        a: i32,
        #[nserde(default)]
        foo: Foo,
        foo2: Foo,
        b: f32,
    }

    let json = r#"{
        "a": 1,
        "b": 2.,
        "foo2": { "x": 3 }
    }"#;

    let test: Test = DeJson::deserialize_json(json).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2.);
    assert_eq!(test.foo.x, 23);
    assert_eq!(test.foo2.x, 3);
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
fn one_field() {
    #[derive(DeJson, SerJson, PartialEq)]
    pub struct OneField {
        field: f32,
    }

    let test = OneField { field: 23. };
    let bytes = SerJson::serialize_json(&test);
    let test_deserialized = DeJson::deserialize_json(&bytes).unwrap();
    assert!(test == test_deserialized);
}

#[test]
fn array() {
    #[derive(DeJson)]
    pub struct Foo {
        x: i32,
    }

    #[derive(DeJson)]
    pub struct Bar {
        foos: Vec<Foo>,
        ints: Vec<i32>,
        floats_a: Option<Vec<f32>>,
        floats_b: Option<Vec<f32>>,
    }

    let json = r#"{
       "foos": [{"x": 1}, {"x": 2}],
       "ints": [1, 2, 3, 4],
       "floats_b": [4., 3., 2., 1.]
    }"#;

    let bar: Bar = DeJson::deserialize_json(json).unwrap();

    assert_eq!(bar.foos.len(), 2);
    assert_eq!(bar.foos[0].x, 1);
    assert_eq!(bar.ints.len(), 4);
    assert_eq!(bar.ints[2], 3);
    assert_eq!(bar.floats_b.unwrap()[2], 2.);
    assert_eq!(bar.floats_a, None);
}

#[test]
fn path_type() {
    #[derive(DeJson)]
    struct Foo {
        a: i32,
        b: std::primitive::i32,
        c: Option<std::primitive::i32>,
        d: Option<Vec<std::vec::Vec<std::primitive::i32>>>,
    }

    let json = r#"{
       "a": 0,
       "b": 1,
       "c": 2,
       "d": [[1, 2], [3, 4]]
    }"#;

    let bar: Foo = DeJson::deserialize_json(json).unwrap();

    assert_eq!(bar.a, 0);
    assert_eq!(bar.b, 1);
    assert_eq!(bar.c, Some(2));
    assert_eq!(bar.d, Some(vec![vec![1, 2], vec![3, 4]]));
}

#[test]
fn hashmaps() {
    #[derive(DeJson)]
    struct Foo {
        map: std::collections::HashMap<String, i32>,
    }

    let json = r#"{
       "map": {
          "asd": 1,
          "qwe": 2
       }
    }"#;

    let foo: Foo = DeJson::deserialize_json(json).unwrap();

    assert_eq!(foo.map["asd"], 1);
    assert_eq!(foo.map["qwe"], 2);
}

#[test]
fn exponents() {
    #[derive(DeJson)]
    struct Foo {
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
    };

    let json = r#"{
        "a": 1e2,
        "b": 1e-2,
        "c": 1E2,
        "d": 1E-2,
        "e": 1.0e2,
        "f": 1.0e-2,
        "g": 1.0E2,
        "h": 1.0E-2
    }"#;

    let foo: Foo = DeJson::deserialize_json(json).unwrap();

    assert_eq!(foo.a, 100.);
    assert_eq!(foo.b, 0.01);
    assert_eq!(foo.c, 100.);
    assert_eq!(foo.d, 0.01);
    assert_eq!(foo.e, 100.);
    assert_eq!(foo.f, 0.01);
    assert_eq!(foo.g, 100.);
    assert_eq!(foo.h, 0.01);
}

#[test]
fn jsonerror() {
    #[derive(DeJson)]
    #[allow(dead_code)]
    struct Foo {
        i: i32,
    }

    let json = r#"{
       "i": "string"
    }"#;

    let res: Result<Foo, _> = DeJson::deserialize_json(json);
    match res {
        Ok(_) => assert!(false),
        Err(e) => {
            let _dyn_e: Box<dyn std::error::Error> = std::convert::From::from(e);
        }
    }
}

#[test]
fn de_enum() {
    #[derive(DeJson, PartialEq, Debug)]
    pub enum Foo {
        A,
        B(i32, String),
        C { a: i32, b: String },
    }

    #[derive(DeJson, PartialEq, Debug)]
    pub struct Bar {
        foo1: Foo,
        foo2: Foo,
        foo3: Foo,
    }

    let json = r#"
       {
          "foo1": { "A": [] },
          "foo2": { "B": [ 1, "asd" ] },
          "foo3": { "C": { "a": 2, "b": "qwe" } }
       }
    "#;

    let test: Bar = DeJson::deserialize_json(json).unwrap();

    assert_eq!(test.foo1, Foo::A);
    assert_eq!(test.foo2, Foo::B(1, "asd".to_string()));
    assert_eq!(
        test.foo3,
        Foo::C {
            a: 2,
            b: "qwe".to_string()
        }
    );
}

#[test]
fn test_various_escapes() {
    let json = r#""\n\t\u0020\f\b\\\"\/\ud83d\uDE0B\r""#;
    let unescaped: String = DeJson::deserialize_json(json).unwrap();
    assert_eq!(unescaped, "\n\t\u{20}\x0c\x08\\\"/😋\r");
}

// there are only 1024*1024 surrogate pairs, so we can do an exhautive test.
#[test]
fn test_surrogate_pairs_exhaustively() {
    for lead in 0xd800..0xdc00 {
        for trail in 0xdc00..0xe000 {
            // find the scalar value represented by the [lead, trail] pair.
            let combined: Vec<char> = core::char::decode_utf16([lead, trail].iter().copied())
                .collect::<Result<_, _>>()
                .unwrap_or_else(|e| {
                    panic!(
                        "[{:#04x}, {:#04x}] not valid surrogate pair? {:?}",
                        lead, trail, e,
                    );
                });
            assert_eq!(combined.len(), 1);
            let expected_string = format!("{}", combined[0]);

            let json = format!(r#""\u{:04x}\u{:04x}""#, lead, trail);
            let result: String = DeJson::deserialize_json(&json).unwrap_or_else(|e| {
                panic!("should be able to parse {}: {:?}", &json, e);
            });
            assert_eq!(result, expected_string, "failed on input {}", json);
            assert_eq!(result.chars().count(), 1);
        }
    }
}
