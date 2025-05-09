use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use std::string::ToString;
use syn::{DeriveInput, Fields, Expr, ExprPath};

#[proc_macro_derive(FieldIter)]
pub fn field_iter_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;

    let named = match ast.data {
        syn::Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("FieldIter only works on structs with named fields"),
        },
        _ => {
            panic!("FieldIter only works on structs with named fields")
        }
    };

    let idents: Vec<_> = named.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let names: Vec<_> = idents.iter().map(|i| i.to_string()).collect();

    let expanded = quote! {
        impl #name {
            pub fn iter_fields(&self) -> Vec<(&'static str, &dyn std::fmt::Debug)> {
                vec![
                #((#names, &self.#idents as &dyn std::fmt::Debug)),*
                ]
            }
        }
    };

    expanded.into()
}

/// Split a camel‑cased identifier into its capital‑letter‑delimited parts
/// and return the *last* segment.
///
/// `KeyM`  → "M"
/// `Backspace` → "Backspace" (only one part)
/// `Digit5` → "Digit5"
fn last_camel_segment(ident: &str) -> String {
    let mut current = String::new();
    let mut segments = Vec::new();

    for c in ident.chars() {
        if c.is_uppercase() && !current.is_empty() {
            segments.push(current);
            current = String::new();
        }
        current.push(c);
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments.pop().unwrap_or_default()
}

/// `key_name!(KeyCode::KeyM)` → `"M"` (`&'static str` literal)
#[proc_macro_error]
#[proc_macro]
pub fn key_name(item: TokenStream) -> TokenStream {
    // Parse whatever tokens the caller gave as a generic Rust expression.
    // We expect something like a path expression (`KeyCode::KeyM`).
    let expr = syn::parse_macro_input!(item as Expr);

    let variant_ident = match &expr {
        Expr::Path(ExprPath { path, .. }) => {
            path.segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_else(|| abort!(expr, "Expected an enum variant path"))
        }
        _ => abort!(expr, "Expected an enum variant path, e.g. `KeyCode::KeyM`"),
    };

    let final_piece = last_camel_segment(&variant_ident);

    // Build a string literal token
    let lit = syn::LitStr::new(&final_piece, proc_macro2::Span::call_site());

    // Expand to that literal
    TokenStream::from(quote! { #lit })
}