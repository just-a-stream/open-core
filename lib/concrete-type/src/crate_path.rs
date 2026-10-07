use proc_macro2::{Group, Punct, Spacing, TokenStream, TokenTree};
use quote::ToTokens;
use syn::Path;

pub fn to_macro_path(path: &Path) -> TokenStream {
    to_macro_tokens(path.to_token_stream())
}

fn to_macro_tokens(tokens: TokenStream) -> TokenStream {
    tokens
        .into_iter()
        .flat_map(|tree| match tree {
            TokenTree::Ident(ident) if ident == "crate" => {
                let mut dollar = Punct::new('$', Spacing::Alone);
                dollar.set_span(ident.span());
                vec![TokenTree::Punct(dollar), TokenTree::Ident(ident)]
            }
            TokenTree::Group(group) => {
                let mut nested = Group::new(group.delimiter(), to_macro_tokens(group.stream()));
                nested.set_span(group.span());
                vec![TokenTree::Group(nested)]
            }
            other => vec![other],
        })
        .collect()
}
