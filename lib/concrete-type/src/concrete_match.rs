use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, Path, Token, Type, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

struct ConcreteMatch {
    scrutinees: Punctuated<Expr, Token![,]>,
    binders: Punctuated<Binder, Token![,]>,
    body: Expr,
}

impl Parse for ConcreteMatch {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<Token![match]>()?;
        let scrutinees;
        let scrutinees_paren = parenthesized!(scrutinees in input);
        let scrutinees = Punctuated::parse_terminated(&scrutinees)?;
        let arms;
        braced!(arms in input);
        let binders_input;
        let binders_paren = parenthesized!(binders_input in arms);
        let binders = Punctuated::<Binder, Token![,]>::parse_terminated(&binders_input)?;
        arms.parse::<Token![=>]>()?;
        let body = arms.parse()?;
        arms.parse::<Option<Token![,]>>()?;

        if !arms.is_empty() {
            return Err(arms.error("concrete!(match ..) takes one arm, binding the concrete types"));
        }
        if scrutinees.is_empty() {
            return Err(syn::Error::new(
                scrutinees_paren.span.join(),
                "concrete!(match ..) needs at least one enum value to match",
            ));
        }
        if binders.len() != scrutinees.len() {
            return Err(syn::Error::new(
                binders_paren.span.join(),
                format!(
                    "{} binders for {} scrutinees: bind each scrutinee once, as `T: Enum` or `Enum`",
                    binders.len(),
                    scrutinees.len()
                ),
            ));
        }

        Ok(Self {
            scrutinees,
            binders,
            body,
        })
    }
}

struct Binder {
    alias: Ident,
    config: Option<Ident>,
    enum_path: Path,
}

impl Parse for Binder {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let leading = Path::parse_mod_style(input)?;
        let config = if input.peek(token::Paren) {
            let config;
            parenthesized!(config in input);
            Some(config.parse()?)
        } else {
            None
        };

        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let Some(alias) = leading.get_ident().cloned() else {
                return Err(syn::Error::new_spanned(
                    &leading,
                    "a type binder is one identifier, as in `E: Exchange`",
                ));
            };
            return Ok(Self {
                alias,
                config,
                enum_path: Path::parse_mod_style(input)?,
            });
        }

        let Some(alias) = leading.segments.last().map(|segment| segment.ident.clone()) else {
            return Err(syn::Error::new_spanned(&leading, "expected an enum path"));
        };

        Ok(Self {
            alias,
            config,
            enum_path: leading,
        })
    }
}

struct VariantList {
    enum_path: Option<TokenStream>,
    entries: Vec<Entry>,
}

impl Parse for VariantList {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let enum_path;
        parenthesized!(enum_path in input);
        let enum_path: TokenStream = enum_path.parse()?;
        let entries = Punctuated::<Entry, Token![,]>::parse_terminated(input)?;

        Ok(Self {
            enum_path: (!enum_path.is_empty()).then_some(enum_path),
            entries: entries.into_iter().collect(),
        })
    }
}

struct Entry {
    variant: Option<Ident>,
    carries_config: bool,
    concrete: Type,
}

impl Parse for Entry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let variant = if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            None
        } else {
            Some(input.parse()?)
        };
        let carries_config = input.peek(token::Paren);
        if carries_config {
            let field;
            parenthesized!(field in input);
            field.parse::<Token![_]>()?;
        }
        input.parse::<Token![=>]>()?;

        Ok(Self {
            variant,
            carries_config,
            concrete: input.parse()?,
        })
    }
}

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let concrete_match: ConcreteMatch = syn::parse2(input.clone())?;
    let original = Group::new(Delimiter::Brace, input);

    Ok(request_variants(&concrete_match, &original, &[]))
}

pub fn step(input: TokenStream) -> syn::Result<TokenStream> {
    let malformed = || syn::Error::new(Span::call_site(), "malformed concrete! step");
    let mut trees = input.into_iter();
    let (Some(TokenTree::Group(state)), Some(TokenTree::Group(variants))) =
        (trees.next(), trees.next())
    else {
        return Err(malformed());
    };
    let mut state = state.stream().into_iter();
    let Some(TokenTree::Group(original)) = state.next() else {
        return Err(malformed());
    };
    let mut lists: Vec<TokenStream> = state
        .filter_map(|tree| match tree {
            TokenTree::Group(group) => Some(group.stream()),
            _ => None,
        })
        .collect();
    lists.push(variants.stream());

    let concrete_match: ConcreteMatch = syn::parse2(original.stream())?;
    if lists.len() < concrete_match.binders.len() {
        return Ok(request_variants(&concrete_match, &original, &lists));
    }

    let lists = lists
        .into_iter()
        .map(syn::parse2)
        .collect::<syn::Result<Vec<VariantList>>>()?;

    Ok(emit_match(&concrete_match, &lists))
}

fn request_variants(
    concrete_match: &ConcreteMatch,
    original: &Group,
    lists: &[TokenStream],
) -> TokenStream {
    let enum_path = &concrete_match.binders[lists.len()].enum_path;
    let lists = lists.iter().map(|list| quote! { [#list] });

    quote! {
        #enum_path!(@concrete_type_variants [::concrete_type::__concrete_step] {
            #original #(#lists)*
        })
    }
}

fn emit_match(concrete_match: &ConcreteMatch, lists: &[VariantList]) -> TokenStream {
    let body = concrete_match.body.to_token_stream();
    let enum_paths: Vec<TokenStream> = concrete_match
        .binders
        .iter()
        .zip(lists)
        .map(|(binder, list)| {
            list.enum_path
                .clone()
                .unwrap_or_else(|| binder.enum_path.to_token_stream())
        })
        .collect();
    let arms = combinations(lists)
        .into_iter()
        .map(|combination| emit_arm(&concrete_match.binders, &enum_paths, &combination, &body));
    let scrutinees = concrete_match.scrutinees.iter();

    quote! {
        match (#(#scrutinees,)*) {
            #(#arms)*
        }
    }
}

fn combinations(lists: &[VariantList]) -> Vec<Vec<&Entry>> {
    lists.iter().fold(vec![Vec::new()], |prefixes, list| {
        prefixes
            .into_iter()
            .flat_map(|prefix| {
                list.entries.iter().map(move |entry| {
                    let mut combination = prefix.clone();
                    combination.push(entry);
                    combination
                })
            })
            .collect()
    })
}

fn emit_arm(
    binders: &Punctuated<Binder, Token![,]>,
    enum_paths: &[TokenStream],
    combination: &[&Entry],
    body: &TokenStream,
) -> TokenStream {
    let (patterns, (resolutions, bindings)): (Vec<TokenStream>, (Vec<_>, Vec<_>)) = binders
        .iter()
        .zip(enum_paths)
        .zip(combination)
        .enumerate()
        .map(|(index, ((binder, enum_path), entry))| {
            let resolved = format_ident!("__ConcreteType{index}");
            let concrete = &entry.concrete;
            let (pattern, binding) = emit_binding(binder, enum_path, entry, &resolved);

            (pattern, (quote! { type #resolved = #concrete; }, binding))
        })
        .unzip();

    quote! {
        (#(#patterns,)*) => {
            #(#resolutions)*
            {
                #(#bindings)*
                #body
            }
        }
    }
}

fn emit_binding(
    binder: &Binder,
    enum_path: &TokenStream,
    entry: &Entry,
    resolved: &Ident,
) -> (TokenStream, TokenStream) {
    let alias = &binder.alias;
    let type_alias = quote! { type #alias = #resolved; };

    match (&entry.variant, entry.carries_config, &binder.config) {
        (Some(variant), true, Some(config)) => {
            (quote! { #enum_path::#variant(#config) }, type_alias)
        }
        (Some(variant), true, None) => (quote! { #enum_path::#variant(_) }, type_alias),
        (Some(variant), false, config) => (
            quote! { #enum_path::#variant },
            unit_config(type_alias, config.as_ref()),
        ),
        (None, _, config) => (quote! { _ }, unit_config(type_alias, config.as_ref())),
    }
}

fn unit_config(type_alias: TokenStream, config: Option<&Ident>) -> TokenStream {
    match config {
        Some(config) => quote! { #type_alias let #config = (); },
        None => type_alias,
    }
}
