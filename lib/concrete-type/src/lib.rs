#![doc(html_root_url = "https://docs.rs/concrete-type")]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use crate::concrete_map::ConcreteMap;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod bound;
mod concrete_map;
mod concrete_match;
mod crate_path;
mod derive;
mod matcher;
mod model;

/// Maps each variant to the type named by `#[concrete(path::to::Type)]`.
///
/// Emits a matcher named after the enum in snake case: `exchange!(value; T => body)` runs the
/// body with `T` set to the matched variant's type, and `exchange!(value; (T, config) => body)`
/// also binds the variant's single field (`()` for a unit variant). An optional
/// `#[concrete(bound(Trait + Send))]` on the enum checks every type at its variant:
///
/// ```compile_fail,E0277
/// # use concrete_type::Concrete;
/// trait Venue {}
/// struct Binance;
///
/// #[derive(Concrete)]
/// #[concrete(bound(Venue))]
/// # #[concrete(bound(Send))]
/// enum Exchange { #[concrete(Binance)] Binance }
/// # fn main() {}
/// ```
#[proc_macro_derive(Concrete, attributes(concrete))]
pub fn derive_concrete(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Matches several enums at once, binding each one's concrete type.
///
/// `concrete!(match (exchange, strategy) { (E: Exchange, S: Strategy) => body })`, where
/// `E(config): Exchange` also binds a variant's field and a bare `Exchange` binds the type under
/// the enum's own name.
#[proc_macro]
pub fn concrete(input: TokenStream) -> TokenStream {
    concrete_match::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Maps the variants of an enum from another crate, which cannot take the derive.
///
/// `concrete_map! { other::Enum => { Unit => Type, Data(_) => Type, _ => Type } }` emits the
/// same matcher as [`Concrete`](derive@Concrete); `_` maps every variant not listed. In another
/// module, `concrete!` binds the enum through the map's module, as in `T: maps::Enum`.
#[proc_macro]
pub fn concrete_map(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as ConcreteMap);

    concrete_map::expand(&map).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __concrete_step(input: TokenStream) -> TokenStream {
    concrete_match::step(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
