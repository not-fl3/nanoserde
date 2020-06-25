extern crate proc_macro;

#[macro_use]
mod shared;

mod serde_bin;
use crate::serde_bin::*;

//mod serde_ron;
//use crate::serde_ron::*;

mod serde_json;

mod parse;

use crate::serde_json::*;

#[proc_macro_derive(SerBin, attributes(nserde))]
pub fn derive_ser_bin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_ser_bin_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_ser_bin_struct(struct_),
        parse::Data::Struct(struct_) => derive_ser_bin_struct_unnamed(struct_),
        _ => unimplemented!("Only structs are supported"),
    };

    ts
}

#[proc_macro_derive(DeBin, attributes(nserde))]
pub fn derive_de_bin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_de_bin_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_de_bin_struct(struct_),
        parse::Data::Struct(struct_) => derive_de_bin_struct_unnamed(struct_),

        _ => unimplemented!("Only structs are supported"),
    };

    ts
}

#[proc_macro_derive(SerRon, attributes(nserde))]
pub fn derive_ser_ron(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = parse_macro_input!(input as DeriveInput);
    // // ok we have an ident, its either a struct or a enum
    // let ts = match &input.data {
    //     Data::Struct(DataStruct {fields: Fields::Named(fields), ..}) => {
    //         derive_ser_ron_struct(&input, fields)
    //     },
    //     Data::Struct(DataStruct {fields: Fields::Unnamed(fields), ..}) => {
    //         derive_ser_ron_struct_unnamed(&input, fields)
    //     },
    //     Data::Enum(enumeration) => {
    //         derive_ser_ron_enum(&input, enumeration)
    //     },
    //     _ => error(Span::call_site(), "only structs or enums supported")
    // };
    // proc_macro::TokenStream::from(ts)

    unimplemented!()
}

#[proc_macro_derive(DeRon)]
pub fn derive_de_ron(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = parse_macro_input!(input as DeriveInput);
    // // ok we have an ident, its either a struct or a enum
    // let ts = match &input.data {
    //     Data::Struct(DataStruct {fields: Fields::Named(fields), ..}) => {
    //         derive_de_ron_struct(&input, fields)
    //     },
    //     Data::Struct(DataStruct {fields: Fields::Unnamed(fields), ..}) => {
    //         derive_de_ron_struct_unnamed(&input, fields)
    //     },
    //     Data::Enum(enumeration) => {
    //         derive_de_ron_enum(&input, enumeration)
    //     },
    //     _ => error(Span::call_site(), "only structs or enums supported")
    // };
    // //println!("{}", ts.to_string());
    // proc_macro::TokenStream::from(ts)

    unimplemented!()
}

#[proc_macro_derive(SerJson, attributes(nserde))]
pub fn derive_ser_json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_ser_json_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_ser_json_struct(struct_),
        //parse::Data::Struct(struct_) => derive_ser_json_struct_unnamed(struct_),
        _ => unimplemented!("Only named structs are supported"),
    };

    ts
}

#[proc_macro_derive(DeJson, attributes(nserde))]
pub fn derive_de_json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_de_json_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) => derive_de_json_struct(struct_),
        _ => unimplemented!("Only structs are supported"),
    };

    ts
}
