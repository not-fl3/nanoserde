use nanoserde::{DeBin, DeJson, DeRon, SerBin, SerJson, SerRon};

#[cfg(feature = "no_std")]
use hashbrown::HashMap;

#[cfg(not(feature = "no_std"))]
use std::collections::HashMap;

#[test]
fn ser_de() {
    #[derive(DeBin, SerBin, DeJson, SerJson, DeRon, SerRon, PartialEq, Debug)]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
        e: Option<HashMap<String, String>>,
        f: Option<([u32; 4], String)>,
        g: (),
    }

    let mut map = HashMap::new();
    map.insert("a".to_string(), "b".to_string());

    let test: Test = Test {
        a: 1,
        b: 2.,
        c: Some("asd".to_string()),
        d: None,
        e: Some(map),
        f: Some(([1, 2, 3, 4], "tuple".to_string())),
        g: (),
    };

    let bytes = SerBin::serialize_bin(&test);
    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();
    assert_eq!(test, test_deserialized);

    let bytes = SerJson::serialize_json(&test);
    let test_deserialized = DeJson::deserialize_json(&bytes).unwrap();
    assert_eq!(test, test_deserialized);

    let bytes = SerRon::serialize_ron(&test);
    let test_deserialized = DeRon::deserialize_ron(&bytes).unwrap();
    assert_eq!(test, test_deserialized);
}
