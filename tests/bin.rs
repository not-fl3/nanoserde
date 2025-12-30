#![cfg(feature = "binary")]

extern crate alloc;

use std::{array, ops::Range, sync::atomic::AtomicBool};

use alloc::collections::{BTreeMap, BTreeSet, LinkedList};

use nanoserde::{DeBin, SerBin};

#[test]
fn binary() {
    #[derive(DeBin, SerBin, PartialEq)]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
    }

    let test: Test = Test {
        a: 1,
        b: 2.,
        c: Some("asd".to_string()),
        d: None,
    };

    let bytes = SerBin::serialize_bin(&test);

    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
}

#[test]
fn binary_generics() {
    #[derive(DeBin, SerBin, PartialEq, Debug)]
    struct TestGenericsWSkip<A, B, C, DD: Eq + Ord, EE>
    where
        EE: Eq + Ord,
    {
        test1: A,
        test2: B,
        test3: C,
        #[nserde(skip)]
        test4: BTreeMap<DD, A>,
        test5: BTreeMap<EE, A>,
    }

    let test: TestGenericsWSkip<i32, usize, String, Option<bool>, u128> = TestGenericsWSkip {
        test1: 0,
        test2: 42,
        test3: String::from("test123"),
        test4: vec![(Some(true), 1)].into_iter().collect(),
        test5: vec![(15_u128, 1)].into_iter().collect(),
    };

    let bytes = SerBin::serialize_bin(&test);

    let mut test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    // show that everything is equal, except for the skipped field
    assert_ne!(test, test_deserialized);
    test_deserialized.test4 = test.test4.clone();
    assert_eq!(test, test_deserialized);

    #[derive(DeBin, SerBin, PartialEq)]
    pub enum TestEnum<A, B> {
        Test1(A, B),
        Test2(B, A),
    }

    let test: TestEnum<i32, f32> = TestEnum::Test1(3, 15.);

    let bytes = SerBin::serialize_bin(&test);

    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
}

#[test]
fn field_proxy() {
    #[derive(PartialEq)]
    pub struct NonSerializable {
        foo: i32,
    }

    #[derive(DeBin, SerBin, PartialEq)]
    pub struct Serializable {
        x: i32,
    }

    impl Into<NonSerializable> for &Serializable {
        fn into(self) -> NonSerializable {
            NonSerializable { foo: self.x }
        }
    }
    impl Into<Serializable> for &NonSerializable {
        fn into(self) -> Serializable {
            Serializable { x: self.foo }
        }
    }

    #[derive(DeBin, SerBin, PartialEq)]
    pub struct Test {
        #[nserde(proxy = "Serializable")]
        foo: NonSerializable,
    }

    let test = Test {
        foo: NonSerializable { foo: 6 },
    };

    let bytes = SerBin::serialize_bin(&test);
    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
}

#[test]
fn field_ignore_self_bound() {
    #[derive(SerBin)]
    pub struct Serializable<'a>
    where
        Self: 'a,
    {
        foo: &'a i32,
    }

    #[derive(DeBin)]
    pub struct DeSerializable {
        foo: i32,
    }

    let foo_base = 42;

    let test = Serializable { foo: &42 };

    let bytes = SerBin::serialize_bin(&test);
    let deser: DeSerializable = DeBin::deserialize_bin(&bytes).unwrap();
    assert_eq!(foo_base, deser.foo);
}

#[test]
fn struct_proxy() {
    #[derive(PartialEq, Debug)]
    struct NonSerializable<T: PartialEq> {
        s: T,
    }

    #[derive(DeBin, SerBin, PartialEq, Debug)]
    #[nserde(proxy = "PortableVec2")]
    pub struct SimdVec2 {
        simd_data: NonSerializable<u64>,
    }

    #[derive(DeBin, SerBin, PartialEq, Debug)]
    pub struct PortableVec2 {
        x: u64,
        y: u64,
    }

    impl Into<SimdVec2> for &PortableVec2 {
        fn into(self) -> SimdVec2 {
            SimdVec2 {
                simd_data: NonSerializable { s: self.x + self.y },
            }
        }
    }
    impl Into<PortableVec2> for &SimdVec2 {
        fn into(self) -> PortableVec2 {
            PortableVec2 {
                x: self.simd_data.s / 2,
                y: self.simd_data.s / 2 + self.simd_data.s % 2,
            }
        }
    }

    let test = SimdVec2 {
        simd_data: NonSerializable { s: 23 },
    };

    let bytes = SerBin::serialize_bin(&test);
    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
}

#[test]
fn tuple_struct() {
    #[derive(DeBin, SerBin, PartialEq)]
    pub struct Test(i32, pub i32, pub(crate) String, f32, [u64; 100]);

    #[derive(DeBin, SerBin, PartialEq)]
    pub struct Vec2(pub(crate) f32, pub(crate) f32);

    let test = Test(0, 1, "asd".to_string(), 2., [3_u64; 100]);
    let bytes = SerBin::serialize_bin(&test);

    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
}

#[test]
fn enums() {
    #[derive(DeBin, SerBin, PartialEq, Debug)]
    pub enum Foo {
        A,
        B(i32),
        C { x: String },
    }

    #[derive(DeBin, SerBin, PartialEq, Debug)]
    pub struct Test {
        foo1: Foo,
        foo2: Foo,
        foo3: Foo,
    }

    let test: Test = Test {
        foo1: Foo::A,
        foo2: Foo::B(23),
        foo3: Foo::C {
            x: "qwe".to_string(),
        },
    };

    let bytes = SerBin::serialize_bin(&test);

    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
    assert_eq!(test_deserialized.foo1, Foo::A);
    assert_eq!(test_deserialized.foo2, Foo::B(23));
    assert_eq!(
        test_deserialized.foo3,
        Foo::C {
            x: "qwe".to_string()
        }
    );
}

#[test]
fn pub_tuple_struct() {
    #[derive(DeBin, SerBin, PartialEq)]
    struct Foo(pub [u8; 3]);
}

#[test]
fn collections() {
    #[derive(DeBin, SerBin, PartialEq, Debug)]
    pub struct Test {
        pub a: Vec<i32>,
        pub b: LinkedList<f32>,
        pub c: BTreeMap<i32, i32>,
        pub d: BTreeSet<i32>,
        pub e: Range<i32>,
    }

    let test: Test = Test {
        a: vec![1, 2, 3],
        b: vec![1.0, 2.0, 3.0, 4.0].into_iter().collect(),
        c: vec![(1, 2), (3, 4)].into_iter().collect(),
        d: vec![1, 2, 3, 4, 5, 6].into_iter().collect(),
        e: 1..3,
    };

    let bytes = SerBin::serialize_bin(&test);

    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert_eq!(test, test_deserialized);
}

#[test]
fn array_leak_test() {
    static TOGGLED_ON_DROP: AtomicBool = AtomicBool::new(false);

    #[derive(Default, Clone, SerBin, DeBin)]
    struct IncrementOnDrop {
        inner: u128,
    }

    impl Drop for IncrementOnDrop {
        fn drop(&mut self) {
            TOGGLED_ON_DROP.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let items: [_; 2] = array::from_fn(|_| IncrementOnDrop::default());
    let serialized = nanoserde::SerBin::serialize_bin(&items);
    let corrupted_serialized = &serialized[..serialized.len() - 1];

    if let Ok(_) = <[IncrementOnDrop; 2] as nanoserde::DeBin>::deserialize_bin(corrupted_serialized)
    {
        panic!("Unexpected success")
    }

    assert!(TOGGLED_ON_DROP.load(std::sync::atomic::Ordering::SeqCst))
}

#[test]
fn binary_crate() {
    use nanoserde as renamed;
    #[derive(renamed::DeBin, renamed::SerBin, PartialEq)]
    #[nserde(crate = "renamed")]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
    }

    let test: Test = Test {
        a: 1,
        b: 2.,
        c: Some("asd".to_string()),
        d: None,
    };

    let bytes = renamed::SerBin::serialize_bin(&test);

    let test_deserialized = renamed::DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
}

#[test]
fn generic_enum() {
    #[derive(DeBin, SerBin, PartialEq, Debug)]
    pub enum Foo<T, U>
    where
        T: Copy,
        U: Clone,
    {
        A,
        B(T, String),
        C { a: U, b: String },
    }

    #[derive(DeBin, SerBin, PartialEq, Debug)]
    pub struct Bar<T, U>
    where
        T: Copy,
        U: Clone,
    {
        foo1: Foo<T, U>,
        foo2: Foo<T, U>,
        foo3: Foo<T, U>,
    }

    let test = Bar {
        foo1: Foo::A,
        foo2: Foo::B(1, "asd".to_string()),
        foo3: Foo::C {
            a: 2,
            b: "qwe".to_string(),
        },
    };
    let bytes = SerBin::serialize_bin(&test);
    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();
    assert!(test == test_deserialized);
}

#[cfg(feature = "std")]
#[test]
fn std_time() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // Deserialize a known value
    let known_bytes = [
        42, 0, 0, 0, 0, 0, 0, 0, // seconds
        123, 0, 0, 0, // nanoseconds
    ];
    let deserialized: Duration = DeBin::deserialize_bin(&known_bytes).unwrap();
    assert_eq!(deserialized, Duration::new(42, 123));

    // Duration round trip
    let durations = [
        Duration::new(0, 0),
        Duration::new(42, 123_456_789),
        Duration::new(u64::MAX, 999_999_999),
    ];
    for dur in durations {
        let bytes = SerBin::serialize_bin(&dur);
        let deserialized: Duration = DeBin::deserialize_bin(&bytes).unwrap();
        assert_eq!(dur, deserialized);
    }

    // Not enough data
    assert!(Duration::deserialize_bin(&[1, 2, 3]).is_err()); // Truncated

    // Nanos too large
    let invalid_nanos = {
        let mut b = Vec::new();
        42u64.ser_bin(&mut b);
        1_000_000_001u32.ser_bin(&mut b);
        b
    };
    assert!(Duration::deserialize_bin(&invalid_nanos).is_err()); // Invalid nanos

    // SystemTime round trip
    let times = [
        UNIX_EPOCH,
        UNIX_EPOCH + Duration::new(42, 0),
        UNIX_EPOCH + Duration::new(1_640_995_200, 500_000_000),
    ];
    for time in times {
        let bytes = SerBin::serialize_bin(&time);
        let deserialized: SystemTime = DeBin::deserialize_bin(&bytes).unwrap();
        assert_eq!(time, deserialized);
    }

    // Empty and not enough data
    assert!(SystemTime::deserialize_bin(&[]).is_err());
    assert!(SystemTime::deserialize_bin(&[1, 2, 3]).is_err());
    let none_bytes = {
        let mut b = Vec::new();
        0u8.ser_bin(&mut b);
        b
    };
    assert_eq!(
        SystemTime::deserialize_bin(&none_bytes).unwrap(),
        UNIX_EPOCH
    );

    // Combined round trip
    #[derive(DeBin, SerBin, PartialEq, Debug)]
    struct Test {
        duration: Duration,
        system_time: SystemTime,
    }

    let test = Test {
        duration: Duration::new(42, 0),
        system_time: UNIX_EPOCH + Duration::new(42, 0),
    };
    let bytes = SerBin::serialize_bin(&test);
    let deserialized = DeBin::deserialize_bin(&bytes).unwrap();
    assert_eq!(test, deserialized);

    let bytes = SerBin::serialize_bin(&None::<SystemTime>);
    let deserialized: SystemTime = DeBin::deserialize_bin(&bytes).unwrap();
    assert_eq!(deserialized, UNIX_EPOCH);
}
