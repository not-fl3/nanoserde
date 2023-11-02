use std::{
    array,
    collections::{BTreeSet, LinkedList},
    sync::atomic::AtomicBool,
};

#[cfg(feature = "no_std")]
use hashbrown::{HashMap, HashSet};

#[cfg(not(feature = "no_std"))]
use std::collections::{HashMap, HashSet};

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
    #[derive(DeBin, SerBin, PartialEq)]
    struct TestGenericsWSkip<A, B, C, DD: Eq + std::hash::Hash, EE>
    where
        EE: Eq + std::hash::Hash,
    {
        test1: A,
        test2: B,
        test3: C,
        #[nserde(skip)]
        test4: HashMap<DD, A>,
        test5: HashMap<EE, A>,
    }

    let test: TestGenericsWSkip<i32, usize, String, Option<bool>, u128> = TestGenericsWSkip {
        test1: 0,
        test2: 42,
        test3: String::from("test123"),
        test4: vec![(Some(true), 1)].into_iter().collect(),
        test5: vec![(15_u128, 1)].into_iter().collect(),
    };

    let bytes = SerBin::serialize_bin(&test);

    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);

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
    pub struct Serializable<'a> where Self: 'a {
        foo: &'a i32,
    }

    #[derive(DeBin)]
    pub struct DeSerializable {
        foo: i32,
    }

    let foo_base = 42;

    let test = Serializable {
        foo: &42
    };

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
        pub c: HashSet<i32>,
        pub d: BTreeSet<i32>,
    }

    let test: Test = Test {
        a: vec![1, 2, 3],
        b: vec![1.0, 2.0, 3.0, 4.0].into_iter().collect(),
        c: vec![1, 2, 3, 4, 5].into_iter().collect(),
        d: vec![1, 2, 3, 4, 5, 6].into_iter().collect(),
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
