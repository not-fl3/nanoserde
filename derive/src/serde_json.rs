use crate::parse::Struct;

use proc_macro::TokenStream;

use crate::shared;
// use proc_macro2::{TokenStream};
// use syn::{
//     parse_quote,
//     Ident,
//     DeriveInput,
//     Fields,
//     FieldsNamed,
//     FieldsUnnamed,
//     DataEnum,
//     LitInt,
//     LitStr,
//     Type,
// };
// use quote::quote;
// use quote::format_ident;
// use syn::spanned::Spanned;

pub fn derive_ser_json_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl SerJson for {} {{
            fn ser_json(&self, d: usize, s: &mut makepad_tinyserde::SerJsonState) {{
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
    // let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    // let bound = parse_quote!(SerJson);
    // let bounded_where_clause = where_clause_with_bound(&input.generics, bound);
    // let ident = &input.ident;

    let mut s = String::new();

    let last = struct_.fields.len() - 1;
    for (index, field) in struct_.fields.iter().enumerate() {
        let struct_fieldname = field.field_name.clone().unwrap();
        let json_fieldname =
            shared::attrs_replace(&field.attributes).unwrap_or_else(|| struct_fieldname.clone());

        if index == last {
            if field.ty.is_option {
                l!(
                    s,
                    "if let Some(t) = &self.{} {{ s.field(d+1, \"{}\");t.ser_json(d+1, s);}};",
                    struct_fieldname,
                    json_fieldname
                );
            } else {
                l!(
                    s,
                    "s.field(d+1,\"{}\"); self.{}.ser_json(d+1, s);",
                    json_fieldname,
                    struct_fieldname
                );
            }
        } else {
            if field.ty.is_option {
                l!(s, "if let Some(t) = &self.{} {{ s.field(d+1, \"{}\");t.ser_json(d+1, s);s.conl();}};", struct_fieldname, json_fieldname);
            } else {
                l!(
                    s,
                    "s.field(d+1,\"{}\"); self.{}.ser_json(d+1, s);s.conl();",
                    json_fieldname,
                    struct_fieldname
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

pub fn derive_de_json_named(struct_: &Struct) -> TokenStream {
    let mut local_vars = Vec::new();
    let mut struct_field_names = Vec::new();
    let mut json_field_names = Vec::new();
    let mut unwraps = Vec::new();

    let container_attr_default = shared::attrs_default(&struct_.attributes);

    for field in &struct_.fields {
        let struct_fieldname = field.field_name.as_ref().unwrap().to_string();
        let localvar = format!("_{}", struct_fieldname);
        let field_attr_default = shared::attrs_default(&field.attributes);
        let json_fieldname =
            shared::attrs_replace(&field.attributes).unwrap_or(struct_fieldname.clone());

        if field.ty.is_option {
            unwraps.push(format!(
                "{{if let Some(t) = {} {{t}}else {{ None }} }}",
                localvar
            ));
        } else if container_attr_default || field_attr_default {
            unwraps.push(format!(
                "{{if let Some(t) = {} {{t}}else {{ Default::default() }} }}",
                localvar
            ));
        } else {
            unwraps.push(format!(
                "{{if let Some(t) = {} {{t}} else {{return Err(s.err_nf(\"{}\"))}} }}",
                localvar, struct_fieldname
            ));
        }

        struct_field_names.push(struct_fieldname);
        json_field_names.push(json_fieldname);
        local_vars.push(localvar);
    }

    let mut r = String::new();
    for local_var in &local_vars {
        l!(r, "let mut {} = None;", local_var);
    }
    l!(r, "s.curly_open(i) ?;");
    l!(r, "while let Some(_) = s.next_str() {");

    if json_field_names.len() != 0 {
        l!(r, "match s.strbuf.as_ref() {");
        for (json_field_name, local_var) in json_field_names.iter().zip(local_vars.iter()) {
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
    l!(r, "{} {{", struct_.name);
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
    let body = derive_de_json_named(struct_);

    format!(
        "impl DeJson for {} {{
            fn de_json(s: &mut nanoserde::DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self,
            nanoserde::DeJsonErr> {{
                std::result::Result::Ok({{ {} }})
            }}
        }}", struct_.name, body)
    .parse()
    .unwrap()
}

// pub fn derive_ser_json_enum(input: &DeriveInput, enumeration: &DataEnum) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let bound = parse_quote!(SerJson);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let ident = &input.ident;

//     let mut match_item = Vec::new();

//     for variant in &enumeration.variants {
//         let ident = &variant.ident;
//         let lit = LitStr::new(&ident.to_string(), ident.span());
//         match &variant.fields {
//             Fields::Unit => {
//                 match_item.push(quote!{
//                     Self::#ident => {s.label(#lit);s.out.push_str(":[]");},
//                 })
//             },
//             Fields::Named(fields_named) => {
//                 let mut items = Vec::new();
//                 let mut field_names = Vec::new();
//                 let last = fields_named.named.len() - 1;
//                 for (index, field) in fields_named.named.iter().enumerate() {
//                     if let Some(field_name) = &field.ident {
//                         let field_string = LitStr::new(&field_name.to_string(), field_name.span());
//                         if index == last{
//                             if type_is_option(&field.ty) {
//                                 items.push(quote!{if #field_name.is_some(){s.field(d+1, #field_string);#field_name.ser_json(d+1, s);}})
//                             }
//                             else{
//                                 items.push(quote!{s.field(d+1, #field_string);#field_name.ser_json(d+1, s);})
//                             }
//                         }
//                         else{
//                             if type_is_option(&field.ty) {
//                                 items.push(quote!{if #field_name.is_some(){s.field(d+1, #field_string);#field_name.ser_json(d+1, s);s.conl();}})
//                             }
//                             else{
//                                 items.push(quote!{s.field(d+1, #field_string);#field_name.ser_json(d+1, s);s.conl();})
//                             }
//                         }
//                         field_names.push(field_name);
//                     }
//                 }
//                 match_item.push(quote!{
//                     Self::#ident {#(#field_names,) *} => {
//                         s.label(#lit);
//                         s.out.push(':');
//                         s.st_pre();
//                         #(
//                             #items
//                         )*
//                         s.st_post(d);
//                     }
//                 });
//             },
//             Fields::Unnamed(fields_unnamed) => {
//                 let mut field_names = Vec::new();
//                 let mut str_names = Vec::new();
//                 let last = fields_unnamed.unnamed.len() - 1;
//                 for (index, field) in fields_unnamed.unnamed.iter().enumerate() {
//                     let field_name = Ident::new(&format!("f{}", index), field.span());
//                     if index != last{
//                         str_names.push(quote!{
//                             #field_name.ser_json(d, s); s.out.push(',');
//                         });
//                     }
//                     else{
//                         str_names.push(quote!{
//                             #field_name.ser_json(d, s);
//                         });
//                     }
//                     field_names.push(field_name);
//                 }
//                 match_item.push(quote!{
//                     Self::#ident (#(#field_names,) *) => {
//                         s.label(#lit);
//                         s.out.push(':');
//                         s.out.push('[');
//                         #(#str_names) *
//                         s.out.push(']');
//                     }
//                 });
//             },
//         }
//     }

//     quote! {
//         impl #impl_generics SerJson for #ident #ty_generics #bounded_where_clause {
//             fn ser_json(&self, d: usize, s: &mut makepad_tinyserde::SerJsonState) {
//                 s.out.push('{');
//                 match self {
//                     #(
//                         #match_item
//                     ) *
//                 }
//                 s.out.push('}');
//             }
//         }
//     }
// }

// pub fn derive_de_json_enum(input: &DeriveInput, enumeration: &DataEnum) -> TokenStream {

//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let ident = &input.ident;
//     let bound = parse_quote!(DeJson);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let mut match_item = Vec::new();

//     for variant in &enumeration.variants {
//         let ident = &variant.ident;
//         let lit = LitStr::new(&ident.to_string(), ident.span());
//         match &variant.fields {
//             Fields::Unit => {
//                 match_item.push(quote!{
//                     #lit => {s.block_open(i)?;s.block_close(i)?;Self::#ident},
//                 })
//             },
//             Fields::Named(fields_named) => {
//                 let body = derive_de_json_named(quote!{Self::#ident}, fields_named);
//                 match_item.push(quote!{
//                     #lit => {#body},
//                 });
//             },
//             Fields::Unnamed(fields_unnamed) => {
//                 let mut field_names = Vec::new();
//                 for _ in &fields_unnamed.unnamed {
//                     field_names.push(quote! {{let r = DeJson::de_json(s,i)?;s.eat_comma_block(i)?;r}});
//                 }
//                 match_item.push(quote!{
//                     #lit => {s.block_open(i)?;let r = Self::#ident(#(#field_names,) *); s.block_close(i)?;r},
//                 });
//             },
//         }
//     }

//     quote! {
//         impl #impl_generics DeJson for #ident #ty_generics #bounded_where_clause {
//             fn de_json(s: &mut DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self,DeJsonErr> {
//                 // we are expecting an identifier
//                 s.curly_open(i)?;
//                 let _ = s.string(i)?;
//                 s.colon(i)?;
//                 let r = std::result::Result::Ok(match s.strbuf.as_ref() {
//                     #(
//                         #match_item
//                     ) *
//                     _ => return std::result::Result::Err(s.err_enum(&s.strbuf))
//                 });
//                 s.curly_close(i)?;
//                 r
//             }
//         }
//     }
// }

// pub fn derive_ser_json_struct_unnamed(input: &DeriveInput, fields:&FieldsUnnamed) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let bound = parse_quote!(SerJson);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);
//     let ident = &input.ident;

//     let mut str_names = Vec::new();
//     let last = fields.unnamed.len() - 1;
//     for (index, field) in fields.unnamed.iter().enumerate() {
//         let field_name = LitInt::new(&format!("{}", index), field.span());
//         if index != last{
//             str_names.push(quote!{
//                 self.#field_name.ser_json(d, s);
//                 s.out.push(',');
//             })
//         }
//         else{
//             str_names.push(quote!{
//                 self.#field_name.ser_json(d, s);
//             })
//         }
//     }
//     quote! {
//         impl #impl_generics SerJson for #ident #ty_generics #bounded_where_clause {
//             fn ser_json(&self, d: usize, s: &mut makepad_tinyserde::SerJsonState) {
//                 s.out.push('[');
//                 #(
//                     #str_names
//                 ) *
//                 s.out.push(']');
//             }
//         }
//     }
// }

// pub fn derive_de_json_struct_unnamed(input: &DeriveInput, fields:&FieldsUnnamed) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let ident = &input.ident;
//     let bound = parse_quote!(DeJson);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let mut items = Vec::new();
//     for _ in &fields.unnamed {
//         items.push(quote!{{let r = DeJson::de_json(s,i)?;s.eat_comma_block(i)?;r}});
//     }

//     quote! {
//         impl #impl_generics DeJson for #ident #ty_generics #bounded_where_clause {
//             fn de_json(s: &mut makepad_tinyserde::DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self,DeJsonErr> {
//                 s.block_open(i)?;
//                 let r = Self(
//                     #(
//                         #items
//                     ) *
//                 );
//                 s.block_close(i)?;
//                 std::result::Result::Ok(r)
//             }
//         }
//     }
// }
