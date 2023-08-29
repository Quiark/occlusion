use syn::{Data, DeriveInput, Field, Fields, Ident, Meta, Attribute};
use quote::quote;
use proc_macro::TokenStream;
use proc_macro2::Span;

// TODO multiple attributes, each generating a new struct
// like #[proj(Request, Response)]
// and also maybe exclusion

// occ as in occlude (we are doing projection)

#[proc_macro_derive(Projection, attributes(occlude))]
pub fn derive_projection(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let proj_name = Ident::new(&format!("Proj{}", name), Span::call_site());

    let struct_fields = match &ast.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("Proj can only be derived for structs"),
    };

    let target_fields: Vec<&Field> = struct_fields
        .iter()
        .filter(|field| {
            ! field
                .attrs
                .iter()
                //.filter_map(Attribute::parse_meta)
                /* .any(|at| match at {
                    Meta::Path(ident) => ident.is_ident("proj"),
                    _ => false,
                }) */
                .any(|at| at.path().is_ident("occlude"))
        })
        .collect();

    let field_ids: Vec<&Ident> = target_fields.iter().map(|field| field.ident.as_ref().unwrap()).collect();
    let field_types: Vec<&syn::Type> = target_fields.iter().map(|field| &field.ty).collect();

    let project_fn = Ident::new(&format!("{}_proj", name), Span::call_site());

    let generated = quote! {
        pub struct #proj_name {
            #(#field_ids: #field_types),*
        }

        impl #name {
            pub fn #project_fn(&self) -> #proj_name {
                #proj_name {
                    #(#field_ids: self.#field_ids.clone()),*
                }
            }
        }

        impl core::convert::From<#name> for #project_fn {
            pub fn from(item: #name) -> #project_fn {
                #project_fn {
                    #(#field_ids: self.#field_ids.clone()),*
                }
            }

        }
    };

    generated.into()
}

