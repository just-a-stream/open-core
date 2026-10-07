use crate::{
    crate_path::to_macro_path,
    model::{ConcreteEnum, ConcreteVariant, Shape},
};
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::hash::{DefaultHasher, Hash, Hasher};
use syn::ext::IdentExt;

pub fn emit(concrete_enum: &ConcreteEnum) -> TokenStream {
    let macro_name = match macro_name(&concrete_enum.name) {
        Ok(macro_name) => macro_name,
        Err(err) => return err.into_compile_error(),
    };
    let doc = format!(
        "Matches a `{name}` value and evaluates the body with the matched variant's concrete \
         type bound to the type parameter: `{macro_name}!(value; T => body)`, or \
         `{macro_name}!(value; (T, config) => body)` to bind the variant's configuration too, \
         `()` for a unit variant.",
        name = concrete_enum.name,
    );
    let type_arms: Vec<TokenStream> = concrete_enum
        .variants
        .iter()
        .map(|variant| type_arm(&concrete_enum.path, variant))
        .collect();
    let config_arms: Vec<TokenStream> = concrete_enum
        .variants
        .iter()
        .map(|variant| config_arm(&concrete_enum.path, variant))
        .collect();

    let hidden_name = hidden_name(&concrete_enum.name, &macro_name, &type_arms);
    let (export, reexport_vis) = if concrete_enum.exported {
        (quote! { #[macro_export] }, quote! { pub })
    } else {
        (quote! {}, quote! { pub(crate) })
    };

    let variants = concrete_enum.variants.iter().map(variant_entry);

    quote! {
        #[doc(hidden)]
        #export
        macro_rules! #hidden_name {
            (@concrete_type_variants [$($callback:tt)*] { $($state:tt)* }) => {
                $($callback)*! { { $($state)* } [ #(#variants),* ] }
            };
            ($value:expr; $concrete:ident => $body:block) => {
                match $value {
                    #(#type_arms)*
                }
            };
            ($value:expr; $concrete:ident => $body:expr) => {
                match $value {
                    #(#type_arms)*
                }
            };
            ($value:expr; ($concrete:ident, $config:ident) => $body:block) => {
                match $value {
                    #(#config_arms)*
                }
            };
            ($value:expr; ($concrete:ident, $config:ident) => $body:expr) => {
                match $value {
                    #(#config_arms)*
                }
            };
        }

        #[doc = #doc]
        #[allow(unused_imports)]
        #reexport_vis use #hidden_name as #macro_name;
    }
}

pub fn macro_name(enum_name: &Ident) -> syn::Result<Ident> {
    let snake = enum_name.unraw().to_string().to_case(Case::Snake);

    match snake.as_str() {
        "crate" | "self" | "super" => Err(syn::Error::new(
            enum_name.span(),
            format!("`{enum_name}` snake-cases to `{snake}`, which cannot name a matcher macro"),
        )),
        _ if snake != "gen" && syn::parse_str::<Ident>(&snake).is_ok() => {
            Ok(Ident::new(&snake, enum_name.span()))
        }
        _ => Ok(Ident::new_raw(&snake, enum_name.span())),
    }
}

fn hidden_name(enum_name: &Ident, macro_name: &Ident, type_arms: &[TokenStream]) -> Ident {
    let site = enum_name.span().unwrap();
    let mut hasher = DefaultHasher::new();
    (site.file(), site.line(), site.column()).hash(&mut hasher);
    quote! { #(#type_arms)* }.to_string().hash(&mut hasher);

    format_ident!("__concrete_type_{}_{:016x}", macro_name, hasher.finish())
}

fn variant_entry(variant: &ConcreteVariant) -> TokenStream {
    let ident = &variant.ident;
    let concrete = to_macro_path(&variant.concrete);

    match variant.shape {
        Shape::Unit => quote! { #ident => #concrete },
        Shape::Config => quote! { #ident(_) => #concrete },
    }
}

fn type_arm(enum_path: &TokenStream, variant: &ConcreteVariant) -> TokenStream {
    let ident = &variant.ident;
    let concrete = to_macro_path(&variant.concrete);
    let pattern = match variant.shape {
        Shape::Unit => quote! { #enum_path::#ident },
        Shape::Config => quote! { #enum_path::#ident(_) },
    };

    quote! {
        #pattern => {
            type __ConcreteType = #concrete;
            {
                type $concrete = __ConcreteType;
                $body
            }
        }
    }
}

fn config_arm(enum_path: &TokenStream, variant: &ConcreteVariant) -> TokenStream {
    let ident = &variant.ident;
    let concrete = to_macro_path(&variant.concrete);
    let (pattern, config) = match variant.shape {
        Shape::Unit => (quote! { #enum_path::#ident }, quote! { () }),
        Shape::Config => (quote! { #enum_path::#ident(config) }, quote! { config }),
    };

    quote! {
        #pattern => {
            type __ConcreteType = #concrete;
            {
                type $concrete = __ConcreteType;
                let $config = #config;
                $body
            }
        }
    }
}
