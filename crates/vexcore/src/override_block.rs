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
                // pub const fn new() -> Self
                pub new
                // pub const fn none() -> Self
                pub none
                // pub const fn all() -> Self
                pub all
                // pub const fn union(flags: &[Self]) -> Self
                pub union
                // pub const fn union_without(with: &[Self], without: &[Self])
                pub union_without
                // pub const fn try_find(&str) -> Option<Self>
                pub try_find
                // pub const fn find(&str) -> Self
                pub find
                // pub const fn find_or(&str, default: Self) -> Self
                pub find_or
                // pub const fn find_or_none(&str) -> Self
                pub find_or_none
                // pub const fn count_ones(self) -> u32
                pub count_ones
                // pub const fn count_zeros(self) -> u32
                pub count_zeros
                // pub const fn get(self, index: u32) -> bool
                pub get
                // pub const fn set(&mut self, index: u32, on: bool) -> &mut Self
                pub set
                // pub const fn swap(&mut self, index: u32, on: bool) -> bool
                pub swap
                // pub const fn from_index(index: u32) -> Self
                pub from_index
                // pub const fn add(&mut self, add: Self) -> &mut Self
                pub add
                // pub const fn add_all(&mut self, flags: &[Self]) -> &mut Self
                pub add_all
                // pub const fn remove(&mut self, remove: Self) -> &mut Self
                pub remove
                // pub const fn remove_all(&mut self, flags: &[Self]) -> &mut Self
                pub remove_all
                // pub const fn with(self, flag: Self) -> Self
                pub with
                // pub const fn with_all(self, flags: &[Self]) -> Self
                pub with_all
                // pub const fn without(self, flag: Self) -> Self
                pub without
                // pub const fn without_all(self, flags: &[Self]) -> Self
                pub without_all
                // pub const fn has_all(self, other: Self) -> bool
                pub has_all
                // pub const fn has_none(self, other: Self) -> bool
                pub has_none
                // pub const fn has_any(self, other: Self) -> bool
                pub has_any
                // pub const fn as_slice(&self) -> &[MaskTy]
                pub as_slice
                // pub const fn as_mut_slice(&mut self) -> &mut [MaskTy]
                pub as_mut_slice
                // pub const fn into_inner(self) -> [MaskTy; MaskCount]
                pub into_inner
                // pub const fn as_bytes(&self) -> &[u8]
                pub as_bytes
                // pub const fn as_mut_bytes(&mut self) -> &mut [u8]
                pub as_mut_bytes
                // pub const fn to_be_bytes(self) -> [u8; size_of::<Self>()]
                pub to_be_bytes
                // pub const fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self
                pub from_be_bytes
                // pub const fn to_le_bytes(self) -> [u8; size_of::<Self>()]
                pub to_le_bytes
                // pub const fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self
                pub from_le_bytes
                // pub const fn to_ne_bytes(self) -> [u8; size_of::<Self>()]
                pub to_ne_bytes
                // pub const fn from_ne_bytes(bytes: [u8; size_of::<Self>()]) -> Self
                pub from_ne_bytes
                // pub const fn not_assign(&mut self)
                pub not_assign
                // pub const fn not(self) -> Self
                pub not
                // pub const fn and_assign(&mut self, rhs: Self)
                pub and_assign
                // pub const fn and(self, rhs: Self) -> Self
                pub and
                // pub const fn or_assign(&mut self, rhs: Self)
                pub or_assign
                // pub const fn or(self, rhs: Self) -> Self
                pub or
                // pub const fn xor_assign(&mut self, rhs: Self)
                pub xor_assign
                // pub const fn xor(self, rhs: Self) -> Self
                pub xor
                // pub const fn nand_assign(&mut self, rhs: Self)
                pub nand_assign
                // pub const fn nand(self, rhs: Self) -> Self
                pub nand
                // pub const fn nor_assign(&mut self, rhs: Self)
                pub nor_assign
                // pub const fn nor(self, rhs: Self) -> Self
                pub nor
                // pub const fn xnor_assign(&mut self, rhs: Self)
                pub xnor_assign
                // pub const fn xnor(self, rhs: Self) -> Self
                pub xnor
                // pub const fn imply_assign(&mut self, rhs: Self)
                pub imply_assign
                // pub const fn imply(self, rhs: Self) -> Self
                pub imply
                // pub const fn nimply_assign(&mut self, rhs: Self)
                pub nimply_assign
                // pub const fn nimply(self, rhs: Self) -> Self
                pub nimply
                // pub const fn eq(self, rhs: Self) -> bool
                pub eq
                // pub const fn ne(self, rhs: Self) -> bool
                pub ne
            ]
        }
    }
    
    #[inline(always)]
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
    
    pub fn get_vis(&self, key: &Ident) -> &Vis {
        if let Some(item) = self.items.get(key) {
            &item.vis
        } else {
            panic!("Identifier not found.");
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

// This is meant to be used on the associated functions impl block.
// Be careful of its usage, because it only mutates paths that start
// with `Self` or `self`.
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
            let guard = const { ["self", "builder"] }
                .into_iter()
                .all(|name| {
                    first != name
                });
            if guard {
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