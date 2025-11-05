use std::collections::HashMap;

use syn::{Error, Ident, Token, braced, parse::Parse, visit_mut::VisitMut};

use crate::vis::Vis;

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

pub struct OverrideItem {
    pub vis: Vis,
    pub item_ident: Ident,
    pub new_ident: Option<Ident>,
}

pub struct OverrideBlock {
    pub items: HashMap<Ident, OverrideItem>,
}

impl OverrideBlock {
    pub fn new_init() -> Self {
        macro_rules! items {
            ($($vis:vis $name:ident)*) => {
                HashMap::from([
                    $(
                        {
                            let ident: Ident = syn::parse_quote!($name);
                            (
                                ident.clone(),
                                OverrideItem {
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
                pub to_be_bytes
                pub to_le_bytes
                pub from_be_bytes
                pub from_le_bytes
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
    fn insert(&mut self, key: Ident, item: OverrideItem) -> Option<OverrideItem> {
        self.items.insert(key, item)
    }
    
    pub fn get_alt(&self, key: &Ident) -> Option<&Ident> {
        if let Some(item) = self.items.get(key) {
            item.new_ident.as_ref()
        } else {
            None
        }
    }
}

impl Parse for OverrideItem {
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

impl Parse for OverrideBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = OverrideBlock::new_init();
        if !input.peek(Token![override]) {
            return Ok(items);
        }
        input.parse::<Token![override]>()?;
        let block;
        braced!(block in input);
        while !block.is_empty() {
            let item = block.parse::<OverrideItem>()?;
            let key = item.item_ident.clone();
            let ident_span = item.item_ident.span();
            if items.insert(key, item).is_none() {
                return Err(Error::new(ident_span, "Unexpected override item."));
            }
        }
        Ok(items)
    }
}

impl VisitMut for OverrideBlock {
    fn visit_item_fn_mut(&mut self, i: &mut syn::ItemFn) {
        if let Some(item) = self.items.get(&i.sig.ident) {
            let vis = item.vis.resolve(Some(&i.vis));
            i.vis = vis;
            if let Some(alt) = &item.new_ident {
                i.sig.ident = alt.clone();
            }
        }
        syn::visit_mut::visit_item_fn_mut(self, i);
    }
    
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.segments.len() < 2 {
            return;
        }
        let first = &i.segments[0].ident;
        if first != "Self" {
            return;
        }
        let second = &mut i.segments[1].ident;
        if let Some(new_ident) = self.get_alt(second) {
            *second = new_ident.clone();
        }
        syn::visit_mut::visit_path_mut(self, i);
    }
    
    fn visit_expr_method_call_mut(&mut self, i: &mut syn::ExprMethodCall) {
        if let syn::Expr::Path(exp) = &mut *i.receiver {
            if exp.path.segments.len() < 2 {
                return;
            }
            let first = &exp.path.segments[0].ident;
            if first != "self" {
                return;
            }
            let second = &mut exp.path.segments[1].ident;
            if let Some(new_ident) = self.get_alt(second) {
                *second = new_ident.clone();
            }
        }
        syn::visit_mut::visit_expr_method_call_mut(self, i);
    }
}