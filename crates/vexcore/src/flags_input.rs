use quote::{quote, ToTokens};
use syn::{Ident, parse::Parse};

use crate::{const_block::ConstBlock, override_block::OverrideBlock, type_def::TypeDef};

pub struct FlagsInput {
    pub type_def: TypeDef,
    pub config: OverrideBlock,
    pub consts: ConstBlock,
}

impl FlagsInput {
    pub fn type_name(&self) -> &Ident {
        &self.type_def.type_name
    }
}

impl Parse for FlagsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_def = input.parse::<TypeDef>()?;
        let config = input.parse::<OverrideBlock>()?;
        let consts = input.parse::<ConstBlock>()?;
        Ok(Self {
            type_def,
            config,
            consts,
        })
    }
}

impl ToTokens for FlagsInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mask_ty = &self.type_def.mask_type;
        let vexillo = crate::get_vexillo_name();
        tokens.extend(quote!(
            #vexillo :: mask_type_check!{#mask_ty}
        ));
    }
}