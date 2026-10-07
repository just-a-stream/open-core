use crate::{
    matcher,
    model::{ConcreteEnum, ConcreteVariant, Shape},
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit, Meta, Path, Variant};

pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let concrete_enum = parse(input)?;

    Ok(matcher::emit(&concrete_enum))
}

fn parse(input: &DeriveInput) -> syn::Result<ConcreteEnum> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "Concrete can only be derived for enums",
        ));
    };

    let variants = data
        .variants
        .iter()
        .map(parse_variant)
        .collect::<syn::Result<_>>()?;

    Ok(ConcreteEnum {
        name: input.ident.clone(),
        path: input.ident.to_token_stream(),
        variants,
    })
}

fn parse_variant(variant: &Variant) -> syn::Result<ConcreteVariant> {
    let concrete = parse_concrete_path(variant)?;
    let shape = parse_shape(variant)?;

    Ok(ConcreteVariant {
        ident: variant.ident.clone(),
        shape,
        concrete,
    })
}

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

fn parse_shape(variant: &Variant) -> syn::Result<Shape> {
    match &variant.fields {
        Fields::Unit => Ok(Shape::Unit),
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(Shape::Config),
        _ => Err(syn::Error::new_spanned(
            &variant.fields,
            format!(
                "Enum variant `{}` must be a unit variant or carry exactly one unnamed field \
                 holding its configuration",
                variant.ident
            ),
        )),
    }
}
