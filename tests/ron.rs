use nanoserde::{DeRon, SerRon};

use std::{
    collections::{BTreeSet, LinkedList},
    sync::atomic::AtomicBool,
};

#[cfg(feature = "no_std")]
use hashbrown::{HashMap, HashSet};

#[cfg(not(feature = "no_std"))]
use std::collections::{HashMap, HashSet};

#[test]
fn ron_de() {
    #[derive(DeRon)]
    pub struct Test {
        a: i32,
        b: f32,
        c: Option<String>,
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
    assert_eq!(test.c, None);
    assert_eq!(test.d.unwrap(), "hello");
}

#[test]
fn de_container_default() {
    #[derive(DeRon)]
    #[nserde(default)]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
    }

    let ron = r#"(
        a: 1,
        d: "hello",
    )"#;

    let test: Test = DeRon::deserialize_ron(ron).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 0.);
    assert_eq!(test.d.unwrap(), "hello");
    assert_eq!(test.c, None);
}

#[test]
fn rename() {
    #[derive(DeRon, SerRon, PartialEq)]
    #[nserde(default)]
    pub struct Test {
        #[nserde(rename = "fooField")]
        pub a: i32,
        #[nserde(rename = "barField")]
        pub b: i32,
    }

    let ron = r#"(
        fooField: 1,
        barField: 2,
    )"#;

    let test: Test = DeRon::deserialize_ron(ron).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2);

    let bytes = SerRon::serialize_ron(&test);
    let test_deserialized = DeRon::deserialize_ron(&bytes).unwrap();
    assert!(test == test_deserialized);
}

#[test]
fn de_field_default() {
    #[derive(DeRon)]
    struct Foo {
        x: i32,
    }
    impl Default for Foo {
        fn default() -> Foo {
            Foo { x: 23 }
        }
    }

    #[derive(DeRon)]
    pub struct Test {
        a: i32,
        #[nserde(default)]
        foo: Foo,
        foo2: Foo,
        b: f32,
    }

    let ron = r#"(
        a: 1,
        b: 2.,
        foo2: (x: 3)
    )"#;

    let test: Test = DeRon::deserialize_ron(ron).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2.);
    assert_eq!(test.foo.x, 23);
    assert_eq!(test.foo2.x, 3);
}

#[test]
fn doctests() {
    /// This is test
    /// second doc comment
    #[derive(DeRon)]
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

    let ron = r#"(
        a: 1,
        b: 2.0,
        d: "hello"
    )"#;

    let test: Test = DeRon::deserialize_ron(ron).unwrap();
    assert_eq!(test.a, 1);
    assert_eq!(test.b, 2.);
    assert_eq!(test.d.unwrap(), "hello");
    assert_eq!(test.c, None);
}

#[test]
fn empty() {
    #[derive(DeRon)]
    pub struct Empty {}

    let ron = r#"(
    )"#;

    let _: Empty = DeRon::deserialize_ron(ron).unwrap();
}

#[test]
fn one_field() {
    #[derive(DeRon, SerRon, PartialEq)]
    pub struct OneField {
        field: f32,
    }

    let test = OneField { field: 23. };
    let bytes = SerRon::serialize_ron(&test);
    let test_deserialized = DeRon::deserialize_ron(&bytes).unwrap();
    assert!(test == test_deserialized);
}

#[test]
fn one_field_map() {
    #[derive(DeRon, SerRon, PartialEq)]
    pub struct OneField {
        field: HashMap<String, f32>,
    }

    let test = OneField {
        field: HashMap::new(),
    };
    let bytes = SerRon::serialize_ron(&test);
    let test_deserialized = DeRon::deserialize_ron(&bytes).unwrap();
    assert!(test == test_deserialized);
}

#[test]
fn array() {
    #[derive(DeRon)]
    pub struct Foo {
        x: i32,
    }

    #[derive(DeRon)]
    pub struct Bar {
        foos: Vec<Foo>,
        ints: Vec<i32>,
        floats_a: Option<Vec<f32>>,
        floats_b: Option<Vec<f32>>,
    }

    let ron = r#"(
       foos: [(x: 1), (x: 2)],
       ints: [1, 2, 3, 4],
       floats_b: [4., 3., 2., 1.]
    )"#;

    let bar: Bar = DeRon::deserialize_ron(ron).unwrap();

    assert_eq!(bar.foos.len(), 2);
    assert_eq!(bar.foos[0].x, 1);
    assert_eq!(bar.ints.len(), 4);
    assert_eq!(bar.ints[2], 3);
    assert_eq!(bar.floats_b.unwrap()[2], 2.);
    assert_eq!(bar.floats_a, None);
}

#[test]
fn collections() {
    #[derive(DeRon, SerRon, PartialEq, Debug)]
    pub struct Test {
        pub a: Vec<i32>,
        pub b: LinkedList<f32>,
        pub c: HashSet<i32>,
        pub d: BTreeSet<i32>,
    }

    let test: Test = Test {
        a: vec![1, 2, 3],
        b: vec![1.0, 2.0, 3.0, 4.0].into_iter().collect(),
        c: vec![1, 2, 3, 4, 5].into_iter().collect(),
        d: vec![1, 2, 3, 4, 5, 6].into_iter().collect(),
    };

    let bytes = SerRon::serialize_ron(&test);

    let test_deserialized = DeRon::deserialize_ron(&bytes).unwrap();

    assert_eq!(test, test_deserialized);
}

#[test]
fn path_type() {
    #[derive(DeRon)]
    struct Foo {
        a: i32,
        b: std::primitive::i32,
        c: Option<std::primitive::i32>,
        d: Option<Vec<std::vec::Vec<std::primitive::i32>>>,
    }

    let ron = r#"(
       a: 0,
       b: 1,
       c: 2,
       d: [[1, 2], [3, 4]]
    )"#;

    let bar: Foo = DeRon::deserialize_ron(ron).unwrap();

    assert_eq!(bar.a, 0);
    assert_eq!(bar.b, 1);
    assert_eq!(bar.c, Some(2));
    assert_eq!(bar.d, Some(vec![vec![1, 2], vec![3, 4]]));
}

#[test]
fn hashmaps() {
    #[derive(DeRon)]
    struct Foo {
        map: HashMap<String, i32>,
    }

    let ron = r#"(
       map: {
          "asd": 1,
          "qwe": 2,
       }
    )"#;

    let foo: Foo = DeRon::deserialize_ron(ron).unwrap();

    assert_eq!(foo.map["asd"], 1);
    assert_eq!(foo.map["qwe"], 2);
}

#[test]
fn exponents() {
    #[derive(DeRon)]
    struct Foo {
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
    }

    let ron = r#"(
        a: 1e2,
        b: 1e-2,
        c: 1E2,
        d: 1E-2,
        e: 1.0e2,
        f: 1.0e-2,
        g: 1.0E2,
        h: 1.0E-2
    )"#;

    let foo: Foo = DeRon::deserialize_ron(ron).unwrap();

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
fn ronerror() {
    #[derive(DeRon)]
    #[allow(dead_code)]
    struct Foo {
        i: i32,
    }

    let ron = r#"(
       i: "string"
    )"#;

    let res: Result<Foo, _> = DeRon::deserialize_ron(ron);
    match res {
        Ok(_) => assert!(false),
        Err(e) => {
            let _dyn_e: Box<dyn std::error::Error> = std::convert::From::from(e);
        }
    }
}

#[test]
fn de_enum() {
    #[derive(DeRon, PartialEq, Debug)]
    pub enum Foo {
        A,
        B(i32, String),
        C { a: i32, b: String },
    }

    #[derive(DeRon, PartialEq, Debug)]
    pub struct Bar {
        foo1: Foo,
        foo2: Foo,
        foo3: Foo,
    }

    let ron = r#"
       (
          foo1: A,
          foo2: B(1, "asd"),
          foo3: C(a: 2, b: "qwe"),
       )
    "#;

    let test: Bar = DeRon::deserialize_ron(ron).unwrap();

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
fn de_ser_enum() {
    #[derive(SerRon, DeRon, PartialEq, Debug)]
    pub enum Fud {
        A = 0,
        B = 1,
        C = 2,
    }

    #[derive(SerRon, DeRon, PartialEq, Debug)]
    pub struct Bar {
        foo1: Fud,
        foo2: Fud,
        foo3: Fud,
    }

    let ron = "(\n    foo1:A,\n    foo2:B,\n    foo3:C,\n)";

    let data = Bar {
        foo1: Fud::A,
        foo2: Fud::B,
        foo3: Fud::C,
    };
    let serialized = SerRon::serialize_ron(&data);

    assert_eq!(serialized, ron);

    let deserialized: Bar = DeRon::deserialize_ron(&serialized).unwrap();
    assert_eq!(deserialized, data);
}

#[test]
fn ser_enum_complex() {
    #[derive(SerRon, DeRon, PartialEq, Debug)]
    pub enum Foo {
        A,
        B(i32, String),
        C { a: i32, b: String },
    }

    #[derive(SerRon, DeRon, PartialEq, Debug)]
    pub struct Bar {
        foo1: Foo,
        foo2: Foo,
        foo3: Foo,
    }

    let ron = "(\n    foo1:A,\n    foo2:B(1, \"asd\"),\n    foo3:C(\n        a:2,\n        b:\"qwe\",\n    ),\n)";

    let data = Bar {
        foo1: Foo::A,
        foo2: Foo::B(1, String::from("asd")),
        foo3: Foo::C {
            a: 2,
            b: String::from("qwe"),
        },
    };
    let serialized = SerRon::serialize_ron(&data);
    assert_eq!(serialized, ron);

    let deserialized: Bar = DeRon::deserialize_ron(&serialized).unwrap();
    assert_eq!(deserialized, data);
}

#[test]
fn test_various_escapes() {
    let ron = r#""\n\t\u0020\f\b\\\"\/\ud83d\uDE0B\r""#;
    let unescaped: String = DeRon::deserialize_ron(ron).unwrap();
    assert_eq!(unescaped, "\n\t\u{20}\x0c\x08\\\"/😋\r");
}

// there are only 1024*1024 surrogate pairs, so we can do an exhautive test.
#[test]
#[cfg_attr(miri, ignore)]
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

            let ron = format!(r#""\u{:04x}\u{:04x}""#, lead, trail);
            let result: String = DeRon::deserialize_ron(&ron).unwrap_or_else(|e| {
                panic!("should be able to parse {}: {:?}", &ron, e);
            });
            assert_eq!(result, expected_string, "failed on input {}", ron);
            assert_eq!(result.chars().count(), 1);
        }
    }
}

#[test]
fn tuple_struct() {
    #[derive(DeRon, SerRon, PartialEq)]
    pub struct Test(i32, pub i32, pub(crate) String, f32);

    #[derive(DeRon, SerRon, PartialEq)]
    pub struct Vec2(pub(crate) f32, pub(crate) f32);

    let test = Test(0, 1, "asd".to_string(), 2.);
    let bytes = SerRon::serialize_ron(&test);

    let test_deserialized = DeRon::deserialize_ron(&bytes).unwrap();

    assert!(test == test_deserialized);
}

#[test]
fn array_leak_test() {
    static TOGGLED_ON_DROP: AtomicBool = AtomicBool::new(false);

    #[derive(Default, Clone, DeRon, SerRon)]
    struct IncrementOnDrop {
        inner: u64,
    }

    impl Drop for IncrementOnDrop {
        fn drop(&mut self) {
            TOGGLED_ON_DROP.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let items: [_; 2] = core::array::from_fn(|_| IncrementOnDrop::default());
    let serialized = nanoserde::SerRon::serialize_ron(&items);
    let corrupted_serialized = &serialized[..serialized.len() - 1];

    if let Ok(_) = <[IncrementOnDrop; 2] as nanoserde::DeRon>::deserialize_ron(corrupted_serialized)
    {
        panic!("Unexpected success")
    }

    assert!(TOGGLED_ON_DROP.load(std::sync::atomic::Ordering::SeqCst))
}

// https://github.com/not-fl3/nanoserde/issues/89
#[test]
fn test_deser_oversized_value() {
    use nanoserde::DeRon;

    #[derive(DeRon, Clone, PartialEq, Debug)]
    pub struct EnumConstant {
        value: i32,
    }

    let max_ron = format!(r#"(value:{})"#, i32::MAX);
    let wrap_ron = format!(r#"(value:{})"#, i32::MAX as i64 + 1);
    assert_eq!(
        <EnumConstant as DeRon>::deserialize_ron(&max_ron).unwrap(),
        EnumConstant { value: i32::MAX }
    );
    assert_eq!(
        <EnumConstant as DeRon>::deserialize_ron(&wrap_ron)
            .unwrap_err()
            .msg,
        format!(
            "Value out of range {}>{} ",
            (i32::MAX as i64 + 1).to_string(),
            i32::MAX.to_string()
        )
    );
}
