use crate::{
    bound, matcher,
    model::{ConcreteEnum, ConcreteVariant, Shape},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput, Fields, Meta, Path, Variant, Visibility};

pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let concrete_enum = parse(input)?;

    let matcher = matcher::emit(&concrete_enum);
    let bound = bound::emit(&concrete_enum);

    Ok(quote! {
        #matcher
        #bound
    })
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
        foreign: false,
        exported: matches!(input.vis, Visibility::Public(_)),
        bound: bound::parse(&input.attrs)?,
        variants,
        remainder: None,
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
                "Enum variant `{}` is missing its #[concrete(path::to::Type)] attribute",
                variant.ident
            ),
        ));
    };
    let Meta::List(_) = &attr.meta else {
        return Err(syn::Error::new_spanned(
            attr,
            "expected #[concrete(path::to::Type)], naming the type as a path rather than a string",
        ));
    };

    attr.parse_args().map_err(|err: syn::Error| {
        syn::Error::new(
            err.span(),
            format!("#[concrete(..)] must name a type path such as `crate::module::Type`: {err}"),
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
