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
    pub struct Test(i32, pub i32, pub(crate) String, f32);

    #[derive(DeBin, SerBin, PartialEq)]
    pub struct Vec2(pub(crate) f32, pub(crate) f32);

    let test = Test(0, 1, "asd".to_string(), 2.);
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
        foo3: Foo
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
