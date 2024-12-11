use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::parse::{Attribute, Category, Enum, Field, Struct, Type};

use proc_macro::TokenStream;

use crate::shared;

pub fn derive_ser_ron_proxy(proxy_type: &str, type_: &str, crate_name: &str) -> TokenStream {
    format!(
        "impl {}::SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut {}::SerRonState) {{
                let proxy: {} = self.into();
                proxy.ser_ron(d, s);
            }}
        }}",
        crate_name, type_, crate_name, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_de_ron_proxy(proxy_type: &str, type_: &str, crate_name: &str) -> TokenStream {
    format!(
        "impl {}::DeRon for {} {{
            fn de_ron(_s: &mut {}::DeRonState, i: &mut core::str::Chars) -> ::core::result::Result<Self, {}::DeRonErr> {{
                let proxy: {} = {}::DeRon::deserialize_ron(i)?;
                ::core::result::Result::Ok(Into::into(&proxy))
            }}
        }}",
        crate_name, type_, crate_name, crate_name, proxy_type, crate_name
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_ron_struct(struct_: &Struct, crate_name: &str) -> TokenStream {
    let mut s = String::new();

    for field in &struct_.fields {
        let struct_fieldname = field.field_name.clone().unwrap();
        let ron_fieldname =
            shared::attrs_rename(&field.attributes).unwrap_or_else(|| struct_fieldname.clone());
        let skip = shared::attrs_skip(&field.attributes);
        if skip {
            continue;
        }
        if field.ty.base() == "Option" {
            l!(
                s,
                "if let Some(t) = &self.{} {{
                    s.field(d+1, \"{}\");
                    t.ser_ron(d+1, s);
                    s.conl();
                }};",
                struct_fieldname,
                ron_fieldname
            );
        } else {
            l!(
                s,
                "s.field(d+1,\"{}\");
                self.{}.ser_ron(d+1, s);
                s.conl();",
                ron_fieldname,
                struct_fieldname
            );
        }
    }

    format!(
        "
        impl {}::SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut {}::SerRonState) {{
                s.st_pre();
                {}
                s.st_post(d);
            }}
        }}
    ",
        crate_name,
        struct_
            .name
            .as_ref()
            .expect("Cannot implement for anonymous struct"),
        crate_name,
        s
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_ron_struct_unnamed(struct_: &Struct, crate_name: &str) -> TokenStream {
    let mut body = String::new();

    let last = struct_.fields.len() - 1;
    for (n, _) in struct_.fields.iter().enumerate() {
        l!(body, "self.{}.ser_ron(d, s);", n);
        if n != last {
            l!(body, "s.out.push_str(\", \");");
        }
    }
    format!(
        "
        impl {}::SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut {}::SerRonState) {{
                s.out.push('(');
                {}
                s.out.push(')');
            }}
        }}",
        crate_name,
        struct_
            .name
            .as_ref()
            .expect("Cannot implement for anonymous struct"),
        crate_name,
        body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_ron_named(
    name: &String,
    fields: &Vec<Field>,
    attributes: &[Attribute],
    crate_name: &str,
) -> String {
    let mut local_vars = Vec::new();
    let mut struct_field_names = Vec::new();
    let mut ron_field_names = Vec::new();

    let container_attr_default = shared::attrs_default(attributes).is_some();

    let mut unwraps = Vec::new();
    for field in fields {
        let struct_fieldname = field.field_name.as_ref().unwrap().to_string();
        let localvar = format!("_{}", struct_fieldname);
        let field_attr_default = shared::attrs_default(&field.attributes);
        let field_attr_default_with = shared::attrs_default_with(&field.attributes);
        let default_val = if let Some(v) = field_attr_default {
            if let Some(mut val) = v {
                if field.ty.base() == "String" {
                    val = format!("\"{}\".to_string()", val)
                }
                if field.ty.base() == "Option" {
                    val = format!("Some({})", val);
                }
                Some(val)
            } else if field.ty.base() != "Option" {
                Some(String::from("Default::default()"))
            } else {
                Some(String::from("None"))
            }
        } else if let Some(mut v) = field_attr_default_with {
            v.push_str("()");
            Some(v)
        } else {
            None
        };
        let ron_fieldname =
            shared::attrs_rename(&field.attributes).unwrap_or(struct_fieldname.clone());
        let skip = crate::shared::attrs_skip(&field.attributes);

        if !skip {
            if field.ty.base() == "Option" {
                unwraps.push(format!(
                    "{{
                        if let Some(t) = {} {{
                            t
                        }} else {{
                            {}
                        }}
                    }}",
                    localvar,
                    default_val.unwrap_or_else(|| String::from("None"))
                ));
            } else if container_attr_default || default_val.is_some() {
                unwraps.push(format!(
                    "{{
                        if let Some(t) = {} {{
                            t
                        }} else {{
                            {}
                        }}
                    }}",
                    localvar,
                    default_val.unwrap_or_else(|| String::from("Default::default()"))
                ));
            } else {
                unwraps.push(format!(
                    "{{
                        if let Some(t) = {} {{
                            t
                        }} else {{
                            return Err(s.err_nf(\"{}\"))
                        }}
                    }}",
                    localvar, struct_fieldname
                ));
            }
        } else {
            unwraps.push(format!(
                "{{ {} }}",
                default_val.as_deref().unwrap_or("Default::default()")
            ));
        }

        struct_field_names.push(struct_fieldname);
        ron_field_names.push(ron_fieldname);
        local_vars.push((localvar, field.ty.full()));
    }

    let mut local_lets = String::new();

    for (local, local_type) in &local_vars {
        l!(
            local_lets,
            "let mut {}: Option<{}> = None;",
            local,
            local_type
        )
    }

    let match_names = if !ron_field_names.is_empty() {
        let mut inner = String::new();
        for (ron_field_name, (local_var, _)) in ron_field_names.iter().zip(local_vars.iter()) {
            l!(
                inner,
                "\"{}\" => {{
                    s.next_colon(i)?;
                    {} = Some({}::DeRon::de_ron(s, i)?)
                }},",
                ron_field_name,
                local_var,
                crate_name
            );
        }
        format!(
            "match s.identbuf.as_ref() {{
                {}
                _ => return ::core::result::Result::Err(s.err_exp(&s.identbuf))
            }}",
            inner
        )
    } else {
        String::new()
    };

    let mut body = String::new();

    for (field_name, unwrap) in struct_field_names.iter().zip(unwraps.iter()) {
        l!(body, "{}: {},", field_name, unwrap);
    }

    format!(
        "{{
            {}
            s.paren_open(i)?;
            while let Some(_) = s.next_ident() {{
                {}
                s.eat_comma_paren(i)?;
            }};
            s.paren_close(i)?;
            {} {{
                {}
            }}
        }}",
        local_lets, match_names, name, body
    )
}

pub fn derive_de_ron_struct(struct_: &Struct, crate_name: &str) -> TokenStream {
    let body = derive_de_ron_named(
        struct_
            .name
            .as_ref()
            .expect("Cannot implement for anonymous struct"),
        &struct_.fields,
        &struct_.attributes,
        crate_name,
    );

    format!(
        "impl {}::DeRon for {} {{
            fn de_ron(s: &mut {}::DeRonState, i: &mut core::str::Chars) -> ::core::result::Result<Self,{}::DeRonErr> {{
                ::core::result::Result::Ok({})
            }}
        }}", crate_name, struct_.name.as_ref().expect("Cannot implement for anonymous struct"), crate_name, crate_name, body)
    .parse()
    .unwrap()
}

pub fn derive_de_ron_struct_unnamed(struct_: &Struct, crate_name: &str) -> TokenStream {
    let mut body = String::new();

    for _ in &struct_.fields {
        l!(
            body,
            "{{
                let r = {}::DeRon::de_ron(s, i)?;
                s.eat_comma_paren(i)?;
                r
            }},",
            crate_name
        );
    }

    format! ("
        impl {}::DeRon for {} {{
            fn de_ron(s: &mut {}::DeRonState, i: &mut core::str::Chars) -> ::core::result::Result<Self,{}::DeRonErr> {{
                s.paren_open(i)?;
                let r = Self({});
                s.paren_close(i)?;
                ::core::result::Result::Ok(r)
            }}
        }}", crate_name, struct_.name.as_ref().expect("Cannot implement for anonymous struct"), crate_name, crate_name, body
    ).parse().unwrap()
}

pub fn derive_ser_ron_enum(enum_: &Enum, crate_name: &str) -> TokenStream {
    let mut body = String::new();

    for variant in &enum_.variants {
        let ident = &variant.field_name.clone().unwrap();
        match &variant.ty {
            Type {
                ident: Category::None,
                ..
            } => {
                // unit variant
                l!(body, "Self::{} => s.out.push_str(\"{}\"),", ident, ident)
            }
            Type {
                ident: Category::AnonymousStruct { contents },
                ..
            } => {
                let mut names = Vec::new();
                let mut inner = String::new();
                for field in contents.fields.iter() {
                    let name = field.field_name.as_ref().unwrap();
                    names.push(name.clone());
                    if field.ty.base() == "Option" {
                        l!(
                            inner,
                            "if {}.is_some() {{
                                s.field(d+1, \"{}\");
                                {}.ser_ron(d+1, s);
                                s.conl();
                            }}",
                            name.as_str(),
                            name.as_str(),
                            name.as_str()
                        )
                    } else {
                        l!(
                            inner,
                            "s.field(d+1, \"{}\");
                            {}.ser_ron(d+1, s);
                            s.conl();",
                            name,
                            name
                        )
                    }
                }
                l!(
                    body,
                    "Self::{} {{ {} }} => {{
                        s.out.push_str(\"{}\");
                        s.st_pre();
                        {}
                        s.st_post(d);
                    }}",
                    ident,
                    names.join(","),
                    ident,
                    inner
                );
            }
            Type {
                ident: Category::Tuple { contents },
                ..
            } => {
                let mut names = Vec::new();
                let mut inner = String::new();
                let last = contents.len() - 1;
                for (index, _) in &mut contents.iter().enumerate() {
                    let name = format!("f{}", index);
                    l!(inner, "{}.ser_ron(d, s);", name);
                    if index != last {
                        l!(inner, "s.out.push_str(\", \");")
                    }
                    names.push(name);
                }
                l!(
                    body,
                    "Self::{} ({}) => {{
                        s.out.push_str(\"{}\");
                        s.out.push('(');
                        {}
                        s.out.push(')');
                    }}",
                    ident,
                    names.join(","),
                    ident,
                    inner
                )
            }
            v => {
                unimplemented!("Unexpected type in enum: {:?}", v)
            }
        };
    }
    format!(
        "
        impl {}::SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut {}::SerRonState) {{
                match self {{
                    {}
                }}
            }}
        }}",
        crate_name, enum_.name, crate_name, body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_ron_enum(enum_: &Enum, crate_name: &str) -> TokenStream {
    let mut body = String::new();
    for variant in &enum_.variants {
        let ident = variant.field_name.clone().unwrap();

        match &variant.ty {
            Type {
                wraps: None,
                ident: Category::None,
                ..
            } => {
                // unit variant
                l!(body, "\"{}\" => Self::{},", ident, ident)
            }
            Type {
                ident: Category::AnonymousStruct { contents },
                ..
            } => {
                let name = format!("{}::{}", enum_.name, ident);
                let inner = derive_de_ron_named(&name, &contents.fields, &[], crate_name);
                l!(body, "\"{}\" => {}", ident, inner);
            }
            Type {
                ident: Category::Tuple { contents },
                ..
            } => {
                let mut inner = String::new();
                for _ in contents.iter() {
                    l!(
                        inner,
                        "{{
                            let r = {}::DeRon::de_ron(s, i)?;
                            s.eat_comma_paren(i)?;
                            r
                        }}, ",
                        crate_name
                    )
                }

                l!(
                    body,
                    "\"{}\" => {{
                        s.paren_open(i)?;
                        let r = Self::{} ({});
                        s.paren_close(i)?;
                        r
                    }}, ",
                    ident,
                    ident,
                    inner
                );
            }
            v => {
                unimplemented!("Unexpected type in enum: {:?}", v)
            }
        };
    }

    format! ("
        impl {}::DeRon for {} {{
            fn de_ron(s: &mut {}::DeRonState, i: &mut core::str::Chars) -> ::core::result::Result<Self,{}::DeRonErr> {{
                // we are expecting an identifier
                s.ident(i)?;
                ::core::result::Result::Ok(match s.identbuf.as_ref() {{
                    {}
                    _ => return ::core::result::Result::Err(s.err_enum(&s.identbuf))
                }})
            }}
        }}", crate_name, enum_.name, crate_name, crate_name, body).parse().unwrap()
}
