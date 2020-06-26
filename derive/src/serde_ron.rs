use crate::parse::Struct;

use proc_macro::TokenStream;

use crate::shared;

pub fn derive_de_ron_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl DeRon for {} {{
            fn de_ron(_s: &mut nanoserde::DeJsonState, i: &mut std::str::Chars) -> std::result::Result<Self, nanoserde::DeJsonErr> {{ {{
                let proxy: {} = DeRon::deserialize_ron(i)?;
                std::result::Result::Ok(Into::into(&proxy))
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

// pub fn derive_ser_ron_struct(input: &DeriveInput, fields: &FieldsNamed) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let bound = parse_quote!(SerRon);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);
//     let ident = &input.ident;

//     let mut outputs = Vec::new();
//     for field in &fields.named {
//         let fieldname = field.ident.clone().unwrap();
//         let fieldstring = LitStr::new(&fieldname.to_string(), ident.span());
//         if type_is_option(&field.ty) {
//             outputs.push(quote! {if let Some(t) = self.#fieldname {s.field(d+1,#fieldstring);t.ser_ron(d+1, s);s.conl();};})
//         }
//         else {
//             outputs.push(quote! {s.field(d+1,#fieldstring);self.#fieldname.ser_ron(d+1, s);s.conl();})
//         }
//     }

//     quote!{
//         impl #impl_generics SerRon for #ident #ty_generics #bounded_where_clause {
//             fn ser_ron(&self, d: usize, s: &mut makepad_tinyserde::SerRonState) {
//                 s.st_pre();
//                 #(
//                     #outputs
//                 ) *
//                 s.st_post(d);
//             }
//         }
//     }
// }

// pub fn type_is_option(ty: &Type) -> bool {
//     if let Type::Path(tp) = ty {
//         if tp.path.segments.len() == 1 && tp.path.segments[0].ident.to_string() == "Option" {
//             return true;
//         }
//     }
//     return false
// }

pub fn derive_de_ron_named(struct_: &Struct) -> TokenStream {
    let mut local_vars = Vec::new();
    let mut struct_field_names = Vec::new();
    let mut ron_field_names = Vec::new();

    let container_attr_default = shared::attrs_default(&struct_.attributes);

    let mut unwraps = Vec::new();
    for field in &struct_.fields {
        let struct_fieldname = field.field_name.as_ref().unwrap().to_string();
        let localvar = format!("_{}", struct_fieldname);
        let field_attr_default = shared::attrs_default(&field.attributes);
        let ron_fieldname =
            shared::attrs_rename(&field.attributes).unwrap_or(struct_fieldname.clone());

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
        ron_field_names.push(ron_fieldname);
        local_vars.push(localvar);
    }

    let mut r = String::new();
    for local_var in &local_vars {
        l!(r, "let mut {} = None;", local_var);
    }

    l!(r, "s.paren_open(i) ?;");
    l!(r, "while let Some(_) = s.next_ident() {");
    if ron_field_names.len() != 0 {
        l!(r, "match s.identbuf.as_ref() {");
        for (ron_field_name, local_var) in ron_field_names.iter().zip(local_vars.iter()) {
            l!(
                r,
                "\"{}\" => {{ s.next_colon(i) ?;{} = Some(DeRon::de_ron(s, i) ?) }},",
                ron_field_name,
                local_var
            );
        }
        l!(
            r,
            "_ => return std::result::Result::Err(s.err_exp(&s.identbuf))"
        );
        l!(r, "}");
    }
    l!(r, "s.eat_comma_paren(i) ?;");
    l!(r, "}");
    l!(r, "s.paren_close(i) ?;");
    l!(r, "{} {{", struct_.name);
    for (field_name, unwrap) in struct_field_names.iter().zip(unwraps.iter()) {
        l!(r, "{}: {},", field_name, unwrap);
    }
    l!(r, "}");

    r.parse().unwrap()
}

pub fn derive_de_ron_struct(struct_: &Struct) -> TokenStream {
    let body = derive_de_ron_named(struct_);

    format!(
        "impl DeRon for {} {{
            fn de_ron(s: &mut nanoserde::DeRonState, i: &mut std::str::Chars) -> std::result::Result<Self,
            nanoserde::DeRonErr> {{
                std::result::Result::Ok({{ {} }})
            }}
        }}", struct_.name, body)
    .parse()
    .unwrap()
}

// pub fn derive_ser_ron_enum(input: &DeriveInput, enumeration: &DataEnum) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let bound = parse_quote!(SerRon);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let ident = &input.ident;

//     let mut match_item = Vec::new();

//     for variant in &enumeration.variants {
//         let ident = &variant.ident;
//         let lit = LitStr::new(&ident.to_string(), ident.span());
//         match &variant.fields {
//             Fields::Unit => {
//                 match_item.push(quote!{
//                     Self::#ident => s.out.push_str(#lit),
//                 })
//             },
//             Fields::Named(fields_named) => {
//                 let mut items = Vec::new();
//                 let mut field_names = Vec::new();
//                 for field in &fields_named.named {
//                     if let Some(field_name) = &field.ident {
//                         let field_string = LitStr::new(&field_name.to_string(), field_name.span());
//                         if type_is_option(&field.ty) {
//                             items.push(quote!{
//                                 if #field_name.is_some(){
//                                     s.field(d+1, #field_string);
//                                     #field_name.ser_ron(d+1, s);
//                                     s.conl();
//                                 }
//                             })
//                         }
//                         else{
//                             items.push(quote!{
//                                 s.field(d+1, #field_string);
//                                 #field_name.ser_ron(d+1, s);
//                                 s.conl();
//                             })
//                         }
//                         field_names.push(field_name);
//                     }
//                 }
//                 match_item.push(quote!{
//                     Self::#ident {#(#field_names,) *} => {
//                         s.out.push_str(#lit);
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
//                             #field_name.ser_ron(d, s); s.out.push_str(", ");
//                         });
//                     }
//                     else{
//                         str_names.push(quote!{
//                             #field_name.ser_ron(d, s);
//                         });
//                     }
//                     field_names.push(field_name);
//                 }
//                 match_item.push(quote!{
//                     Self::#ident (#(#field_names,) *) => {
//                         s.out.push_str(#lit);
//                         s.out.push('(');
//                         #(#str_names) *
//                         s.out.push(')');
//                     }
//                 });
//             },
//         }
//     }

//     quote! {
//         impl #impl_generics SerRon for #ident #ty_generics #bounded_where_clause {
//             fn ser_ron(&self, d: usize, s: &mut makepad_tinyserde::SerRonState) {
//                 match self {
//                     #(
//                         #match_item
//                     ) *
//                 }
//             }
//         }
//     }
// }

// pub fn derive_de_ron_enum(input: &DeriveInput, enumeration: &DataEnum) -> TokenStream {

//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let ident = &input.ident;
//     let bound = parse_quote!(DeRon);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let mut match_item = Vec::new();

//     for variant in &enumeration.variants {
//         let ident = &variant.ident;
//         let lit = LitStr::new(&ident.to_string(), ident.span());
//         match &variant.fields {
//             Fields::Unit => {
//                 match_item.push(quote!{
//                     #lit => Self::#ident,
//                 })
//             },
//             Fields::Named(fields_named) => {
//                 let body = derive_de_ron_named(quote!{Self::#ident}, fields_named);
//                 match_item.push(quote!{
//                     #lit => {#body},
//                 });
//             },
//             Fields::Unnamed(fields_unnamed) => {
//                 let mut field_names = Vec::new();
//                 for _ in &fields_unnamed.unnamed {
//                     field_names.push(quote! {{let r = DeRon::de_ron(s,i)?;s.eat_comma_paren(i)?;r}});
//                 }
//                 match_item.push(quote!{
//                     #lit => {s.paren_open(i)?;let r = Self::#ident(#(#field_names,) *); s.paren_close(i)?;r},
//                 });
//             },
//         }
//     }

//     quote! {
//         impl #impl_generics DeRon for #ident #ty_generics #bounded_where_clause {
//             fn de_ron(s: &mut DeRonState, i: &mut std::str::Chars) -> std::result::Result<Self,DeRonErr> {
//                 // we are expecting an identifier
//                 s.ident(i)?;
//                 std::result::Result::Ok(match s.identbuf.as_ref() {
//                     #(
//                         #match_item
//                     ) *
//                     _ => return std::result::Result::Err(s.err_enum(&s.identbuf))
//                 })
//             }
//         }
//     }
// }

// pub fn derive_ser_ron_struct_unnamed(input: &DeriveInput, fields:&FieldsUnnamed) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let bound = parse_quote!(SerRon);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);
//     let ident = &input.ident;

//     let mut str_names = Vec::new();
//     let last = fields.unnamed.len() - 1;
//     for (index, field) in fields.unnamed.iter().enumerate() {
//         let field_name = LitInt::new(&format!("{}", index), field.span());
//         if index != last{
//             str_names.push(quote!{
//                 self.#field_name.ser_ron(d, s);
//                 s.out.push_str(", ");
//             })
//         }
//         else{
//             str_names.push(quote!{
//                 self.#field_name.ser_ron(d, s);
//             })
//         }
//     }
//     quote! {
//         impl #impl_generics SerRon for #ident #ty_generics #bounded_where_clause {
//             fn ser_ron(&self, d: usize, s: &mut makepad_tinyserde::SerRonState) {
//                 s.out.push('(');
//                 #(
//                     #str_names
//                 ) *
//                 s.out.push(')');
//             }
//         }
//     }
// }

// pub fn derive_de_ron_struct_unnamed(input: &DeriveInput, fields:&FieldsUnnamed) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let ident = &input.ident;
//     let bound = parse_quote!(DeRon);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let mut items = Vec::new();
//     for _ in &fields.unnamed {
//         items.push(quote!{{let r = DeRon::de_ron(s,i)?;s.eat_comma_paren(i)?;r},});
//     }

//     quote! {
//         impl #impl_generics DeRon for #ident #ty_generics #bounded_where_clause {
//             fn de_ron(s: &mut makepad_tinyserde::DeRonState, i: &mut std::str::Chars) -> std::result::Result<Self,DeRonErr> {
//                 s.paren_open(i)?;
//                 let r = Self(
//                     #(
//                         #items
//                     ) *
//                 );
//                 s.paren_close(i)?;
//                 std::result::Result::Ok(r)
//             }
//         }
//     }
// }
