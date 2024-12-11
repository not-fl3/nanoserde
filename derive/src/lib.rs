#![no_std]

extern crate alloc;
extern crate proc_macro;

#[cfg(any(feature = "json", feature = "ron", feature = "binary"))]
#[macro_use]
mod shared;

#[cfg(feature = "binary")]
mod serde_bin;
#[cfg(feature = "binary")]
use crate::serde_bin::*;

#[cfg(feature = "ron")]
mod serde_ron;
#[cfg(feature = "ron")]
use crate::serde_ron::*;

#[cfg(feature = "json")]
mod serde_json;
#[cfg(feature = "json")]
use crate::serde_json::*;

#[cfg(any(feature = "json", feature = "ron", feature = "binary"))]
mod parse;

#[cfg(feature = "binary")]
#[proc_macro_derive(SerBin, attributes(nserde))]
pub fn derive_ser_bin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    let crate_name = shared::attrs_crate(input.attributes()).unwrap_or("nanoserde");

    if let Some(proxy) = shared::attrs_proxy(input.attributes()) {
        return derive_ser_bin_proxy(&proxy, input.name(), crate_name);
    }

    // ok we have an ident, its either a struct or a enum
    match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_ser_bin_struct(struct_, crate_name),
        parse::Data::Struct(struct_) => derive_ser_bin_struct_unnamed(struct_, crate_name),
        parse::Data::Enum(enum_) => derive_ser_bin_enum(enum_, crate_name),
        _ => unimplemented!("Only structs and enums are supported"),
    }
}

#[cfg(feature = "binary")]
#[proc_macro_derive(DeBin, attributes(nserde))]
pub fn derive_de_bin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    let crate_name = shared::attrs_crate(input.attributes()).unwrap_or("nanoserde");

    if let Some(proxy) = shared::attrs_proxy(input.attributes()) {
        return derive_de_bin_proxy(&proxy, input.name(), crate_name);
    }

    // ok we have an ident, its either a struct or a enum
    match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_de_bin_struct(struct_, crate_name),
        parse::Data::Struct(struct_) => derive_de_bin_struct_unnamed(struct_, crate_name),
        parse::Data::Enum(enum_) => derive_de_bin_enum(enum_, crate_name),

        _ => unimplemented!("Only structs and enums are supported"),
    }
}

#[cfg(feature = "ron")]
#[proc_macro_derive(SerRon, attributes(nserde))]
pub fn derive_ser_ron(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    let crate_name = shared::attrs_crate(input.attributes()).unwrap_or("nanoserde");

    if let Some(proxy) = shared::attrs_proxy(input.attributes()) {
        return derive_ser_ron_proxy(&proxy, input.name(), crate_name);
    }

    // ok we have an ident, its either a struct or a enum
    match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_ser_ron_struct(struct_, crate_name),
        parse::Data::Struct(struct_) => derive_ser_ron_struct_unnamed(struct_, crate_name),
        parse::Data::Enum(enum_) => derive_ser_ron_enum(enum_, crate_name),
        _ => unimplemented!("Only structs and enums are supported"),
    }
}

#[cfg(feature = "ron")]
#[proc_macro_derive(DeRon, attributes(nserde))]
pub fn derive_de_ron(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    let crate_name = shared::attrs_crate(input.attributes()).unwrap_or("nanoserde");

    if let Some(proxy) = shared::attrs_proxy(input.attributes()) {
        return derive_de_ron_proxy(&proxy, input.name(), crate_name);
    }

    // ok we have an ident, its either a struct or a enum
    match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_de_ron_struct(struct_, crate_name),
        parse::Data::Struct(struct_) => derive_de_ron_struct_unnamed(struct_, crate_name),
        parse::Data::Enum(enum_) => derive_de_ron_enum(enum_, crate_name),
        _ => unimplemented!("Only structs and enums are supported"),
    }
}

#[cfg(feature = "json")]
#[proc_macro_derive(SerJson, attributes(nserde))]
pub fn derive_ser_json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    let crate_name = shared::attrs_crate(input.attributes()).unwrap_or("nanoserde");

    if let Some(proxy) = shared::attrs_proxy(input.attributes()) {
        return derive_ser_json_proxy(&proxy, input.name(), crate_name);
    }

    // ok we have an ident, its either a struct or a enum
    match &input {
        parse::Data::Struct(struct_) if struct_.named => {
            derive_ser_json_struct(struct_, crate_name)
        }
        parse::Data::Struct(struct_) => derive_ser_json_struct_unnamed(struct_, crate_name),
        parse::Data::Enum(enum_) => derive_ser_json_enum(enum_, crate_name),
        _ => unimplemented!(""),
    }
}

#[cfg(feature = "json")]
#[proc_macro_derive(DeJson, attributes(nserde))]
pub fn derive_de_json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    let crate_name = shared::attrs_crate(input.attributes()).unwrap_or("nanoserde");

    if let Some(proxy) = shared::attrs_proxy(input.attributes()) {
        return derive_de_json_proxy(&proxy, input.name(), crate_name);
    }

    // ok we have an ident, its either a struct or a enum
    match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_de_json_struct(struct_, crate_name),
        parse::Data::Struct(struct_) => derive_de_json_struct_unnamed(struct_, crate_name),
        parse::Data::Enum(enum_) => derive_de_json_enum(enum_, crate_name),
        parse::Data::Union(_) => unimplemented!("Unions are not supported"),
    }
}
