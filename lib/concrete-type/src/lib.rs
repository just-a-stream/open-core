#![doc(html_root_url = "https://docs.rs/concrete-type")]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod crate_path;
mod derive;
mod matcher;
mod model;

/// Maps each variant of an enum to a concrete type.
///
/// Every variant carries `#[concrete = "path::to::Type"]` and either no data or exactly one
/// unnamed field holding its configuration. The derive emits a matcher macro named after the
/// enum in snake case (`exchange!` for `Exchange`) with two forms:
///
/// - `exchange!(value; T => body)` evaluates the body with `T` aliased to the matched
///   variant's concrete type;
/// - `exchange!(value; (T, config) => body)` also binds the variant's configuration to
///   `config`, `()` for a unit variant.
///
/// A path starting with `crate::` is rewritten to `$crate::`, so the macro resolves it from any
/// crate; any other path resolves where the macro is called.
#[proc_macro_derive(Concrete, attributes(concrete))]
pub fn derive_concrete(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
