#![cfg(any(feature = "binary", feature = "json", feature = "ron"))]

extern crate alloc;

#[cfg(feature = "binary")]
use nanoserde::{DeBin, SerBin};
#[cfg(feature = "json")]
use nanoserde::{DeJson, SerJson};
#[cfg(feature = "ron")]
use nanoserde::{DeRon, SerRon};

use alloc::collections::BTreeMap;

#[test]
fn ser_de() {
    #[derive(PartialEq, Debug)]
    #[cfg_attr(feature = "binary", derive(DeBin, SerBin))]
    #[cfg_attr(feature = "json", derive(DeJson, SerJson))]
    #[cfg_attr(feature = "ron", derive(DeRon, SerRon))]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
        e: Option<BTreeMap<String, String>>,
        f: Option<([u32; 4], String)>,
        g: (),
        h: f64,
    }

    let mut map = BTreeMap::new();
    map.insert("a".to_string(), "b".to_string());

    let test: Test = Test {
        a: 1,
        b: 2.718281828459045,
        c: Some("asd".to_string()),
        d: None,
        e: Some(map),
        f: Some(([1, 2, 3, 4], "tuple".to_string())),
        g: (),
        h: 1e30,
    };

    #[cfg(feature = "binary")]
    {
        let bytes = SerBin::serialize_bin(&test);
        let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();
        assert_eq!(test, test_deserialized);
    }

    #[cfg(feature = "json")]
    {
        let bytes = SerJson::serialize_json(&test);
        let test_deserialized = DeJson::deserialize_json(&bytes).unwrap();
        assert_eq!(test, test_deserialized);
    }

    #[cfg(feature = "ron")]
    {
        let bytes = SerRon::serialize_ron(&test);
        let test_deserialized = DeRon::deserialize_ron(&bytes).unwrap();
        assert_eq!(test, test_deserialized);
    }
}
