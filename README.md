# nanoserde

Fork of https://crates.io/crates/makepad-tinyserde with all the dependencies removed.
No more syn, proc_macro2 or quote in the build tree!

```
> cargo tree
nanoserde v0.1.0 (/../nanoserde)
└── nanoserde-derive v0.1.0 (/../nanoserde/derive)
```

Work in progress, features that works:
- json: serialization/deserialization 
- json: containers supported: Struct, Tuple Struct
- json: field types supported: HashMap, Vec, Option, String, i\*, f\*
- json: field attributes: `#[nserde(default)]`, `#[nserde(rename = "")]`, `#[nserde(proxy = "")]`
- json: container attributes: `#[nserde(default)]`, `#[nserde(proxy = "")]` 

- binary: serialization/deserialization 
- binary: containers supported: Struct
- binary: field types supported: HashMap, Vec, Option, String, i\*, f\*
- binary: field attributes `#[nserde(proxy = "")]`
- binary: container attributes `#[nserde(proxy = "")]`

And this is going to be even more restricted and limited serialization/deserialization library than makepad-tinyserde. 
Generic bounds, lifetime bounds, where clauses and probably a lot more is not supported and probably will never be supported.

This is used in [macroquad](https://github.com/not-fl3/macroquad/) game engine and only features needed for macroquad's internal serialization needs are going to be well supported. 

