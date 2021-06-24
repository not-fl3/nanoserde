use crate::parse::{Enum, Field, Struct};

use proc_macro::TokenStream;

use crate::shared;

pub fn derive_ser_json_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl SerJson for {} {{
            fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {{
                let proxy: {} = self.into();
                proxy.ser_json(d, s);
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_json_struct(struct_: &Struct) -> TokenStream {
    let mut s = String::new();

    l!(s, "let mut first_field_was_serialized = false;");

    if struct_.fields.len() >= 1 {
        for (_index, field) in struct_.fields.iter().enumerate() {
            let struct_fieldname = field.field_name.clone().unwrap();
            let json_fieldname =
                shared::attrs_rename(&field.attributes).unwrap_or_else(|| struct_fieldname.clone());
            let skip = shared::attrs_skip(&field.attributes);
            if skip {
                continue;
            }
            let proxied_field = if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
                format!(
                    "{{let proxy: {} = Into::into(&self.{});proxy}}",
                    proxy, struct_fieldname
                )
            } else {
                format!("self.{}", struct_fieldname)
            };

            if field.ty.is_option {
                l!(
                    s,
                    "if let Some(t) = &{} {{ if first_field_was_serialized {{ s.conl(); }};first_field_was_serialized = true;s.field(d+1, \"{}\");t.ser_json(d+1, s);}};",
                    proxied_field,
                    json_fieldname
                );
            } else {
                l!(
                    s,
                    "if first_field_was_serialized {{ s.conl(); }};first_field_was_serialized = true;s.field(d+1,\"{}\"); {}.ser_json(d+1, s);",
                    json_fieldname,
                    proxied_field
                );
            }
        }
    }

    format!(
        "
        impl SerJson for {} {{
            fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {{
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

pub fn derive_de_json_named(name: &str, defaults: bool, fields: &[Field]) -> TokenStream {
    let mut local_vars = Vec::new();
    let mut struct_field_names = Vec::new();
    let mut json_field_names = Vec::new();
    let mut matches = Vec::new();
    let mut unwraps = Vec::new();

    let container_attr_default = defaults;

    for field in fields {
        let struct_fieldname = field.field_name.as_ref().unwrap().to_string();
        let localvar = format!("_{}", struct_fieldname);
        let field_attr_default = shared::attrs_default(&field.attributes);
        let field_attr_default_with = shared::attrs_default_with(&field.attributes);
        let default_val = if let Some(v) = field_attr_default {
            Some(v.unwrap_or_else(|| String::from("Default::default()")))
        } else if let Some(mut v) = field_attr_default_with {
            v.push_str("()");
            Some(v)
        } else {
            None
        };
        let json_fieldname =
            shared::attrs_rename(&field.attributes).unwrap_or(struct_fieldname.clone());
        let proxy = crate::shared::attrs_proxy(&field.attributes);
        let skip = crate::shared::attrs_skip(&field.attributes);

        let proxified_t = if let Some(proxy) = proxy {
            format!("From::<&{}>::from(&t)", proxy)
        } else {
            format!("t")
        };

        if skip == false {
            if field.ty.is_option {
                unwraps.push(format!(
                    "{{if let Some(t) = {} {{ {} }} else {{ None }} }}",
                    localvar, proxified_t
                ));
            } else if container_attr_default || default_val.is_some() {
                unwraps.push(format!(
                    "{{if let Some(t) = {} {{ {} }} else {{ {} }} }}",
                    localvar,
                    proxified_t,
                    default_val.unwrap_or_else(|| String::from("Default::default()"))
                ));
            } else {
                unwraps.push(format!(
                    "{{if let Some(t) = {} {{ {} }} else {{return Err(s.err_nf(\"{}\"))}} }}",
                    localvar, proxified_t, struct_fieldname
                ));
            }
            matches.push((json_fieldname.clone(), localvar.clone()));
            local_vars.push(localvar);
        } else {
            unwraps.push(format!("None"));
        }

        struct_field_names.push(struct_fieldname);
        json_field_names.push(json_fieldname);
    }

    let mut r = String::new();
    for local_var in &local_vars {
        l!(r, "let mut {} = None;", local_var);
    }
    l!(r, "s.curly_open(i) ?;");
    l!(r, "while let Some(_) = s.next_str() {");

    if json_field_names.len() != 0 {
        l!(r, "match AsRef::<str>::as_ref(&s.strbuf) {");
        for (json_field_name, local_var) in matches.iter() {
            l!(
                r,
                "\"{}\" => {{s.next_colon(i) ?;{} = Some(DeJson::de_json(s, i) ?)}},",
                json_field_name,
                local_var
            );
        }
        // TODO: maybe introduce "exhaustive" attribute?
        // l!(
        //     r,
        //     "_ => return std::result::Result::Err(s.err_exp(&s.strbuf))"
        // );
        l!(r, "_ => {s.next_colon(i)?; s.whole_field(i)?; }");
        l!(r, "}");
    }
    l!(r, "s.eat_comma_curly(i) ?");
    l!(r, "}");
    l!(r, "s.curly_close(i) ?;");
    l!(r, "{} {{", name);
    for (field_name, unwrap) in struct_field_names.iter().zip(unwraps.iter()) {
        l!(r, "{}: {},", field_name, unwrap);
    }
    l!(r, "}");

    r.parse().unwrap()
}

pub fn derive_de_json_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl DeJson for {} {{
            fn de_json(_s: &mut nanoserde::DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self, nanoserde::DeJsonErr> {{
                let proxy: {} = DeJson::deserialize_json(i)?;
                std::result::Result::Ok(Into::into(&proxy))
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_de_json_struct(struct_: &Struct) -> TokenStream {
    let body = derive_de_json_named(
        &struct_.name,
        shared::attrs_default(&struct_.attributes).is_some()
            || shared::attrs_default_with(&struct_.attributes).is_some(),
        &struct_.fields[..],
    );

    format!(
        "impl DeJson for {} {{
            fn de_json(s: &mut nanoserde::DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self,
            nanoserde::DeJsonErr> {{
                std::result::Result::Ok({{ {} }})
            }}
        }}", struct_.name, body)
        .parse().unwrap()
}

pub fn derive_ser_json_enum(enum_: &Enum) -> TokenStream {
    let mut r = String::new();

    for variant in enum_.variants.iter() {
        let json_variant_name =
            shared::attrs_rename(&variant.attributes).unwrap_or(variant.name.clone());
        // Unit
        if variant.fields.len() == 0 {
            l!(
                r,
                "Self::{} => s.label(\"{}\"),",
                variant.name,
                json_variant_name
            );
        }
        // Named
        else if variant.named {
            let mut items = String::new();
            let mut field_names = vec![];
            let last = variant.fields.len() - 1;
            for (index, field) in variant.fields.iter().enumerate() {
                if let Some(name) = &&field.field_name {
                    let proxied_field =
                        if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
                            format!("{{let proxy: {} = Into::into(&{});proxy}}", proxy, name)
                        } else {
                            format!("{}", name)
                        };

                    if index == last {
                        if field.ty.is_option {
                            l!(
                                items,
                                "if {}.is_some(){{s.field(d+1, \"{}\");{}.ser_json(d+1, s);}}",
                                name,
                                name,
                                proxied_field
                            )
                        } else {
                            l!(
                                items,
                                "s.field(d+1, \"{}\");{}.ser_json(d+1, s);",
                                name,
                                proxied_field
                            )
                        }
                    } else {
                        if field.ty.is_option {
                            l!(
                                items,
                                "if {}.is_some(){{s.field(d+1, \"{}\");{}.ser_json(d+1, s);s.conl();}}",
                                name,
                                name,
                                proxied_field
                            );
                        } else {
                            l!(
                                items,
                                "s.field(d+1, \"{}\");{}.ser_json(d+1, s);s.conl();",
                                name,
                                proxied_field
                            );
                        }
                    }
                    field_names.push(name.clone());
                }
            }
            l!(
                r,
                "Self::{} {{ {} }} => {{
                        s.out.push('{{');
                        s.label(\"{}\");
                        s.out.push(':');
                        s.st_pre();
                        {}
                        s.st_post(d);
                        s.out.push('}}');
                    }}",
                variant.name,
                field_names.join(","),
                json_variant_name,
                items
            );
        }
        // Unnamed
        else {
            let mut names = Vec::new();
            let mut inner = String::new();
            let last = variant.fields.len() - 1;
            for (index, _) in &mut variant.fields.iter().enumerate() {
                let field_name = format!("f{}", index);
                names.push(field_name.clone());
                if index != last {
                    l!(inner, "{}.ser_json(d, s); s.out.push(',');", field_name);
                } else {
                    l!(inner, "{}.ser_json(d, s);", field_name);
                }
            }
            l!(
                r,
                "Self::{}  ({}) => {{
                        s.out.push('{{');
                        s.label(\"{}\");
                        s.out.push(':');
                        s.out.push('[');
                        {}
                        s.out.push(']');
                        s.out.push('}}');
                    }}",
                variant.name,
                names.join(","),
                json_variant_name,
                inner
            );
        }
    }

    format!(
        "
        impl SerJson for {} {{
            fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {{
                match self {{
                    {}
                }}
            }}
        }}",
        enum_.name, r
    )
    .parse()
    .unwrap()
}

pub fn derive_de_json_enum(enum_: &Enum) -> TokenStream {
    let mut r_units = String::new();
    let mut r_rest = String::new();

    for variant in &enum_.variants {
        let json_variant_name =
            shared::attrs_rename(&variant.attributes).unwrap_or(variant.name.clone());

        // Unit
        if variant.fields.len() == 0 {
            l!(
                r_units,
                "\"{}\" => Self::{},",
                json_variant_name,
                variant.name
            );
        }
        // Named
        else if variant.named {
            let body =
                derive_de_json_named(&format!("Self::{}", variant.name), false, &variant.fields);
            l!(r_rest, "\"{}\" => {{ {} }}, ", json_variant_name, body);
        }
        // Unnamed
        else if variant.named == false {
            let mut field_names = String::new();

            for _ in &variant.fields {
                l!(
                    field_names,
                    "{let r = DeJson::de_json(s,i)?;s.eat_comma_block(i)?;r},"
                );
            }
            l!(
                r_rest,
                "\"{}\" => {{s.block_open(i)?;let r = Self::{}({}); s.block_close(i)?;r}}",
                json_variant_name,
                variant.name,
                field_names
            );
        }
    }

    let mut r = format!(
        "impl DeJson for {} {{
            fn de_json(s: &mut nanoserde::DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self, nanoserde::DeJsonErr> {{
                match s.tok {{",
        enum_.name,
    );

    if !r_rest.is_empty() {
        r.push_str(&format!(
            "
                    nanoserde::DeJsonTok::CurlyOpen => {{
                        s.curly_open(i)?;
                        let _ = s.string(i)?;
                        s.colon(i)?;
                        let r = std::result::Result::Ok(match s.strbuf.as_ref() {{
                            {}
                            _ => return std::result::Result::Err(s.err_enum(&s.strbuf))
                        }});
                        s.curly_close(i)?;
                        r
                    }},",
            r_rest,
        ))
    }

    if !r_units.is_empty() {
        r.push_str(&format!(
            "
                    nanoserde::DeJsonTok::Str => {{
                        let _ = s.string(i)?;
                        std::result::Result::Ok(match s.strbuf.as_ref() {{
                            {}
                            _ => return std::result::Result::Err(s.err_enum(&s.strbuf))
                        }})
                    }},",
            r_units,
        ))
    }

    r.push_str(
        r#"
                    _ => std::result::Result::Err(s.err_token("String or {")),
                }
            }
        }
"#,
    );

    r.parse().unwrap()
}

pub fn derive_ser_json_struct_unnamed(struct_: &Struct) -> TokenStream {
    let mut body = String::new();

    let transparent = shared::attrs_transparent(&struct_.attributes);

    // encode empty struct as {}
    if struct_.fields.len() == 0 {
        l!(body, "s.out.push('}');");
        l!(body, "s.out.push('{');");
    }
    // if its a newtype struct and it should be transparent - skip any curles
    // and skip "container"
    else if transparent && struct_.fields.len() == 1 {
        l!(body, "self.{}.ser_json(d, s);", 0);
    }
    // if more than one field - encode as array []
    else {
        l!(body, "s.out.push('[');");
        let last = struct_.fields.len() - 1;
        for (n, _) in struct_.fields.iter().enumerate() {
            l!(body, "self.{}.ser_json(d, s);", n);
            if n != last {
                l!(body, "s.out.push_str(\", \");");
            }
        }
        l!(body, "s.out.push(']');");
    }

    format!(
        "
        impl SerJson for {} {{
            fn ser_json(&self, d: usize, s: &mut nanoserde::SerJsonState) {{
                {}
            }}
        }}",
        struct_.name, body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_json_struct_unnamed(struct_: &Struct) -> TokenStream {
    let mut body = String::new();

    let transparent = shared::attrs_transparent(&struct_.attributes);

    for _ in &struct_.fields {
        l!(body, "{ let r = DeJson::de_json(s, i)?;");
        if struct_.fields.len() != 1 {
            l!(body, "  s.eat_comma_block(i)?;");
        }
        l!(body, "  r");
        l!(body, "},");
    }

    // no fields - was encoded as {}
    let body = if struct_.fields.len() == 0 {
        format!("s.curly_open(i)?;let r = Self;s.curly_close(i)?;")
    }
    // if it was transparent newtype struct - skip "container"
    // and just deserialize content
    else if transparent && struct_.fields.len() == 1 {
        format!("let r = Self({});", body)
    }
    // more than one field, was an array []
    else {
        format!(
            "s.block_open(i)?;
             let r = Self({});
             s.block_close(i)?;",
            body
        )
    };

    format! ("
        impl DeJson for {} {{
            fn de_json(s: &mut nanoserde::DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self,nanoserde::DeJsonErr> {{
                {}
                std::result::Result::Ok(r)
            }}
        }}",struct_.name, body
    ).parse().unwrap()
}
