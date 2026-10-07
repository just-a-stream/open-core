#![doc(html_root_url = "https://docs.rs/concrete-type")]
#![warn(missing_docs)]

//! # Concrete Type
//!
//! A procedural macro library for mapping enum variants to concrete types.
//!
//! This crate provides two main derive macros:
//!
//! - [`Concrete`] - For enums where each variant maps to a specific concrete type
//! - [`ConcreteConfig`] - For enums where each variant has associated configuration data
//!   and maps to a specific concrete type
//!
//! These macros enable type-level programming based on runtime enum values by generating
//! helper methods and macros that provide access to the concrete types associated with
//! enum variants.
//!
//! ## Path Resolution
//!
//! When specifying concrete types, you can use two path formats:
//!
//! - `crate::path::to::Type` - Use this for types defined in the same crate as the enum.
//!   The macro will transform this to `$crate::path::to::Type` for proper hygiene,
//!   allowing the generated macro to work both within the defining crate and from external crates.
//!
//! - `other_crate::path::to::Type` - Use this for types from external crates.
//!   The path is used as-is.
//!
//! ## Examples
//!
//! ### Basic Usage with `Concrete`
//!
//! ```rust
//! use concrete_type::Concrete;
//!
//! mod exchanges {
//!     pub struct Binance;
//!     pub struct Coinbase;
//!
//!     impl Binance {
//!         pub fn new() -> Self { Binance }
//!         pub fn name(&self) -> &'static str { "binance" }
//!     }
//!
//!     impl Coinbase {
//!         pub fn new() -> Self { Coinbase }
//!         pub fn name(&self) -> &'static str { "coinbase" }
//!     }
//! }
//!
//! #[derive(Concrete, Clone, Copy)]
//! enum Exchange {
//!     #[concrete = "crate::exchanges::Binance"]
//!     Binance,
//!     #[concrete = "crate::exchanges::Coinbase"]
//!     Coinbase,
//! }
//!
//! fn main() {
//!     // Use the auto-generated exchange! macro for type-level dispatch
//!     let exchange = Exchange::Binance;
//!     let name = exchange!(exchange; ExchangeImpl => {
//!         // ExchangeImpl is aliased to the concrete type
//!         let instance = ExchangeImpl::new();
//!         instance.name()
//!     });
//!     assert_eq!(name, "binance");
//! }
//! ```
//!
//! ### Using `ConcreteConfig` with Configuration Data
//!
//! ```rust
//! use concrete_type::ConcreteConfig;
//!
//! // Define concrete types and configuration types
//! mod exchanges {
//!     pub trait ExchangeApi {
//!         type Config;
//!         fn new(config: Self::Config) -> Self;
//!         fn name(&self) -> &'static str;
//!     }
//!
//!     pub struct Binance;
//!     pub struct BinanceConfig {
//!         pub api_key: String,
//!     }
//!
//!     impl ExchangeApi for Binance {
//!         type Config = BinanceConfig;
//!         fn new(_: Self::Config) -> Self { Self }
//!         fn name(&self) -> &'static str { "binance" }
//!     }
//! }
//!
//! // Define the enum with concrete type mappings and config data
//! #[derive(ConcreteConfig)]
//! enum ExchangeConfig {
//!     #[concrete = "crate::exchanges::Binance"]
//!     Binance(exchanges::BinanceConfig),
//! }
//!
//! fn main() {
//!     // Using the auto-generated macro with access to both type and config
//!     let config = ExchangeConfig::Binance(
//!         exchanges::BinanceConfig { api_key: "secret".to_string() }
//!     );
//!
//!     let name = exchange_config!(config; (Exchange, cfg) => {
//!         // Inside this block:
//!         // - Exchange is the concrete type
//!         // - cfg is the configuration instance (BinanceConfig)
//!         use exchanges::ExchangeApi;
//!         Exchange::new(cfg).name()
//!     });
//!     assert_eq!(name, "binance");
//! }
//! ```
//!
//! See the crate documentation and examples for more details.

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, ExprLit, Fields, GenericArgument, Ident, Lit, Meta, Path,
    PathArguments, ReturnType, Type, Variant, parse_macro_input,
};

struct Mapping<'a> {
    variant: &'a Variant,
    concrete: TokenStream2,
}

enum ConfigField {
    Data,
    Unit,
}

/// A derive macro that implements the mapping between enum variants and concrete types.
///
/// This macro is designed for enums where each variant maps to a specific concrete type.
/// Each variant must be annotated with the `#[concrete = "path::to::Type"]` attribute that
/// specifies the concrete type that the variant represents.
///
/// # Path Resolution
///
/// - Use `crate::path::to::Type` for types in the same crate (transforms to `$crate::`)
/// - Use `other_crate::path::to::Type` for types from external crates (used as-is)
///
/// # Generated Code
///
/// The macro generates a macro with the snake_case name of the enum
/// (e.g., `exchange!` for `Exchange`, `strategy_kind!` for `StrategyKind`) that can be used
/// to execute code with the concrete type.
///
/// # Example
///
/// ```rust
/// use concrete_type::Concrete;
///
/// mod strategies {
///     pub struct StrategyA;
///     pub struct StrategyB;
/// }
///
/// #[derive(Concrete)]
/// enum StrategyKind {
///     #[concrete = "crate::strategies::StrategyA"]
///     StrategyA,
///     #[concrete = "crate::strategies::StrategyB"]
///     StrategyB,
/// }
///
/// fn main() {
///     // The generated macro is named after the enum in snake_case
///     let strategy = StrategyKind::StrategyA;
///     let result = strategy_kind!(strategy; T => {
///         // T is aliased to strategies::StrategyA here
///         std::any::type_name::<T>()
///     });
///     assert!(result.ends_with("strategies::StrategyA"));
/// }
/// ```
///
/// This enables type-level programming with enums, where you can define enum variants and
/// map them to concrete type implementations.
#[proc_macro_derive(Concrete, attributes(concrete))]
pub fn derive_concrete(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand_concrete(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// A derive macro that implements the mapping between enum variants with associated data and
/// concrete types.
///
/// This macro is designed for enums where each variant has associated configuration data and maps
/// to a specific concrete type. Each variant must be annotated with the
/// `#[concrete = "path::to::Type"]` attribute and contain a single field (no tuples)
/// that holds the configuration data for that concrete type. If the variant has no data, then it
/// defaults to the unit type `()`.
///
/// # Path Resolution
///
/// - Use `crate::path::to::Type` for types in the same crate (transforms to `$crate::`)
/// - Use `other_crate::path::to::Type` for types from external crates (used as-is)
///
/// # Generated Code
///
/// The macro generates:
/// 1. A `config` method that returns a reference to the configuration data.
/// 2. A macro with the snake_case name of the enum + "_config" (with "Config" suffix removed if present)
///    that allows access to both the concrete type and configuration data
///
/// # Example
///
/// ```rust
/// use concrete_type::ConcreteConfig;
///
/// // Define concrete types and configuration types
/// #[derive(Debug)]
/// struct BinanceConfig {
///     api_key: String,
/// }
///
/// struct Binance;
///
/// struct Okx;
///
/// #[derive(ConcreteConfig)]
/// enum ExchangeConfig {
///     #[concrete = "Binance"]
///     Binance(BinanceConfig),
///     #[concrete = "Okx"]
///     Okx,
/// }
///
/// fn main() {
///     // Using the generated macro for a variant with config data
///     let config = ExchangeConfig::Binance(BinanceConfig { api_key: "key".to_string() });
///     let result = exchange_config!(config; (Exchange, cfg) => {
///         // "Exchange" symbol is concrete type Binance
///         // "cfg" symbol is the BinanceConfig instance
///         format!("{} with config: {:?}", std::any::type_name::<Exchange>(), cfg)
///     });
///     assert!(result.ends_with("Binance with config: BinanceConfig { api_key: \"key\" }"));
///
///     // Using the generated macro for a variant without config data
///     let config = ExchangeConfig::Okx;
///     let result = exchange_config!(config; (Exchange, cfg) => {
///         // "Exchange" symbol is concrete type Okx
///         // "cfg" symbol is the unit value () (since the Okx variant doesn't have config)
///         format!("{} with config: {:?}", std::any::type_name::<Exchange>(), cfg)
///     });
///     assert!(result.ends_with("Okx with config: ()"));
/// }
/// ```
#[proc_macro_derive(ConcreteConfig, attributes(concrete))]
pub fn derive_concrete_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand_concrete_config(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_concrete(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let enum_name = &input.ident;
    let mappings = parse_mappings(input, "Concrete")?;
    mappings
        .iter()
        .try_for_each(|mapping| require_unit_variant(mapping.variant))?;

    let macro_name = Ident::new(
        &enum_name.to_string().to_case(Case::Snake),
        enum_name.span(),
    );
    let doc = format!(
        "Dispatches on a `{enum_name}` value, running the block with the matched variant's \
         concrete type bound to the type parameter."
    );
    let arms = mappings.iter().map(|mapping| {
        let variant = &mapping.variant.ident;
        let concrete = &mapping.concrete;
        quote! {
            #enum_name::#variant => {
                type $type_param = #concrete;
                $code_block
            }
        }
    });

    Ok(emit_dispatch_macro(
        &macro_name,
        &doc,
        quote! { $type_param:ident },
        arms,
    ))
}

fn expand_concrete_config(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let enum_name = &input.ident;
    let mappings = parse_mappings(input, "ConcreteConfig")?
        .into_iter()
        .map(|mapping| config_field(mapping.variant).map(|field| (mapping, field)))
        .collect::<syn::Result<Vec<_>>>()?;

    let macro_name = config_macro_name(enum_name);
    let doc = format!(
        "Dispatches on a `{enum_name}` value, running the block with the matched variant's \
         concrete type bound to the type parameter and its configuration bound to the config \
         parameter."
    );
    let arms = mappings.iter().map(|(mapping, field)| {
        let variant = &mapping.variant.ident;
        let concrete = &mapping.concrete;
        match field {
            ConfigField::Data => quote! {
                #enum_name::#variant(config) => {
                    type $type_param = #concrete;
                    let $config_param = config;
                    $code_block
                }
            },
            ConfigField::Unit => quote! {
                #enum_name::#variant => {
                    type $type_param = #concrete;
                    let $config_param = ();
                    $code_block
                }
            },
        }
    });
    let accessor_arms = mappings.iter().map(|(mapping, field)| {
        let variant = &mapping.variant.ident;
        match field {
            ConfigField::Data => quote! { #enum_name::#variant(config) => config },
            ConfigField::Unit => quote! { #enum_name::#variant => &() },
        }
    });

    let dispatch_macro = emit_dispatch_macro(
        &macro_name,
        &doc,
        quote! { ($type_param:ident, $config_param:ident) },
        arms,
    );

    Ok(quote! {
        #dispatch_macro

        impl #enum_name {
            /// Returns a reference to the configuration data associated with this enum variant
            /// Unit variants return a reference to the unit type `()`
            pub fn config(&self) -> &dyn ::core::any::Any {
                match self {
                    #(#accessor_arms),*
                }
            }
        }
    })
}

fn emit_dispatch_macro(
    macro_name: &Ident,
    doc: &str,
    params: TokenStream2,
    arms: impl Iterator<Item = TokenStream2>,
) -> TokenStream2 {
    quote! {
        #[doc = #doc]
        #[macro_export]
        macro_rules! #macro_name {
            ($enum_instance:expr; #params => $code_block:block) => {
                match $enum_instance {
                    #(#arms),*
                }
            };
        }
    }
}

fn config_macro_name(enum_name: &Ident) -> Ident {
    let name = enum_name.to_string();
    let base = name.strip_suffix("Config").unwrap_or(&name);

    Ident::new(
        &format!("{}_config", base.to_case(Case::Snake)),
        enum_name.span(),
    )
}

fn parse_mappings<'a>(input: &'a DeriveInput, derive: &str) -> syn::Result<Vec<Mapping<'a>>> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            format!("{derive} can only be derived for enums"),
        ));
    };

    data.variants
        .iter()
        .map(|variant| {
            let path = parse_concrete_path(variant)?;

            Ok(Mapping {
                variant,
                concrete: transform_path_for_macro(&path),
            })
        })
        .collect()
}

fn require_unit_variant(variant: &Variant) -> syn::Result<()> {
    if matches!(variant.fields, Fields::Unit) {
        return Ok(());
    }

    Err(syn::Error::new_spanned(
        &variant.fields,
        format!(
            "Enum variant `{}` carries data, which Concrete cannot bind; derive ConcreteConfig instead",
            variant.ident
        ),
    ))
}

fn config_field(variant: &Variant) -> syn::Result<ConfigField> {
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(ConfigField::Data),
        Fields::Unit => Ok(ConfigField::Unit),
        _ => Err(syn::Error::new_spanned(
            &variant.ident,
            format!(
                "Enum variant `{}` must either be a unit variant or have exactly one unnamed field for config",
                variant.ident
            ),
        )),
    }
}

/// Helper function to extract the concrete type path from a variant's `#[concrete = "..."]` attribute
fn parse_concrete_path(variant: &Variant) -> syn::Result<Path> {
    let Some(attr) = variant
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("concrete"))
    else {
        return Err(syn::Error::new_spanned(
            &variant.ident,
            format!(
                "Enum variant `{}` is missing the #[concrete = \"...\"] attribute",
                variant.ident
            ),
        ));
    };
    let Meta::NameValue(meta) = &attr.meta else {
        return Err(syn::Error::new_spanned(
            attr,
            "expected #[concrete = \"path::to::Type\"]",
        ));
    };
    let Expr::Lit(ExprLit {
        lit: Lit::Str(path),
        ..
    }) = &meta.value
    else {
        return Err(syn::Error::new_spanned(
            &meta.value,
            "expected a string literal naming a type path, as in #[concrete = \"path::to::Type\"]",
        ));
    };

    path.parse().map_err(|err: syn::Error| {
        syn::Error::new(
            err.span(),
            format!(
                "#[concrete = \"...\"] must name a type path such as \"crate::module::Type\": {err}"
            ),
        )
    })
}

/// Transforms a path for use in generated macro code.
///
/// If the path starts with `crate::`, it transforms to `$crate::` for proper
/// macro hygiene. This allows the generated macro to work correctly both within
/// the defining crate and from external crates.
///
/// This function also recursively transforms any `crate::` paths inside generic
/// arguments (e.g., `Wrapper<crate::inner::Type>` becomes `Wrapper<$crate::inner::Type>`).
///
/// Paths that don't start with `crate::` are returned as-is (after processing their generics).
fn transform_path_for_macro(path: &Path) -> TokenStream2 {
    let starts_with_crate = path.segments.first().is_some_and(|s| s.ident == "crate");
    let transformed_segments: Vec<TokenStream2> = path
        .segments
        .iter()
        .skip(usize::from(starts_with_crate))
        .map(|segment| {
            let ident = &segment.ident;
            let args = transform_path_arguments(&segment.arguments);
            quote! { #ident #args }
        })
        .collect();
    let leading_colon = &path.leading_colon;

    if starts_with_crate && !transformed_segments.is_empty() {
        quote! { $crate :: #(#transformed_segments)::* }
    } else if transformed_segments.is_empty() {
        quote! { #path }
    } else {
        quote! { #leading_colon #(#transformed_segments)::* }
    }
}

/// Transform path arguments (generic parameters), recursively handling nested `crate::` paths.
fn transform_path_arguments(args: &PathArguments) -> TokenStream2 {
    match args {
        PathArguments::None => quote! {},
        PathArguments::AngleBracketed(angle) => {
            let transformed_args = angle.args.iter().map(|arg| match arg {
                GenericArgument::Type(ty) => transform_type(ty),
                other => quote! { #other },
            });
            quote! { < #(#transformed_args),* > }
        }
        PathArguments::Parenthesized(paren) => {
            let inputs = paren.inputs.iter().map(|input| transform_type(&input.ty));
            let output = match &paren.output {
                ReturnType::Default => quote! {},
                ReturnType::Type(arrow, ty) => {
                    let transformed = transform_type(ty);
                    quote! { #arrow #transformed }
                }
            };
            quote! { ( #(#inputs),* ) #output }
        }
    }
}

/// Transform a type, recursively handling `crate::` paths within.
fn transform_type(ty: &Type) -> TokenStream2 {
    match ty {
        Type::Path(type_path) => {
            let transformed = transform_path_for_macro(&type_path.path);
            match &type_path.qself {
                Some(qself) => {
                    let qself_ty = transform_type(&qself.ty);
                    quote! { < #qself_ty > :: #transformed }
                }
                None => transformed,
            }
        }
        Type::Reference(ref_type) => {
            let lifetime = &ref_type.lifetime;
            let mutability = &ref_type.mutability;
            let elem = transform_type(&ref_type.elem);
            quote! { & #lifetime #mutability #elem }
        }
        Type::Tuple(tuple) => {
            let elems = tuple.elems.iter().map(transform_type);
            quote! { ( #(#elems),* ) }
        }
        Type::Slice(slice) => {
            let elem = transform_type(&slice.elem);
            quote! { [ #elem ] }
        }
        Type::Array(array) => {
            let elem = transform_type(&array.elem);
            let len = &array.len;
            quote! { [ #elem ; #len ] }
        }
        Type::Ptr(ptr) => {
            let mutability = &ptr.mutability;
            let elem = transform_type(&ptr.elem);
            quote! { * #mutability #elem }
        }
        other => quote! { #other },
    }
}
