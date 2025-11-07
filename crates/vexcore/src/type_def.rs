use syn::{Attribute, Ident, Token, Type, Visibility, bracketed, parenthesized, parse::Parse, token::Paren};
use vexmacro::*;

fn default_mask_type() -> Type {
    syn::parse_quote!(u32)
}

pub struct TypeDef {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub type_name: Ident,
    pub mask_vis: Visibility,
    pub mask_type: Type,
}

impl Parse for TypeDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        crate::verify_no_cfg(&attrs, "`cfg` attribute is error prone and is not allowed.\nInstead, use `cfg` on the macro call itself.\nAttempts to circumvent this error is likely to result in undesireable consequences.")?;
        let vis = input.parse()?;
        _ = input.parse::<Token![struct]>()?;
        let type_name = input.parse()?;
        let (mask_vis, mask_type) = if input.peek(Paren) {
            let inner;
            parenthesized!(inner in input);
            let semi_result = input.parse::<Token![;]>();
            let mask_vis = inner.parse()?;
            (
                mask_vis,
                {
                    let sliced;
                    bracketed!(sliced in inner);
                    let mask_type = sliced.parse()?;
                    let first_result = ensure_eof!(sliced);
                    let second_result = ensure_eof!(inner);
                    combine_results!(first_result, second_result, semi_result)?;
                    mask_type
                }
            )
        } else {
            input.parse::<Token![;]>()?;
            (
                Visibility::Inherited,
                default_mask_type(),
            )
        };
        Ok(Self {
            attrs,
            vis,
            type_name,
            mask_vis,
            mask_type,
        })
    }
}