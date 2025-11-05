use syn::{Error, Token, braced, parse::Parse};
use crate::{flag_group::DeclareItem, vis::Vis};


pub struct ConstBlock {
    pub vis: Vis,
    pub groups: Vec<DeclareItem>,
}

impl Parse for ConstBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        _=input.parse::<Token![const]>()?;
        let inner;
        braced!(inner in input);
        let mut groups = Vec::new();
        while !inner.is_empty() {
            groups.push(inner.parse()?);
        }
        if groups.is_empty() {
            return Err(Error::new(input.span(), "Must declare at least one flag constant."));
        }
        Ok(Self {
            vis,
            groups,
        })
    }
}