use syn::{parse::Parse, token::Priv, Visibility};
use syn::{Token, parse_quote};

#[derive(Clone)]
pub enum Vis {
    Private(Priv),
    Syn(Visibility),
}

impl Parse for Vis {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![priv]) {
            Self::Private(input.parse::<Token![priv]>()?)
        } else {
            let vis: Visibility = input.parse()?;
            Self::Syn(vis)
        })
    }
}

impl Vis {
    pub fn resolve(&self, default: Option<&Visibility>) -> Visibility {
        match (self, default) {
            (Self::Private(_priv), _) => parse_quote!(pub(self)),
            (Self::Syn(Visibility::Inherited), Some(visibility)) => visibility.clone(),
            (Self::Syn(visibility), _) => visibility.clone(),
        }
    }
}