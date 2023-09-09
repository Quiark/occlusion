use std::collections::HashMap;
use syn::{Data, DeriveInput, Field, Fields, Ident, Attribute};
use syn::parse::Result;
use quote::quote;
use proc_macro::TokenStream;
use proc_macro2::Span;


// occ as in occlude (we are doing projection)

/// parses one line attribute
fn parse_occlude(attr: &Attribute) -> Result<Vec<Ident>> {
    let mut res = Vec::new();
    attr.parse_nested_meta(|meta| 
        if let Some(id) = meta.path.get_ident() {
            res.push(id.clone());
            Ok(())
        } else {
            Err(meta.error("unsupported attribute"))
        })?;
    Ok(res)
}

fn maybe_parse_for_field(f: &Field) -> Result<Vec<Ident>> {
    let r = f.attrs.iter()
        .filter(|at| at.path().is_ident("occlude"))
        .map(|it| parse_occlude(it).unwrap())
        .flatten()
        .collect();

    Ok(r)
}

/// name: original struct name
fn gen_one_projection(name: &Ident, variant: &Ident, all: &Fields, occluded: &Vec<String>) -> proc_macro2::TokenStream {
    let proj_name = Ident::new(&format!("{}{}", name, variant), Span::call_site());
    let sel_fields: Vec<&Field> = all.iter()
        .filter(|f| !occluded.contains(&f.ident.as_ref().unwrap().to_string()))
        .collect();

    let field_ids: Vec<&Ident> = sel_fields.iter().map(|field| field.ident.as_ref().unwrap()).collect();
    let field_types: Vec<&syn::Type> = sel_fields.iter().map(|field| &field.ty).collect();

    let generated = quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema, sqlxinsert::SqliteUpdate)]
        pub struct #proj_name {
            #(pub #field_ids: #field_types),*
        }

        impl core::convert::From<#name> for #proj_name {
            fn from(item: #name) -> #proj_name {
                #proj_name {
                    #(#field_ids: item.#field_ids),*
                }
            }
        }

        impl core::convert::From<#proj_name> for #name {
            fn from(item: #proj_name) -> #name {
                #name {
                    #(#field_ids: item.#field_ids),*
                    ,
                    ..Default::default()
                }
            }
        }
    };

    generated
}

#[proc_macro_derive(Projections, attributes(occlude))]
pub fn derive_projection(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let struct_fields = match &ast.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("Projections can only be derived for structs"),
    };


    let occlusions: Vec<(&Field, Vec<Ident>)> = struct_fields
        .iter()
        .map(|f| (f, maybe_parse_for_field(f).unwrap()))
        .collect();

    let grouped: HashMap<Ident, Vec<String>> = occlusions
        .iter()
        .flat_map(|(f, ids)| ids.iter().map(|id| (id, f.ident.as_ref().unwrap().to_string())))
        .fold(HashMap::new(), |mut acc, (id, f)| {
            acc.entry(id.clone()).or_insert(Vec::new()).push(f);
            acc
        });

    grouped.iter()
        .map(|(id, fields)| gen_one_projection(name, id, struct_fields, fields))
        .reduce(|a, b| quote! { #a #b })
        .unwrap_or(quote! {})
        .into()

}

