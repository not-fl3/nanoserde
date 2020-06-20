# nanoserde

Fork of https://crates.io/crates/makepad-tinyserde with all the dependencies removed.
No more syn, proc_macro2 or quote in the build tree!

```
> cargo tree
nanoserde v0.1.0 (/../nanoserde)
└── nanoserde-derive v0.1.0 (/../nanoserde/derive)
```

Work in progress, right now only jsons deserialization is implemented.

And this is going to be even more restricted and limited serialization/deserialization library than makepad-tinyserde. 
Generic bounds, lifetime bounds, where clauses and probably a lot more is not supported and probably will never be supported.

This is used in [macroquad](github.com/not-fl3/macroquad/) game engine and only features needed for macroquad's internal serialization needs are going to be well supported. 

