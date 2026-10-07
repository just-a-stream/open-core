#![doc(html_root_url = "https://docs.rs/concrete-type")]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use crate::concrete_map::ConcreteMap;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod bound;
mod concrete_map;
mod crate_path;
mod derive;
mod matcher;
mod model;

/// Maps each variant of an enum to a concrete type.
///
/// Every variant carries `#[concrete(path::to::Type)]` and either no data or exactly one
/// unnamed field holding its configuration. The derive emits a matcher macro named after the
/// enum in snake case (`exchange!` for `Exchange`) with two forms:
///
/// - `exchange!(value; T => body)` evaluates the body with `T` aliased to the matched
///   variant's concrete type;
/// - `exchange!(value; (T, config) => body)` also binds the variant's configuration to
///   `config`, `()` for a unit variant.
///
/// `#[concrete(bound(Trait + Send + 'static))]` on the enum asserts that every concrete type
/// satisfies the bound list, failing at the offending variant.
///
/// A path starting with `crate::` is rewritten to `$crate::`, so the macro resolves it from any
/// crate; any other path resolves where the macro is called.
///
/// For macros that build on the matcher, `exchange!(@concrete_type_variants [callback] { state })`
/// expands to `callback! { { state } [ Variant => Type, Data(_) => Type ] }`, listing every
/// variant with its concrete type, a data variant marked `(_)`.
#[proc_macro_derive(Concrete, attributes(concrete))]
pub fn derive_concrete(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Maps the variants of an enum defined elsewhere, which cannot carry `#[derive(Concrete)]`.
///
/// ```text
/// concrete_map! {
///     #[concrete(bound(Trait + Send))]
///     pub path::to::Enum => {
///         Unit => path::to::Type,
///         Data(_) => path::to::Other,
///         _ => path::to::Remainder,
///     }
/// }
/// ```
///
/// Emits the same matcher macro as [`Concrete`](derive@Concrete), named after the enum in snake
/// case beside the invocation. `Data(_)` marks a variant carrying one unnamed field of
/// configuration, and a final `_ => Type` maps every variant not listed. The bound and a `pub`
/// visibility, which exports the macro to other crates, are optional; several enums may share one
/// invocation. The macro's patterns spell the enum as written here, with `crate::` rewritten to
/// `$crate::`.
#[proc_macro]
pub fn concrete_map(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as ConcreteMap);

    concrete_map::expand(&map).into()
}
