use crate::parse::{Attribute, Enum, Field, Struct};

use proc_macro::TokenStream;

use crate::shared;

pub fn derive_ser_ron_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut nanoserde::SerRonState) {{
                let proxy: {} = self.into();
                proxy.ser_ron(d, s);
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_de_ron_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl DeRon for {} {{
            fn de_ron(_s: &mut nanoserde::DeRonState, i: &mut std::str::Chars) -> std::result::Result<Self, nanoserde::DeRonErr> {{
                let proxy: {} = DeRon::deserialize_ron(i)?;
                std::result::Result::Ok(Into::into(&proxy))
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_ron_struct(struct_: &Struct) -> TokenStream {
    let mut s = String::new();

    for field in &struct_.fields {
        let struct_fieldname = field.field_name.clone().unwrap();
        let ron_fieldname =
            shared::attrs_rename(&field.attributes).unwrap_or_else(|| struct_fieldname.clone());
        if field.ty.is_option {
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
        impl SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut nanoserde::SerRonState) {{
                s.st_pre();
                {}
                s.st_post(d);
            }}
        }}
    ",
        struct_.name, s
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_ron_struct_unnamed(struct_: &Struct) -> TokenStream {
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
        impl SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut nanoserde::SerRonState) {{
                s.out.push('(');
                {}
                s.out.push(')');
            }}
        }}",
        struct_.name, body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_ron_named(
    name: &String,
    fields: &Vec<Field>,
    attributes: &Vec<Attribute>,
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
                if field.ty.path == "String" {
                    val = format!("\"{}\".to_string()", val)
                }
                if field.ty.is_option {
                    val = format!("Some({})", val);
                }
                Some(val)
            } else {
                if !field.ty.is_option {
                    Some(String::from("Default::default()"))
                } else {
                    Some(String::from("None"))
                }
            }
        } else if let Some(mut v) = field_attr_default_with {
            v.push_str("()");
            Some(v)
        } else {
            None
        };
        let ron_fieldname =
            shared::attrs_rename(&field.attributes).unwrap_or(struct_fieldname.clone());

        if field.ty.is_option {
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

        struct_field_names.push(struct_fieldname);
        ron_field_names.push(ron_fieldname);
        local_vars.push(localvar);
    }

    let mut local_lets = String::new();

    for local in &local_vars {
        l!(local_lets, "let mut {} = None;", local)
    }

    let match_names = if ron_field_names.len() != 0 {
        let mut inner = String::new();
        for (ron_field_name, local_var) in ron_field_names.iter().zip(local_vars.iter()) {
            l!(
                inner,
                "\"{}\" => {{
                    s.next_colon(i)?;
                    {} = Some(DeRon::de_ron(s, i)?)
                }},",
                ron_field_name,
                local_var
            );
        }
        format!(
            "match s.identbuf.as_ref() {{
                {}
                _ => return std::result::Result::Err(s.err_exp(&s.identbuf))
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

pub fn derive_de_ron_struct(struct_: &Struct) -> TokenStream {
    let body = derive_de_ron_named(&struct_.name, &struct_.fields, &struct_.attributes);

    format!(
        "impl DeRon for {} {{
            fn de_ron(s: &mut nanoserde::DeRonState, i: &mut std::str::Chars) -> std::result::Result<Self,nanoserde::DeRonErr> {{
                std::result::Result::Ok({})
            }}
        }}", struct_.name, body)
    .parse()
    .unwrap()
}

pub fn derive_de_ron_struct_unnamed(struct_: &Struct) -> TokenStream {
    let mut body = String::new();

    for _ in &struct_.fields {
        l!(
            body,
            "{{
                let r = DeRon::de_ron(s, i)?;
                s.eat_comma_paren(i)?;
                r
            }},"
        );
    }

    format! ("
        impl DeRon for {} {{
            fn de_ron(s: &mut nanoserde::DeRonState, i: &mut std::str::Chars) -> std::result::Result<Self,nanoserde::DeRonErr> {{
                s.paren_open(i)?;
                let r = Self({});
                s.paren_close(i)?;
                std::result::Result::Ok(r)
            }}
        }}",struct_.name, body
    ).parse().unwrap()
}

pub fn derive_ser_ron_enum(enum_: &Enum) -> TokenStream {
    let mut body = String::new();

    for variant in &enum_.variants {
        let ident = &variant.name;
        // Unit
        if variant.fields.len() == 0 {
            l!(body, "Self::{} => s.out.push_str(\"{}\"),", ident, ident)
        }
        // Named
        else if variant.named {
            let mut names = Vec::new();
            let mut inner = String::new();
            for field in &variant.fields {
                if let Some(name) = &field.field_name {
                    names.push(name.clone());
                    if field.ty.is_option {
                        l!(
                            inner,
                            "if {}.is_some() {{
                                s.field(d+1, \"{}\");
                                {}.ser_ron(d+1, s);
                                s.conl();
                            }}",
                            name,
                            name,
                            name
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
            )
        }
        // Unnamed
        else {
            let mut names = Vec::new();
            let mut inner = String::new();
            let last = variant.fields.len() - 1;
            for (index, _) in &mut variant.fields.iter().enumerate() {
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
    }
    format!(
        "
        impl SerRon for {} {{
            fn ser_ron(&self, d: usize, s: &mut nanoserde::SerRonState) {{
                match self {{
                    {}
                }}
            }}
        }}",
        enum_.name, body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_ron_enum(enum_: &Enum) -> TokenStream {
    let mut body = String::new();

    for variant in &enum_.variants {
        let ident = &variant.name;
        // Unit
        if variant.fields.len() == 0 {
            l!(body, "\"{}\" => Self::{},", ident, ident)
        }
        // Named
        else if variant.named {
            let name = format!("{}::{}", enum_.name, variant.name);
            let inner = derive_de_ron_named(&name, &variant.fields, &vec![]);
            l!(body, "\"{}\" => {}", ident, inner);
        }
        // Unnamed
        else {
            let mut inner = String::new();
            for _ in &variant.fields {
                l!(
                    inner,
                    "{
                        let r = DeRon::de_ron(s, i)?;
                        s.eat_comma_paren(i)?;
                        r
                    }, "
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
    }

    format! ("
        impl DeRon for {} {{
            fn de_ron(s: &mut nanoserde::DeRonState, i: &mut std::str::Chars) -> std::result::Result<Self,nanoserde::DeRonErr> {{
                // we are expecting an identifier
                s.ident(i)?;
                std::result::Result::Ok(match s.identbuf.as_ref() {{
                    {}
                    _ => return std::result::Result::Err(s.err_enum(&s.identbuf))
                }})
            }}
        }}", enum_.name, body).parse().unwrap()
}
