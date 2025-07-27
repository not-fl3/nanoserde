use alloc::format;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};

use crate::parse::{Category, Type};
use crate::shared::{enum_bounds_strings, struct_bounds_strings};
use crate::{
    parse::{Enum, Field, Struct},
    shared,
};

use proc_macro::TokenStream;

pub fn derive_ser_json_proxy(proxy_type: &str, type_: &str, crate_name: &str) -> TokenStream {
    format!(
        "impl {}::SerJson for {} {{
            fn ser_json(&self, d: usize, s: &mut {}::SerJsonState) {{
                let proxy: {} = self.into();
                proxy.ser_json(d, s);
            }}
        }}",
        crate_name, type_, crate_name, proxy_type
    )
    .parse()
    .unwrap()
}

fn ser_proxy_guard(fieldname: &str, field: &Field) -> String {
    if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
        if field.ty.base() == "Option" {
            format!(
                "{{{fieldname}.as_ref().map(|f| {{let proxy: {proxy} = Into::into(f);proxy}})}}"
            )
        } else {
            format!("{{let proxy: {proxy} = Into::into(&{fieldname});proxy}}",)
        }
    } else {
        fieldname.to_string()
    }
}

pub fn derive_ser_json_struct(struct_: &Struct, crate_name: &str) -> TokenStream {
    let mut s = String::new();
    let (generic_w_bounds, generic_no_bounds) =
        struct_bounds_strings(struct_, "SerJson", crate_name);

    l!(s, "let mut first_field_was_serialized = false;");

    if !struct_.fields.is_empty() {
        for field in struct_.fields.iter() {
            let struct_fieldname = field.field_name.clone().unwrap();
            let json_fieldname =
                shared::attrs_rename(&field.attributes).unwrap_or_else(|| struct_fieldname.clone());
            let skip = shared::attrs_skip(&field.attributes);
            if skip {
                continue;
            }
            let proxied_field = ser_proxy_guard(&format!("self.{struct_fieldname}"), field);

            if field.ty.base() == "Option" {
                let proxy_attr = crate::shared::attrs_proxy(&field.attributes);
                let struct_null_on_none = shared::attrs_serialize_none_as_null(&struct_.attributes);
                let field_null_on_none = shared::attrs_serialize_none_as_null(&field.attributes);
                let null_on_none =
                    (field_null_on_none || struct_null_on_none) && proxy_attr.is_none();
                let field_header = &format!(
                    "if first_field_was_serialized {{
                                                 s.conl();
                                             }};
                                             first_field_was_serialized = true;
                                             s.field(d+1, \"{}\");",
                    json_fieldname
                );
                l!(
                    s,
                    "{}
                    if let Some(t) = &{} {{
                        {}
                        t.ser_json(d+1, s);
                    }} {}",
                    if null_on_none { field_header } else { "" },
                    proxied_field,
                    if null_on_none { "" } else { field_header },
                    if null_on_none {
                        "else {{
                            Option::<i32>::ser_json(&None, d+1, s);
                        }}"
                    } else {
                        ""
                    }
                );
            } else {
                l!(
                    s,
                    "if first_field_was_serialized {{
                        s.conl();
                    }};
                    first_field_was_serialized = true;
                    s.field(d+1,\"{}\");
                    {}.ser_json(d+1, s);",
                    json_fieldname,
                    proxied_field
                );
            }
        }
    }

    format!(
        "
        impl{} {}::SerJson for {}{} {{
            fn ser_json(&self, d: usize, s: &mut {}::SerJsonState) {{
                s.st_pre();
                {}
                s.st_post(d);
            }}
        }}
    ",
        generic_w_bounds,
        crate_name,
        struct_
            .name
            .as_ref()
            .expect("Cannot implement for anonymous struct"),
        generic_no_bounds,
        crate_name,
        s
    )
    .parse()
    .unwrap()
}

pub fn derive_de_json_named(
    name: &str,
    defaults: bool,
    fields: &[Field],
    crate_name: &str,
) -> TokenStream {
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
            if let Some(mut val) = v {
                if field.ty.base() == "String"
                    || field
                        .ty
                        .wraps
                        .as_ref()
                        .is_some_and(|wrapped| wrapped.iter().any(|ty| ty.base() == "String"))
                {
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
        let json_fieldname =
            shared::attrs_rename(&field.attributes).unwrap_or(struct_fieldname.clone());
        let proxy = crate::shared::attrs_proxy(&field.attributes);
        let skip = crate::shared::attrs_skip(&field.attributes);

        let proxified_t = if let Some(proxy) = proxy {
            if field.ty.base() == "Option" {
                format!("Some(From::<&{proxy}>::from(&t))")
            } else {
                format!("From::<&{proxy}>::from(&t)")
            }
        } else {
            "t".to_string()
        };

        if !skip {
            if field.ty.base() == "Option" || container_attr_default || default_val.is_some() {
                if let Some(default_val) = default_val {
                    unwraps.push(format!(
                        "{}.map_or_else(|| {}, |t| {})",
                        localvar, default_val, proxified_t,
                    ));
                } else {
                    unwraps.push(format!(
                        "{}.map(|t| {}).unwrap_or_default()",
                        localvar, proxified_t,
                    ));
                }
            } else {
                unwraps.push(format!(
                    "{}.map(|t| {}).ok_or_else(|| s.err_nf(\"{}\"))?",
                    localvar, proxified_t, struct_fieldname
                ));
            }
            matches.push((json_fieldname.clone(), localvar.clone()));
            local_vars.push(localvar);
        } else {
            unwraps.push(default_val.unwrap_or_else(|| String::from("Default::default()")));
        }

        struct_field_names.push(struct_fieldname);
        json_field_names.push(json_fieldname);
    }

    let mut r = String::new();
    for local_var in &local_vars {
        l!(r, "let mut {} = None;", local_var);
    }
    l!(r, "s.curly_open(i) ?;");
    l!(r, "while s.next_str().is_some() {");

    if !json_field_names.is_empty() {
        l!(r, "match AsRef::<str>::as_ref(&s.strbuf) {");
        for (json_field_name, local_var) in matches.iter() {
            l!(
                r,
                "\"{}\" => {{s.next_colon(i) ?;{} = Some({}::DeJson::de_json(s, i) ?)}},",
                json_field_name,
                local_var,
                crate_name
            );
        }
        // TODO: maybe introduce "exhaustive" attribute?
        // l!(
        //     r,
        //     "_ => return ::core::result::Result::Err(s.err_exp(&s.strbuf))"
        // );
        l!(r, "_ => {s.next_colon(i)?; s.whole_field(i)?; }");
        l!(r, "}");
    }
    l!(r, "s.eat_comma_curly(i) ?;");
    l!(r, "}");
    l!(r, "s.curly_close(i) ?;");
    l!(r, "{} {{", name);
    for (field_name, unwrap) in struct_field_names.iter().zip(unwraps.iter()) {
        l!(r, "{}: {},", field_name, unwrap);
    }
    l!(r, "}");

    r.parse().unwrap()
}

pub fn derive_de_json_proxy(proxy_type: &str, type_: &str, crate_name: &str) -> TokenStream {
    format!(
        "impl {}::DeJson for {} {{
            #[allow(clippy::ignored_unit_patterns)]
            fn de_json(s: &mut {}::DeJsonState, i: &mut core::str::Chars) -> ::core::result::Result<Self, {}::DeJsonErr> {{
                let proxy: {} = {}::DeJson::de_json(s, i)?;
                ::core::result::Result::Ok(Into::into(&proxy))
            }}
        }}",
        crate_name, type_, crate_name, crate_name, proxy_type, crate_name
    )
    .parse()
    .unwrap()
}

pub fn derive_de_json_struct(struct_: &Struct, crate_name: &str) -> TokenStream {
    let body = derive_de_json_named(
        struct_
            .name
            .as_ref()
            .expect("Cannot implement for anonymous struct"),
        shared::attrs_default(&struct_.attributes).is_some()
            || shared::attrs_default_with(&struct_.attributes).is_some(),
        &struct_.fields[..],
        crate_name,
    );
    let (generic_w_bounds, generic_no_bounds) =
        struct_bounds_strings(struct_, "DeJson", crate_name);

    format!(
        "impl{} {}::DeJson for {}{} {{
            #[allow(clippy::ignored_unit_patterns)]
            fn de_json(s: &mut {}::DeJsonState, i: &mut core::str::Chars) -> ::core::result::Result<Self,
            {}::DeJsonErr> {{
                ::core::result::Result::Ok({{ {} }})
            }}
        }}", generic_w_bounds, crate_name, struct_.name.as_ref().expect("Cannot implement for anonymous struct"), generic_no_bounds, crate_name, crate_name, body)
        .parse().unwrap()
}

pub fn derive_ser_json_enum(enum_: &Enum, crate_name: &str) -> TokenStream {
    let mut r = String::new();
    let (generic_w_bounds, generic_no_bounds) = enum_bounds_strings(enum_, "SerJson", crate_name);

    for variant in enum_.variants.iter() {
        let field_name = variant.field_name.clone().unwrap();
        let json_variant_name =
            shared::attrs_rename(&variant.attributes).unwrap_or(field_name.clone());

        match &variant.ty {
            Type {
                wraps: None,
                ident: Category::None,
                ..
            } => {
                // unit variant
                l!(
                    r,
                    "Self::{} => s.label(\"{}\"),",
                    &field_name,
                    json_variant_name
                );
            }

            Type {
                ident: Category::AnonymousStruct { contents },
                ..
            } => {
                let mut items = String::new();
                let mut field_names = vec![];
                let last = contents.fields.len().saturating_sub(1);
                for (index, field) in contents.fields.iter().enumerate() {
                    if let Some(name) = &&field.field_name {
                        let proxied_field = ser_proxy_guard(name, field);
                        if index == last {
                            if field.ty.base() == "Option" {
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
                        } else if field.ty.base() == "Option" {
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
                    &field_name,
                    field_names.join(","),
                    json_variant_name,
                    items
                );
            }
            Type {
                ident: Category::Tuple { contents },
                ..
            } => {
                let mut names = Vec::new();
                let mut inner = String::new();
                let last = contents.len() - 1;
                for (index, _) in contents.iter().enumerate() {
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
                    &field_name,
                    names.join(","),
                    json_variant_name,
                    inner
                );
            }
            v => {
                unimplemented!("Unexpected type in enum: {:?}", v)
            }
        };
    }

    format!(
        "
        impl{} {}::SerJson for {}{} {{
            fn ser_json(&self, d: usize, s: &mut {}::SerJsonState) {{
                match self {{
                    {}
                }}
            }}
        }}",
        generic_w_bounds, crate_name, enum_.name, generic_no_bounds, crate_name, r
    )
    .parse()
    .unwrap()
}

pub fn derive_de_json_enum(enum_: &Enum, crate_name: &str) -> TokenStream {
    let mut r_units = String::new();
    let mut r_rest = String::new();
    let (generic_w_bounds, generic_no_bounds) = enum_bounds_strings(enum_, "DeJson", crate_name);

    for variant in &enum_.variants {
        let field_name = variant.field_name.clone().unwrap();
        let json_variant_name =
            shared::attrs_rename(&variant.attributes).unwrap_or(field_name.clone());

        match &variant.ty {
            Type {
                wraps: None,
                ident: Category::None,
                ..
            } => {
                // unit variant
                l!(
                    r_units,
                    "\"{}\" => Self::{},",
                    json_variant_name,
                    &field_name
                );
            }
            Type {
                ident: Category::AnonymousStruct { contents },
                ..
            } => {
                let body = derive_de_json_named(
                    &format!("Self::{}", &field_name),
                    false,
                    &contents.fields,
                    crate_name,
                );
                l!(r_rest, "\"{}\" => {{ {} }}, ", json_variant_name, body);
            }
            Type {
                ident: Category::Tuple { contents },
                ..
            } => {
                let mut field_names = String::new();
                for _ in contents.iter() {
                    l!(
                        field_names,
                        "{{let r = {}::DeJson::de_json(s,i)?;s.eat_comma_block(i)?;r}},",
                        crate_name
                    );
                }
                l!(
                    r_rest,
                    "\"{}\" => {{s.block_open(i)?;let r = Self::{}({}); s.block_close(i)?;r}}",
                    json_variant_name,
                    &field_name,
                    field_names
                );
            }
            v => {
                unimplemented!("Unexpected type in enum: {:?}", v)
            }
        };
    }

    let mut r = format!(
        "impl{} {}::DeJson for {}{} {{
            #[allow(clippy::ignored_unit_patterns)]
            fn de_json(s: &mut {}::DeJsonState, i: &mut core::str::Chars) -> ::core::result::Result<Self, {}::DeJsonErr> {{
                match s.tok {{",
        generic_w_bounds, crate_name, enum_.name, generic_no_bounds, crate_name, crate_name
    );

    if !r_rest.is_empty() {
        r.push_str(&format!(
            "
                    {}::DeJsonTok::CurlyOpen => {{
                        s.curly_open(i)?;
                        let _ = s.string(i)?;
                        s.colon(i)?;
                        let r = ::core::result::Result::Ok(match s.strbuf.as_ref() {{
                            {}
                            _ => return ::core::result::Result::Err(s.err_enum(&s.strbuf))
                        }});
                        s.curly_close(i)?;
                        r
                    }},",
            crate_name, r_rest,
        ))
    }

    if !r_units.is_empty() {
        r.push_str(&format!(
            "
                    {}::DeJsonTok::Str => {{
                        let _ = s.string(i)?;
                        ::core::result::Result::Ok(match s.strbuf.as_ref() {{
                            {}
                            _ => return ::core::result::Result::Err(s.err_enum(&s.strbuf))
                        }})
                    }},",
            crate_name, r_units,
        ))
    }

    r.push_str(
        r#"
                    _ => ::core::result::Result::Err(s.err_token("String or {")),
                }
            }
        }
"#,
    );

    r.parse().unwrap()
}

pub fn derive_ser_json_struct_unnamed(struct_: &Struct, crate_name: &str) -> TokenStream {
    let mut body = String::new();
    let (generic_w_bounds, generic_no_bounds) =
        struct_bounds_strings(struct_, "SerJson", crate_name);

    let transparent = shared::attrs_transparent(&struct_.attributes);

    // encode empty struct as {}
    if struct_.fields.is_empty() {
        l!(body, "s.out.push('{');");
        l!(body, "s.out.push('}');");
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
        impl{} {}::SerJson for {}{} {{
            fn ser_json(&self, d: usize, s: &mut {}::SerJsonState) {{
                {}
            }}
        }}",
        generic_w_bounds,
        crate_name,
        struct_
            .name
            .as_ref()
            .expect("Cannot implement for anonymous struct"),
        generic_no_bounds,
        crate_name,
        body
    )
    .parse()
    .unwrap()
}

pub fn derive_de_json_struct_unnamed(struct_: &Struct, crate_name: &str) -> TokenStream {
    let mut body = String::new();
    let (generic_w_bounds, generic_no_bounds) =
        struct_bounds_strings(struct_, "DeJson", crate_name);

    let transparent = shared::attrs_transparent(&struct_.attributes);

    for _ in &struct_.fields {
        l!(body, "{{ let r = {}::DeJson::de_json(s, i)?;", crate_name);
        if struct_.fields.len() != 1 {
            l!(body, "  s.eat_comma_block(i)?;");
        }
        l!(body, "  r");
        l!(body, "},");
    }

    // no fields - was encoded as {}
    let body = if struct_.fields.is_empty() {
        "s.curly_open(i)?;let r = Self;s.curly_close(i)?;".to_string()
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
        impl{} {}::DeJson for {}{} {{
            #[allow(clippy::ignored_unit_patterns)]
            fn de_json(s: &mut {}::DeJsonState, i: &mut core::str::Chars) -> ::core::result::Result<Self,{}::DeJsonErr> {{
                {}
                ::core::result::Result::Ok(r)
            }}
        }}",generic_w_bounds, crate_name, struct_.name.as_ref().expect("Cannot implement for anonymous struct"), generic_no_bounds, crate_name, crate_name,body
    ).parse().unwrap()
}
