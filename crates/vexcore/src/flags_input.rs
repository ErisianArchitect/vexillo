use syn::{Attribute, Ident, Type, Visibility, parse::Parse};

use crate::{flag_group::FlagGroup, impl_block::ImplBlock, type_decl::TypeDecl};

pub struct FlagsInput {
    pub attrs: Vec<Attribute>,
    pub type_decl: TypeDecl,
    pub config: ImplBlock,
    pub groups: Vec<FlagGroup>,
}

impl Parse for FlagsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let type_decl = input.parse::<TypeDecl>()?;
        let config = input.parse::<ImplBlock>()?;
        let mut groups = Vec::new();
        while !input.is_empty() {
            groups.push(input.parse()?);
        }
        Ok(Self {
            attrs,
            type_decl,
            config,
            groups,
        })
    }
}