use crate::{
    bound,
    crate_path::to_macro_path,
    matcher,
    model::{ConcreteEnum, ConcreteVariant, Shape},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Path, Token, Visibility, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

pub struct ConcreteMap {
    enums: Vec<ConcreteEnum>,
}

impl Parse for ConcreteMap {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut enums = Vec::new();

        while !input.is_empty() {
            enums.push(parse_enum(input)?);
        }

        Ok(Self { enums })
    }
}

enum Entry {
    Variant(ConcreteVariant),
    Remainder(Token![_], Path),
}

impl Parse for Entry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Token![_]) {
            let underscore = input.parse()?;
            input.parse::<Token![=>]>()?;
            return Ok(Self::Remainder(underscore, input.parse()?));
        }

        let ident = input.parse()?;
        let shape = if input.peek(token::Paren) {
            let field;
            parenthesized!(field in input);
            field.parse::<Token![_]>()?;
            Shape::Config
        } else {
            Shape::Unit
        };
        input.parse::<Token![=>]>()?;

        Ok(Self::Variant(ConcreteVariant {
            ident,
            shape,
            concrete: input.parse()?,
        }))
    }
}

pub fn expand(map: &ConcreteMap) -> TokenStream {
    let items = map.enums.iter().map(|concrete_enum| {
        let matcher = matcher::emit(concrete_enum);
        let bound = bound::emit(concrete_enum);

        quote! {
            #matcher
            #bound
        }
    });

    quote! { #(#items)* }
}

fn parse_enum(input: ParseStream<'_>) -> syn::Result<ConcreteEnum> {
    let attrs = input.call(Attribute::parse_outer)?;
    if let Some(attr) = attrs.iter().find(|attr| !attr.path().is_ident("concrete")) {
        return Err(syn::Error::new_spanned(
            attr,
            "concrete_map! accepts only #[concrete(bound(..))] on an enum",
        ));
    }
    let vis: Visibility = input.parse()?;
    let path = Path::parse_mod_style(input)?;
    input.parse::<Token![=>]>()?;
    let body;
    braced!(body in input);
    let entries = Punctuated::<Entry, Token![,]>::parse_terminated(&body)?;

    let Some(name) = path.segments.last().map(|segment| segment.ident.clone()) else {
        return Err(syn::Error::new_spanned(&path, "expected an enum path"));
    };
    let (variants, remainder) = split_entries(entries)?;

    Ok(ConcreteEnum {
        name,
        path: to_macro_path(&path),
        exported: matches!(vis, Visibility::Public(_)),
        bound: bound::parse(&attrs)?,
        variants,
        remainder,
    })
}

fn split_entries(
    entries: Punctuated<Entry, Token![,]>,
) -> syn::Result<(Vec<ConcreteVariant>, Option<Path>)> {
    let mut variants = Vec::new();
    let mut remainder = None;

    for entry in entries {
        if let Some((underscore, _)) = &remainder {
            return Err(syn::Error::new_spanned(
                underscore,
                "`_ => Type` must be the last entry: it maps every variant not listed before it",
            ));
        }

        match entry {
            Entry::Variant(variant) => variants.push(variant),
            Entry::Remainder(underscore, concrete) => remainder = Some((underscore, concrete)),
        }
    }

    Ok((variants, remainder.map(|(_, concrete)| concrete)))
}
