use crate::model::ConcreteEnum;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{Attribute, Token, TypeParamBound, parenthesized, punctuated::Punctuated};

pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<TokenStream>> {
    let mut bounds = Punctuated::<TypeParamBound, Token![+]>::new();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("concrete")) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("bound") {
                return Err(meta.error("expected `bound(Trait + ...)`"));
            }

            let content;
            parenthesized!(content in meta.input);
            bounds.extend(
                Punctuated::<TypeParamBound, Token![+]>::parse_separated_nonempty(&content)?,
            );

            Ok(())
        })?;
    }

    Ok((!bounds.is_empty()).then(|| bounds.into_token_stream()))
}

pub fn emit(concrete_enum: &ConcreteEnum) -> TokenStream {
    let Some(bound) = &concrete_enum.bound else {
        return TokenStream::new();
    };
    let param = fresh_param(bound);
    let assertions = concrete_enum.variants.iter().map(|variant| {
        let concrete = &variant.concrete;
        quote_spanned! { variant.ident.span() => assert_bound::<#concrete>(); }
    });

    quote! {
        const _: () = {
            const fn assert_bound<#param>()
            where
                #param: ?Sized + #bound,
            {
            }

            #(#assertions)*
        };
    }
}

fn fresh_param(bound: &TokenStream) -> Ident {
    let taken = bound.to_string();
    let mut name = String::from("__Concrete");
    while taken.contains(&name) {
        name.push('_');
    }

    Ident::new(&name, Span::call_site())
}
