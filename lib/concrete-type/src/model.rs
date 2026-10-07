use proc_macro2::{Ident, TokenStream};
use syn::Path;

pub struct ConcreteEnum {
    pub name: Ident,
    pub path: TokenStream,
    pub exported: bool,
    pub bound: Option<TokenStream>,
    pub variants: Vec<ConcreteVariant>,
    pub remainder: Option<Path>,
}

pub struct ConcreteVariant {
    pub ident: Ident,
    pub shape: Shape,
    pub concrete: Path,
}

pub enum Shape {
    Unit,
    Config,
}
