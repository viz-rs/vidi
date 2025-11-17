//! Macros for Viz Web Framework
//!
//! Generators for handler
//!
//! # handler
//!
//! ## Example
//!
//! ```
//! # use viz_core::{IntoResponse, Result};
//! # use viz_macros::handler;
//!
//! #[handler]
//! fn about() -> impl IntoResponse {
//! }
//!
//! #[handler]
//! async fn index() -> impl IntoResponse {
//!     ()
//! }
//!
//! #[handler]
//! async fn get_user() -> Result<impl IntoResponse> {
//!     Ok(())
//! }
//! ```

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, feature(doc_cfg))]

use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Result, ReturnType};

/// Transforms `extract-handler` to a Handler instance.
#[proc_macro_attribute]
pub fn handler(_args: TokenStream, input: TokenStream) -> TokenStream {
    generate_handler(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn generate_handler(input: TokenStream) -> Result<TokenStream> {
    let ast = syn::parse::<ItemFn>(input)?;
    let vis = &ast.vis;
    let docs = ast
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect::<Vec<_>>();
    let name = ast.sig.ident.clone();
    let asyncness = if ast.sig.asyncness.is_some() {
        Some(quote!(.await))
    } else {
        None
    };
    let mut out = quote!(Ok(res));
    let mut is_ok_type = false;
    match &ast.sig.output {
        // ()
        ReturnType::Default => {
            is_ok_type = true;
        }
        ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Path(path) => {
                if let Some(seg) = &path.path.segments.first() {
                    is_ok_type = true;
                    // T
                    // impl IntoResponse
                    // Result<T>
                    // Result<impl IntoResponse>
                    if seg.ident == "Result" {
                        out = quote!(res);
                    }
                }
            }
            syn::Type::ImplTrait(i) => {
                if let Some(syn::TypeParamBound::Trait(d)) = &i.bounds.first() {
                    // T
                    // impl IntoResponse
                    if matches!(d.path.get_ident(), Some(ident) if ident == "IntoResponse") {
                        is_ok_type = true;
                    }
                }
            }
            syn::Type::Tuple(_) => {
                // (T,...)
                is_ok_type = true;
            }
            _ => {
                is_ok_type = false;
            }
        },
    }

    if !is_ok_type {
        out = quote!();
    }

    let extractors =
        ast.sig
            .inputs
            .clone()
            .into_iter()
            .fold(Vec::new(), |mut extractors, input| {
                if let FnArg::Typed(pat) = input {
                    let ty = &pat.ty;
                    extractors
                        .push(quote!(<#ty as viz_core::FromRequest>::extract(&mut req).await?));
                }
                extractors
            });

    let stream = quote! {
        #(#docs)*
        #[allow(non_camel_case_types)]
        #[derive(Clone)]
        #vis struct #name;

        #[viz_core::async_trait]
        impl viz_core::Handler<viz_core::Request> for #name
        {
            type Output = viz_core::Result<viz_core::Response>;

            #[allow(unused, unused_mut)]
            async fn call(&self, mut req: viz_core::Request) -> Self::Output {
                #ast
                let res = #name(#(#extractors),*)#asyncness;
                #out.map(viz_core::IntoResponse::into_response)
            }
        }
    };

    Ok(stream.into())
}
