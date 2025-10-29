use std::collections::HashMap;

use syn::{Error, Ident, Token, braced, parse::Parse};
use proc_macro::TokenTree;

use crate::flags::vis::Vis;

/*
pub new
pub none
pub all
pub union
pub union_without
pub try_find
pub find
pub find_or
pub find_or_none
pub count_ones
pub count_zeros
pub add
pub add_all
pub remove
pub remove_all
pub with
pub with_all
pub without
pub without_all
pub get
pub set
pub from_index
pub has_all
pub has_none
pub has_any
pub masks
pub masks_mut
pub into_inner
pub as_bytes
pub as_mut_bytes
pub not
pub and
pub or
pub nand
pub nor
pub xor
pub xnor
pub imply
pub nimply
pub eq
pub ne
*/

pub struct ImplItem {
    pub vis: Vis,
    pub item_ident: Ident,
    pub new_ident: Option<Ident>,
}

pub struct ImplBlock {
    pub items: HashMap<Ident, ImplItem>,
}

impl ImplBlock {
    pub fn new_init() -> Self {
        macro_rules! items {
            ($($vis:vis $name:ident)*) => {
                HashMap::from([
                    $(
                        {
                            let ident: Ident = syn::parse_quote!($name);
                            (
                                ident.clone(),
                                ImplItem {
                                    vis: syn::parse_quote!($vis),
                                    item_ident: ident,
                                    new_ident: None,
                                }
                            )
                        },
                    )*
                ])
            };
        }
        Self {
            items: items![
                pub new
                pub none
                pub all
                pub union
                pub union_without
                pub try_find
                pub find
                pub find_or
                pub find_or_none
                pub count_ones
                pub count_zeros
                pub add
                pub add_all
                pub remove
                pub remove_all
                pub with
                pub with_all
                pub without
                pub without_all
                pub get
                pub set
                pub from_index
                pub has_all
                pub has_none
                pub has_any
                pub masks
                pub masks_mut
                pub into_inner
                pub as_bytes
                pub as_mut_bytes
                pub not
                pub and
                pub or
                pub nand
                pub nor
                pub xor
                pub xnor
                pub imply
                pub nimply
                pub eq
                pub ne
            ]
        }
    }
    
    #[inline]
    fn insert(&mut self, key: Ident, item: ImplItem) -> Option<ImplItem> {
        self.items.insert(key, item)
    }
    
    pub fn get_ident(&self, key: &Ident) -> Option<&Ident> {
        if let Some(item) = self.items.get(key) {
            Some(if let Some(alt) = &item.new_ident {
                alt
            } else {
                &item.item_ident
            })
        } else {
            None
        }
    }
}

impl Parse for ImplItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            vis: input.parse()?,
            item_ident: input.parse()?,
            new_ident: {
                if input.peek(Token![:]) {
                    input.parse::<Token![:]>()?;
                    Some(input.parse()?)
                } else {
                    None
                }
            }
        })
    }
}

impl Parse for ImplBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = ImplBlock::new_init();
        if !input.peek(Token![impl]) {
            return Ok(items);
        }
        input.parse::<Token![impl]>()?;
        let block;
        braced!(block in input);
        while !block.is_empty() {
            let item = block.parse::<ImplItem>()?;
            let key = item.item_ident.clone();
            let ident_span = item.item_ident.span();
            if items.insert(key, item).is_none() {
                return Err(Error::new(ident_span, "Unexpected impl item."));
            }
        }
        Ok(items)
    }
}