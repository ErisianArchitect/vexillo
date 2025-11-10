use std::{
    collections::HashSet,
    sync::LazyLock,
};
use proc_macro2::Span;
use syn::{Attribute, Ident, spanned::Spanned};

pub mod bitmask;
pub mod const_block;
pub mod flags_input;
pub mod override_block;
pub mod type_def;
pub mod vis;

pub const FLAG_CFG_ERR_MSG: &'static str = "`cfg` attribute is error prone and is not allowed.\nInstead, use `cfg` on the macro call itself.\n\nDenying `cfg` attributes keeps the flags consistent across versions.\nAttempts to circumvent this error is likely to result in undesireable consequences.";

pub fn verify_no_cfg<'a, It: IntoIterator<Item = &'a Attribute>, M: std::fmt::Display>(attrs: It, message: M) -> syn::Result<()> {
    attrs.into_iter().try_for_each(move |attr| {
        if attr.meta.path().is_ident("cfg") {
            return Err(syn::Error::new(
                attr.span(),
                &message
            ));
        }
        Ok(())
    })
}