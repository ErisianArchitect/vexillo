use syn::{Attribute, Ident, Type, Visibility, parse::Parse};

use crate::{const_block::ConstBlock, flag_group::FlagGroup, override_block::OverrideBlock, type_decl::TypeDecl};

pub struct FlagsInput {
    pub attrs: Vec<Attribute>,
    pub type_decl: TypeDecl,
    pub config: OverrideBlock,
    pub consts: ConstBlock,
}

impl Parse for FlagsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let type_decl = input.parse::<TypeDecl>()?;
        let config = input.parse::<OverrideBlock>()?;
        let consts = input.parse::<ConstBlock>()?;
        Ok(Self {
            attrs,
            type_decl,
            config,
            consts,
        })
    }
}