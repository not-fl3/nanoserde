use alloc::format;
use alloc::string::String;

use crate::{
    parse::{Category, Enum, Struct, Type},
    shared::{enum_bounds_strings, struct_bounds_strings},
};

use proc_macro::TokenStream;

pub fn derive_ser_bin_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl SerBin for {} {{
            fn ser_bin(&self, s: &mut Vec<u8>) {{
                let proxy: {} = self.into();
                proxy.ser_bin(s);
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_de_bin_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl DeBin for {} {{
            fn de_bin(o:&mut usize, d:&[u8]) -> ::core::result::Result<Self, nanoserde::DeBinErr> {{
                let proxy: {} = DeBin::de_bin(o, d)?;
                ::core::result::Result::Ok(Into::into(&proxy))
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_bin_struct(struct_: &Struct) -> TokenStream {
    let mut body = String::new();
    let (generic_w_bounds, generic_no_bounds) = struct_bounds_strings(struct_, "SerBin");

    for field in &struct_.fields {
        if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
            l!(
                body,
                "let proxy: {} = Into::into(&self.{});",
                proxy,
                field.field_name.as_ref().unwrap()
            );
            l!(body, "proxy.ser_bin(s);");
        } else {
            l!(
                body,
                "self.{}.ser_bin(s);",
                field.field_name.as_ref().unwrap()
            );
        }
    }
    format!(
        "impl{} SerBin for {}{} {{
            fn ser_bin(&self, s: &mut Vec<u8>) {{
                {}
            }}
        }}",
        generic_w_bounds,
        struct_
            .name
            .as_ref()
            .expect("Shouldnt have an anonymous struct here"),
        generic_no_bounds,
        body
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_bin_struct_unnamed(struct_: &Struct) -> TokenStream {
    let mut body = String::new();
    let (generic_w_bounds, generic_no_bounds) = struct_bounds_strings(struct_, "SerBin");

    for (n, field) in struct_.fields.iter().enumerate() {
        if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
            l!(body, "let proxy: {} = Into::into(&self.{});", proxy, n);
            l!(body, "proxy.ser_bin(s);");
        } else {
            l!(body, "self.{}.ser_bin(s);", n);
        }
    }
    format!(
        "impl{} SerBin for {}{} {{
            fn ser_bin(&self, s: &mut Vec<u8>) {{
                {}
            }}
        }}",
        generic_w_bounds,
        struct_
            .name
            .as_ref()
            .expect("Shouldnt have an anonymous struct here"),
        generic_no_bounds,
        body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_bin_struct(struct_: &Struct) -> TokenStream {
    let mut body = String::new();
    let (generic_w_bounds, generic_no_bounds) = struct_bounds_strings(struct_, "DeBin");

    for field in &struct_.fields {
        if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
            l!(body, "{}: {{", field.field_name.as_ref().unwrap());
            l!(body, "let proxy: {} = DeBin::de_bin(o, d)?;", proxy);
            l!(body, "Into::into(&proxy)");
            l!(body, "},")
        } else {
            l!(
                body,
                "{}: DeBin::de_bin(o, d)?,",
                field.field_name.as_ref().unwrap()
            );
        }
    }

    format!(
        "impl{} DeBin for {}{} {{
            fn de_bin(o:&mut usize, d:&[u8]) -> ::core::result::Result<Self, nanoserde::DeBinErr> {{
                ::core::result::Result::Ok(Self {{
                    {}
                }})
            }}
        }}",
        generic_w_bounds,
        struct_
            .name
            .as_ref()
            .expect("Shouldnt have an anonymous struct here"),
        generic_no_bounds,
        body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_bin_struct_unnamed(struct_: &Struct) -> TokenStream {
    let mut body = String::new();
    let (generic_w_bounds, generic_no_bounds) = struct_bounds_strings(struct_, "DeBin");

    for (n, field) in struct_.fields.iter().enumerate() {
        if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
            l!(body, "{}: {{", n);
            l!(body, "let proxy: {} = DeBin::de_bin(o, d)?;", proxy);
            l!(body, "Into::into(&proxy)");
            l!(body, "},")
        } else {
            l!(body, "{}: DeBin::de_bin(o, d)?,", n);
        }
    }

    format!(
        "impl{} DeBin for {}{} {{
            fn de_bin(o:&mut usize, d:&[u8]) -> ::core::result::Result<Self, nanoserde::DeBinErr> {{
                ::core::result::Result::Ok(Self {{
                    {}
                }})
            }}
        }}",
        generic_w_bounds,
        struct_
            .name
            .as_ref()
            .expect("Shouldnt have an anonymous struct here"),
        generic_no_bounds,
        body
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_bin_enum(enum_: &Enum) -> TokenStream {
    let mut r = String::new();
    let (generic_w_bounds, generic_no_bounds) = enum_bounds_strings(enum_, "SerBin");

    for (index, variant) in enum_.variants.iter().enumerate() {
        let lit = format!("{}u16", index);
        let ident = variant
            .field_name
            .as_ref()
            .expect("Unnamed enum fields are illegal");
        // Unit
        match &variant.ty {
            Type {
                wraps: None,
                ident: Category::None,
                ..
            } => {
                // unit variant
                l!(r, "Self::{} => {}.ser_bin(s),", ident, lit);
            }
            Type {
                ident: Category::Tuple { contents },
                ..
            } => {
                l!(r, "Self::{} (", ident);
                for (n, _) in contents.iter().enumerate() {
                    l!(r, "f{}, ", n)
                }
                l!(r, ") => {");
                l!(r, "{}.ser_bin(s);", lit);
                for (n, _) in contents.iter().enumerate() {
                    l!(r, "f{}.ser_bin(s);", n)
                }
                l!(r, "}")
            }
            Type {
                ident: Category::AnonymousStruct { contents },
                ..
            } => {
                l!(r, "Self::{} {{", ident);
                for (_, f) in contents.fields.iter().enumerate() {
                    l!(
                        r,
                        "{}, ",
                        f.field_name.as_ref().expect("field must be named")
                    )
                }

                l!(r, "} => {");
                l!(r, "{}.ser_bin(s);", lit);
                for (_, f) in contents.fields.iter().enumerate() {
                    l!(
                        r,
                        "{}.ser_bin(s);",
                        f.field_name.as_ref().expect("field must be named")
                    )
                }
                l!(r, "}")
            }
            v => {
                unimplemented!("Unexpected type in enum: {:?}", v)
            }
        };
    }

    format!(
        "impl{} SerBin for {}{} {{
            fn ser_bin(&self, s: &mut Vec<u8>) {{
                match self {{
                  {}
                }}
            }}
        }}",
        generic_w_bounds, enum_.name, generic_no_bounds, r
    )
    .parse()
    .unwrap()
}

pub fn derive_de_bin_enum(enum_: &Enum) -> TokenStream {
    let mut r = String::new();
    let (generic_w_bounds, generic_no_bounds) = enum_bounds_strings(enum_, "DeBin");

    for (index, variant) in enum_.variants.iter().enumerate() {
        let lit = format!("{}u16", index);

        match &variant.ty {
            Type {
                wraps: None,
                ident: Category::None,
                ..
            } => {
                // unit variant
                l!(
                    r,
                    "{} => Self::{},",
                    lit,
                    variant.field_name.as_ref().unwrap()
                )
            }
            Type {
                ident: Category::Tuple { contents },
                ..
            } => {
                l!(
                    r,
                    "{} => Self::{} (",
                    lit,
                    variant.field_name.as_ref().unwrap()
                );
                for _ in contents {
                    l!(r, "DeBin::de_bin(o, d)?,",);
                }
                l!(r, "),")
            }
            Type {
                ident: Category::AnonymousStruct { contents },
                ..
            } => {
                l!(
                    r,
                    "{} => Self::{} {{",
                    lit,
                    variant.field_name.as_ref().unwrap()
                );
                for f in contents.fields.iter() {
                    l!(
                        r,
                        "{}: DeBin::de_bin(o, d)?,",
                        f.field_name.as_ref().unwrap()
                    );
                }
                l!(r, "},");
            }
            v => {
                unimplemented!("Unexpected type in enum: {:?}", v)
            }
        };
    }

    format!(
        "impl{}  DeBin for {}{} {{
            fn de_bin(o:&mut usize, d:&[u8]) -> ::core::result::Result<Self, nanoserde::DeBinErr> {{
                let id: u16 = DeBin::de_bin(o,d)?;
                Ok(match id {{
                    {}
                    _ => return ::core::result::Result::Err(nanoserde::DeBinErr::new(*o, 0, d.len()))
                }})
            }}
        }}", generic_w_bounds,enum_.name,generic_no_bounds, r)
        .parse()
        .unwrap()
}
