use std::collections::HashSet;

use quote::{quote, ToTokens};
use syn::{Ident, Path, Token, parse::Parse, visit_mut::VisitMut};

use crate::{const_block::{ConstBlock, ConstBuildResult}, override_block::{OverrideBlock, OverrideStage, Overrider}, type_def::TypeDef};

pub struct FlagsInput {
    pub vexillo_crate: Path,
    pub type_def: TypeDef,
    pub config: OverrideBlock,
    pub consts: ConstBuildResult,
}

impl FlagsInput {
    pub fn type_name(&self) -> &Ident {
        &self.type_def.type_name
    }
}

impl Parse for FlagsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        _=input.parse::<Token![use]>()?;
        let vexillo_crate = input.parse()?;
        _=input.parse::<Token![;]>()?;
        let type_def = input.parse::<TypeDef>()?;
        let config = input.parse::<OverrideBlock>()?;
        let consts = input.parse::<ConstBlock>()?.build();
        if (consts.singles.len() + consts.groups.len()) > 65536 {
            return Err(
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Exceeded max flag count of 65536. This limit is for your own sanity."
                )
            );
        }
        Ok(Self {
            vexillo_crate,
            type_def,
            config,
            consts,
        })
    }
}

impl ToTokens for FlagsInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let TypeDef {
            attrs: type_attrs,
            vis: type_vis,
            type_name,
            mask_vis,
            mask_type,
        } = &self.type_def;
        let single_flag_count = self.consts.singles.len();
        let group_flag_count = self.consts.groups.len();
        let add_fn = syn::parse_quote!(add);
        let config = &self.config;
        let add_fn = config.get_alt(&add_fn).unwrap_or(&add_fn);
        let all_builder = self.consts.singles.iter()
            .map(|single| {
                let ident = &single.ident;
                quote!(
                    builder.#add_fn(#type_name::#ident);
                )
            }).collect::<proc_macro2::TokenStream>();
        let flag_consts = self.consts.tokenize(&config);
        let builtin_consts = quote!{
            // ################################
            // #          CONSTANTS           #
            // ################################
            pub const SINGLE_FLAG_COUNT: usize = #single_flag_count;
            pub const GROUP_FLAG_COUNT: usize = #group_flag_count;
            pub const TOTAL_FLAG_COUNT: usize = #type_name::SINGLE_FLAG_COUNT + #type_name::GROUP_FLAG_COUNT;
            pub const BITS: u32 = (#type_name::SINGLE_FLAG_COUNT as u32).next_multiple_of(#type_name::MASK_BITS);
            pub const UNUSED_BITS: u32 = (#type_name::BITS - #type_name::SINGLE_FLAG_COUNT as u32);
            pub const USED_BITS: u32 = (#type_name::BITS - #type_name::UNUSED_BITS);
            pub const MASK_BITS: u32 = #mask_type::BITS;
            pub const MASK_SIZE: usize = ::core::mem::size_of::<#mask_type>();
            pub const MASK_COUNT: usize = {
                let mask_bits = #type_name::MASK_BITS as usize;
                let mask_bits_sub1 = mask_bits - 1;
                (#type_name::SINGLE_FLAG_COUNT + mask_bits_sub1) / mask_bits
            };
            const LAST_MASK_INDEX: usize = #type_name::MASK_COUNT - 1;
            pub const NONE: #type_name = #type_name { masks: [0; #type_name::MASK_COUNT] };
            pub const ALL: #type_name = {
                let mut builder = #type_name::new();
                #all_builder
                builder
            };
        };
        let functions_impl_block = build_builtin_functions(self);
        let op_impls = build_op_impls(self);
        let vexillo = &self.vexillo_crate;
        tokens.extend(quote!(
            #vexillo::mask_type_check!{#mask_type}
            // ################################
            // #       TYPE DEFINITION        #
            // ################################
            #(#type_attrs)*
            #[repr(transparent)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #type_vis struct #type_name {
                #mask_vis masks: [#mask_type; #type_name::MASK_COUNT],
            }
            impl #type_name {
                #builtin_consts
                #flag_consts
            }
            
            #functions_impl_block
            
            #op_impls
        ));
    }
}

// [tag]: build table
// ###############################
// #         BUILD TABLE         #
// ###############################
fn build_flag_table(input: &FlagsInput) -> proc_macro2::TokenStream {
    let const_build = &input.consts;
    let mut all_flag_names = Vec::with_capacity(
        const_build.singles.len() + const_build.groups.len(),
    );
    all_flag_names.extend(
        const_build.singles.iter().map(|item| &item.ident)
    );
    let groups_start = all_flag_names.len();
    all_flag_names.extend(
        const_build.groups.iter().map(|item| &item.ident)
    );
    let singles = &all_flag_names[0..groups_start];
    let groups = &all_flag_names[groups_start..];
    
    let table_len = singles.len() + groups.len();
    let single_count = singles.len();
    let groups_count = groups.len();
    
    
    
    
    // First, we need a table.
    
    todo!()
}

macro_rules! docstr {
    ($doc:literal $(, $($($name:ident = )? $arg:expr),*)?$(,)?) => {
        {
            let doc_string = format!($doc $(, $($($name = )? $arg),*)?);
            quote!(
                #[doc = #doc_string]
            )
        }
    };
}

fn make_func(
    config: &OverrideBlock,
    input: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let src = input.to_string();
    let err_msg = format!("Failed:\n{src}");
    let mut item = syn::parse::<syn::ItemFn>(input.into()).expect(&err_msg);
    
    let name = &item.sig.ident;
    let item_cfg = &config.items[name];
    
    item.sig.ident = item_cfg.new_ident
        .as_ref()
        .unwrap_or(&item_cfg.item_ident)
        .clone();
    item.vis = item_cfg.vis.resolve(None);
    
    quote!(
        #item
    )
}

fn build_builtin_functions(input: &FlagsInput) -> syn::File {
    let mut functions = Vec::<proc_macro2::TokenStream>::with_capacity(64);
    let config = &input.config;
    macro_rules! func {
        (
            #[doc($doc:literal $(, $( $($arg_name:ident = )? $arg:expr),*)?$(,)?)]
            $($tokens:tt)*
        ) => {
            {
                let doc = docstr!($doc $(, $( $($arg_name = )? $arg),*)?);
                let tokens = quote!($($tokens)*);
                let func = make_func(&config, tokens);
                functions.push(quote!(
                    #doc
                    #func
                ));
            }
        };
    }
    
    let type_name = input.type_name();
    let vexillo = &input.vexillo_crate;
    // ################################
    // #          FUNCTIONS           #
    // ################################
    func!( // new
        #[doc("Create a new [{type_name}] with none of the bits set.")]
        #[must_use]
        #[inline(always)]
        const fn new() -> Self {
            Self::NONE
        }
    );
    func!( // none
        #[doc("Create a new [{type_name}] with none of the bits set.")]
        #[must_use]
        #[inline(always)]
        const fn none() -> Self {
            Self::NONE
        }
    );
    func!( // all
        #[doc("Create a new [{type_name}] with all of the flags set.")]
        #[must_use]
        #[inline(always)]
        const fn all() -> Self {
            Self::ALL
        }
    );
    func!( // union
        #[doc("Create a union of all `flags`.")]
        #[must_use]
        #[inline]
        const fn union(flags: &[Self]) -> Self {
            let mut builder = Self::new();
            builder.add_all(flags);
            builder
        }
    );
    func!( // union_without
        #[doc("Create a union of all `flags` without all `removals`.")]
        #[must_use]
        #[inline]
        const fn union_without(flags: &[Self], removals: &[Self]) -> Self {
            let mut builder = Self::new();
            builder.add_all(flags);
            builder.remove_all(removals);
            builder
        }
    );
    // func!( // try_find
    //     #[doc("Try to find a flag by its name. Returns [None] if the flag was not found.")]
    //     #[must_use]
    //     const fn try_find(name: &str) -> Option<Self> {
    //         use std::cmp::Ordering::*;
    //         let mut lo = 0usize;
    //         // ORDERED_FLAGS = [(name, value)]
    //         let mut hi = Self::ORDERED_FLAGS.len();
    //         while lo < hi {
    //             let mid = ((hi - lo) / 2) + lo;
    //             let mid_name = Self::ORDERED_FLAGS[mid].0;
    //             match #vexillo::internal::const_cmp_str(name, mid_name) {
    //                 Less => hi = mid,
    //                 Equal => return Some(Self::ORDERED_FLAGS[mid].1),
    //                 Greater => lo = mid + 1,
    //             }
    //         }
    //         None
    //     }
    // );
    // func!( // find
    //     #[doc("Find a flag by its name. Panics if the flag was not found.")]
    //     #[must_use]
    //     #[track_caller]
    //     #[inline]
    //     const fn find(name: &str) -> Self {
    //         match Self::try_find(name) {
    //             Some(flag) => flag,
    //             _ => panic!("Flag by that name was not found."),
    //         }
    //     }
    // );
    // func!( // find_or
    //     #[doc("Find a flag by its name. Returns `default` if the flag was not found.")]
    //     #[must_use]
    //     #[inline]
    //     const fn find_or(name: &str, default: Self) -> Self {
    //         match Self::try_find(name) {
    //             Some(flag) => flag,
    //             _ => default,
    //         }
    //     }
    // );
    // func!( // find_or_none
    //     #[doc("Find a flag by its name. Returns [Self::NONE] if the flag was not found.")]
    //     #[must_use]
    //     #[inline]
    //     const fn find_or_none(name: &str) -> Self {
    //         match Self::try_find(name) {
    //             Some(flag) => flag,
    //             _ => Self::NONE,
    //         }
    //     }
    // );
    func!( // count_ones
        #[doc("Return the number of ones in the binary representation of `self`.")]
        #[must_use]
        const fn count_ones(self) -> u32 {
            let mut count = 0u32;
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = index.next() {
                count += self.masks[i].count_ones();
            }
            count
        }
    );
    func!( // count_zeros
        #[doc("Return the number of zeros in the binary representation of `self`.")]
        #[must_use]
        const fn count_zeros(self) -> u32 {
            let mut count = 0u32;
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = index.next() {
                count += self.masks[i].count_zeros();
            }
            count - Self::UNUSED_BITS
        }
    );
    func!( // get
        #[doc("Get the bit at `index`.")]
        #[must_use]
        #[inline]
        const fn get(self, index: u32) -> bool {
            let index = #vexillo::internal::MaskIndex::new(index, Self::MASK_BITS);
            self.masks[index.mask] & (1 << index.bit) != 0
        }
    );
    func!( // set
        #[doc("Set the bit at `index`.")]
        #[inline]
        const fn set(&mut self, index: u32, on: bool) -> &mut Self {
            let index = #vexillo::internal::MaskIndex::new(index, Self::MASK_BITS);
            if on {
                self.masks[index.mask] |= (1 << index.bit);
            } else {
                self.masks[index.mask] &= !(1 << index.bit);
            }
            self
        }
    );
    func!( // swap
        #[doc("Swap the bit at `index`.")]
        #[inline]
        const fn swap(&mut self, index: u32, on: bool) -> bool {
            let index = #vexillo::internal::MaskIndex::new(index, Self::MASK_BITS);
            let old = ((self.masks[index.mask] & (1 << index.bit)) != 0);
            if on {
                self.masks[index.mask] |= (1 << index.bit);
            } else {
                self.masks[index.mask] &= !(1 << index.bit);
            }
            old
        }
    );
    func!( // from_index
        #[doc("Create a [{type_name}] with the bit at the given `index` set to 1.")]
        #[inline]
        #[must_use]
        const fn from_index(index: u32) -> Self {
            let mut builder = Self::NONE;
            builder.set(index, true);
            builder
        }
    );
    func!( // leading_zeros
        #[doc("Count the number of leading zeros.")]
        #[must_use]
        const fn leading_zeros(self) -> u32 {
            // leading | trailing
            // 0b0000011110000000
            // UNUSED_BITS bitmask example (UNUSED_BITS is bit-count, not bitmask)
            // 0b11110000
            let valid_mask = self.masks[Self::LAST_MASK_INDEX] & Self::ALL.masks[Self::LAST_MASK_INDEX];
            let lead = valid_mask.leading_zeros();
            if lead < Self::MASK_BITS {
                // valid_mask was mask ANDed with ALL, removing the unused bits.
                return lead - Self::UNUSED_BITS;
            }
            let mut count = lead - Self::UNUSED_BITS;
            let mut index = Self::LAST_MASK_INDEX;
            while index != 0 {
                index -= 1;
                let lead = self.masks[index].leading_zeros();
                count += lead;
                if lead < Self::MASK_BITS {
                    return count;
                }
            }
            Self::USED_BITS
        }
    );
    func!( // leading_ones
        #[doc("Count the number of leading ones.")]
        #[must_use]
        const fn leading_ones(self) -> u32 {
            let with_unused = self.masks[Self::LAST_MASK_INDEX] | !Self::ALL.masks[Self::LAST_MASK_INDEX];
            let lead = with_unused.leading_ones();
            if lead < Self::MASK_BITS {
                return lead - Self::UNUSED_BITS;
            }
            let mut count = lead - Self::UNUSED_BITS;
            let mut index = Self::LAST_MASK_INDEX;
            while index != 0 {
                index -= 1;
                let lead = self.masks[index].leading_ones();
                count += lead;
                if lead < Self::MASK_BITS {
                    return count;
                }
            }
            
            Self::USED_BITS
        }
    );
    func!( // trailing_zeros
        #[doc("Count the number of trailing zeros.")]
        #[must_use]
        const fn trailing_zeros(self) -> u32 {
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            let mut count = 0u32;
            while let i @ 0..Self::MASK_COUNT = index.next() {
                let trailing = self.masks[i].trailing_zeros();
                count += trailing;
                if trailing < Self::MASK_BITS {
                    return if count > Self::USED_BITS {
                        Self::USED_BITS
                    } else {
                        count
                    };
                }
            }
            Self::USED_BITS
        }
    );
    func!( // trailing_ones
        #[doc("Count the number of trailing ones.")]
        #[must_use]
        const fn trailing_ones(self) -> u32 {
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            let mut count = 0u32;
            while let i @ 0..Self::MASK_COUNT = index.next() {
                let trailing = self.masks[i].trailing_ones();
                count += trailing;
                if trailing < Self::MASK_BITS {
                    return if count > Self::USED_BITS {
                        Self::USED_BITS
                    } else {
                        count
                    };
                }
            }
            Self::USED_BITS
        }
    );
    func!( // add
        #[doc("Add all of the bits present in `flag`.")]
        const fn add(&mut self, flag: Self) -> &mut Self {
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = index.next() {
                self.masks[i] |= flag.masks[i];
            }
            self
        }
    );
    func!( // add_if
        #[doc("Add all of the bits present in `flag` if `condition` is `true`.")]
        #[inline]
        const fn add_if(&mut self, flag: Self, condition: bool) -> &mut Self {
            if condition {
                self.add(flag);
            }
            self
        }
    );
    func!( // add_all
        #[doc("Add all of the bits present in all `flags`.")]
        const fn add_all(&mut self, flags: &[Self]) -> &mut Self {
            let mut flag_index = 0usize;
            while flag_index < flags.len() {
                self.add(flags[flag_index]);
                flag_index += 1;
            }
            self
        }
    );
    func!( // add_all_if
        #[doc("Add all of the bits present in all `flags` if `condition` is `true`.")]
        #[inline]
        const fn add_all_if(&mut self, flags: &[Self], condition: bool) -> &mut Self {
            if condition {
                self.add_all(flags);
            }
            self
        }
    );
    func!( // remove
        #[doc("Remove all of the bits present in `flag`.")]
        const fn remove(&mut self, flag: Self) -> &mut Self {
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = index.next() {
                self.masks[i] &= !flag.masks[i];
            }
            self
        }
    );
    func!( // remove_if
        #[doc("Remove all of the bits present in `flag` if `condition` is `true`.")]
        #[inline]
        const fn remove_if(&mut self, flag: Self, condition: bool) -> &mut Self {
            if condition {
                self.remove(flag);
            }
            self
        }
    );
    func!( // remove_all
        #[doc("Remove all of the bits present in all `flags`.")]
        const fn remove_all(&mut self, flags: &[Self]) -> &mut Self {
            let mut flag_index = 0usize;
            while flag_index < flags.len() {
                self.remove(flags[flag_index]);
                flag_index += 1;
            }
            self
        }
    );
    func!( // remove_all_if
        #[doc("Remove all of the bits present in all `flags` if `condition` is `true`.")]
        #[inline]
        const fn remove_all_if(&mut self, flags: &[Self], condition: bool) -> &mut Self {
            if condition {
                self.remove_all(flags);
            }
            self
        }
    );
    func!( // with
        #[doc("Join with `flag`.")]
        #[inline]
        #[must_use]
        const fn with(mut self, flag: Self) -> Self {
            self.add(flag);
            self
        }
    );
    func!( // with_if
        #[doc("Join with `flag` if `condition` is `true`.")]
        #[inline]
        #[must_use]
        const fn with_if(mut self, flag: Self, condition: bool) -> Self {
            self.add_if(flag, condition);
            self
        }
    );
    func!( // with_all
        #[doc("Join with all `flags`.")]
        #[inline]
        #[must_use]
        const fn with_all(mut self, flags: &[Self]) -> Self {
            self.add_all(flags);
            self
        }
    );
    func!( // with_all_if
        #[doc("Join with all `flags` if `condition` is `true`.")]
        #[inline]
        #[must_use]
        const fn with_all_if(mut self, flags: &[Self], condition: bool) -> Self {
            self.add_all_if(flags, condition);
            self
        }
    );
    func!( // without
        #[doc("Exclude `flag`.")]
        #[inline]
        #[must_use]
        const fn without(mut self, flag: Self) -> Self {
            self.remove(flag);
            self
        }
    );
    func!( // without_if
        #[doc("Exclude `flag` if `condition` is `true`.")]
        #[inline]
        #[must_use]
        const fn without_if(mut self, flag: Self, condition: bool) -> Self {
            self.remove_if(flag, condition);
            self
        }
    );
    func!( // without_all
        #[doc("Exclude all `flags`.")]
        #[inline]
        #[must_use]
        const fn without_all(mut self, flags: &[Self]) -> Self {
            self.remove_all(flags);
            self
        }
    );
    func!( // without_all_if
        #[doc("Exclude all `flags` if `condition` is `true`.")]
        #[inline]
        #[must_use]
        const fn without_all_if(mut self, flags: &[Self], condition: bool) -> Self {
            self.remove_all_if(flags, condition);
            self
        }
    );
    func!( // has_all
        #[doc("Test if all bits of `flag` are present in `self`.")]
        #[must_use]
        const fn has_all(self, flag: Self) -> bool {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                if self.masks[i] & flag.masks[i] != flag.masks[i] {
                    return false;
                }
            }
            true
        }
    );
    func!( // has_none
        #[doc("Test if none of the bits of `flag` are present in `self`.")]
        #[must_use]
        const fn has_none(self, flag: Self) -> bool {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                if self.masks[i] & flag.masks[i] != 0 {
                    return false;
                }
            }
            true
        }
    );
    func!( // has_any
        #[doc("Test if any of the bits of `flag` are present in `self`.")]
        #[must_use]
        const fn has_any(self, flag: Self) -> bool {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                if self.masks[i] & flag.masks[i] != 0 {
                    return true;
                }
            }
            false
        }
    );
    func!( // has_some
        #[doc("Test if some but not all of the bits of `flag` are present in `self`.")]
        #[must_use]
        const fn has_some(self, flag: Self) -> bool {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            let mut any_ne = false;
            let mut has_any = false;
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                any_ne |= self.masks[i] != flag.masks[i];
                has_any |= self.masks[i] & flag.masks[i] != 0;
            }
            has_any && any_ne
        }
    );
    let mask_ty = &input.type_def.mask_type;
    func!( // as_slice
        #[doc("Returns the inner `masks` as a slice.")]
        #[inline]
        #[must_use]
        const fn as_slice(&self) -> &[#mask_ty] {
            &self.masks
        }
    );
    func!( // as_mut_slice
        #[doc("Returns the inner `masks` as a mutable slice.")]
        #[inline]
        #[must_use]
        const fn as_mut_slice(&mut self) -> &mut [#mask_ty] {
            &mut self.masks
        }
    );
    func!( // into_inner
        #[doc("Decompose `self` into the inner `masks` array.")]
        #[inline]
        #[must_use]
        const fn into_inner(self) -> [#mask_ty; Self::MASK_COUNT] {
            self.masks
        }
    );
    func!( // as_bytes
        #[doc("Return `self` as a slice of bytes. Endianess is dependent on the target architecture. This is meant to be used for FFI. If you need serialization, use [{type_name}::to_be_bytes], [{type_name}::to_le_bytes], or [{type_name}::to_ne_bytes].")]
        #[inline]
        #[must_use]
        const fn as_bytes(&self) -> &[u8] {
            unsafe {
                ::core::slice::from_raw_parts(
                    &self.masks as *const _ as *const u8,
                    ::core::mem::size_of::<Self>(),
                )
            }
        }
    );
    
    func!( // as_mut_bytes
        #[doc("Return `self` as a mutable slice of bytes. Endianess is dependent on the target architecture. This is meant to be used for FFI. If you need serialization, use [{type_name}::to_be_bytes], [{type_name}::to_le_bytes], or [{type_name}::to_ne_bytes].")]
        #[inline]
        #[must_use]
        const fn as_mut_bytes(&mut self) -> &mut [u8] {
            unsafe {
                ::core::slice::from_raw_parts_mut(
                    &mut self.masks as *mut _ as *mut u8,
                    ::core::mem::size_of::<Self>(),
                )
            }
        }
    );
    func!( // to_be_bytes
        #[doc("Return `self` as Big-Endian bytes.")]
        #[must_use]
        const fn to_be_bytes(self) -> [u8; ::core::mem::size_of::<Self>()] {
            let mut bytes = [0u8; ::core::mem::size_of::<Self>()];
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let offset = i * Self::MASK_SIZE;
                let sub = #vexillo::internal::subslice_mut(&mut bytes, offset..offset+Self::MASK_SIZE);
                let mask_bytes = self.masks[i].to_be_bytes();
                sub.copy_from_slice(&mask_bytes);
            }
            bytes
        }
    );
    func!( // from_be_bytes
        #[doc("Create a new [{type_name}] from Big-Endian `bytes`.")]
        #[must_use]
        const fn from_be_bytes(bytes: [u8; ::core::mem::size_of::<Self>()]) -> Self {
            let mut masks = [0; Self::MASK_COUNT];
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let offset = i * Self::MASK_SIZE;
                let sub = #vexillo::internal::subslice(&bytes, offset..offset+Self::MASK_SIZE);
                let mut mask_bytes = [0u8; Self::MASK_SIZE];
                mask_bytes.copy_from_slice(&sub);
                masks[i] = #mask_ty::from_be_bytes(mask_bytes);
            }
            Self {
                masks,
            }
        }
    );
    func!( // to_le_bytes
        #[doc("Return `self` as Little-Endian bytes.")]
        #[must_use]
        const fn to_le_bytes(self) -> [u8; ::core::mem::size_of::<Self>()] {
            let mut bytes = [0u8; ::core::mem::size_of::<Self>()];
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let offset = i * Self::MASK_SIZE;
                let sub = #vexillo::internal::subslice_mut(&mut bytes, offset..offset+Self::MASK_SIZE);
                let mask_bytes = self.masks[i].to_le_bytes();
                sub.copy_from_slice(&mask_bytes);
            }
            bytes
        }
    );
    func!( // from_le_bytes
        #[doc("Create a new [{type_name}] from Little-Endian `bytes`.")]
        #[must_use]
        const fn from_le_bytes(bytes: [u8; ::core::mem::size_of::<Self>()]) -> Self {
            let mut masks = [0; Self::MASK_COUNT];
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let offset = i * Self::MASK_SIZE;
                let sub = #vexillo::internal::subslice(&bytes, offset..offset+Self::MASK_SIZE);
                let mut mask_bytes = [0u8; Self::MASK_SIZE];
                mask_bytes.copy_from_slice(&sub);
                masks[i] = #mask_ty::from_le_bytes(mask_bytes);
            }
            Self {
                masks,
            }
        }
    );
    func!( // to_ne_bytes
        #[doc("Return `self` as Native-Endian bytes.")]
        #[must_use]
        const fn to_ne_bytes(self) -> [u8; ::core::mem::size_of::<Self>()] {
            let mut bytes = [0u8; ::core::mem::size_of::<Self>()];
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let offset = i * Self::MASK_SIZE;
                let sub = #vexillo::internal::subslice_mut(&mut bytes, offset..offset+Self::MASK_SIZE);
                let mask_bytes = self.masks[i].to_ne_bytes();
                sub.copy_from_slice(&mask_bytes);
            }
            bytes
        }
    );
    func!( // from_ne_bytes
        #[doc("Create a new [{type_name}] from Native-Endian `bytes`.")]
        #[must_use]
        const fn from_ne_bytes(bytes: [u8; ::core::mem::size_of::<Self>()]) -> Self {
            let mut masks = [0; Self::MASK_COUNT];
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let offset = i * Self::MASK_SIZE;
                let sub = #vexillo::internal::subslice(&bytes, offset..offset+Self::MASK_SIZE);
                let mut mask_bytes = [0u8; Self::MASK_SIZE];
                mask_bytes.copy_from_slice(&sub);
                masks[i] = #mask_ty::from_ne_bytes(mask_bytes);
            }
            Self {
                masks,
            }
        }
    );
    func!( // decompose
        #[doc("Decompose bits into booleans.")]
        #[must_use]
        const fn decompose(self) -> [bool; Self::SINGLE_FLAG_COUNT] {
            let mut bools = [false; Self::SINGLE_FLAG_COUNT];
            todo!()
            bools
        }
    );
    func!( // not_assign
        #[doc("Bitwise NOT with assignment.")]
        const fn not_assign(&mut self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                self.masks[i] = !self.masks[i];
            }
            // Ensure that the unused bits are not set.
            self.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            self
        }
    );
    func!( // not
        #[doc("Bitwise NOT.")]
        #[inline]
        #[must_use]
        const fn not(mut self) -> Self {
            self.not_assign();
            self
        }
    );
    func!( // and_assign
        #[doc("Bitwise AND with assignment.")]
        const fn and_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                self.masks[i] &= other.masks[i];
            }
            self
        }
    );
    func!( // and
        #[doc("Bitwise AND.")]
        #[inline]
        #[must_use]
        const fn and(mut self, other: Self) -> Self {
            self.and_assign(other);
            self
        }
    );
    func!( // or_assign
        #[doc("Bitwise OR with assignment.")]
        const fn or_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                self.masks[i] |= other.masks[i];
            }
            self
        }
    );
    func!( // or
        #[doc("Bitwise OR.")]
        #[inline]
        #[must_use]
        const fn or(mut self, other: Self) -> Self {
            self.or_assign(other);
            self
        }
    );
    func!( // xor_assign
        #[doc("Bitwise XOR with assignment")]
        const fn xor_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                self.masks[i] ^= other.masks[i];
            }
            self
        }
    );
    func!( // xor
        #[doc("Bitwise XOR.")]
        #[inline]
        #[must_use]
        const fn xor(mut self, other: Self) -> Self {
            self.xor_assign(other);
            self
        }
    );
    func!( // nand_assign
        #[doc("Bitwise NAND with assignment.")]
        const fn nand_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                self.masks[i] = !(self.masks[i] & other.masks[i]);
            }
            self.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            self
        }
    );
    func!( // nand
        #[doc("Bitwise NAND.")]
        #[inline]
        #[must_use]
        const fn nand(mut self, other: Self) -> Self {
            self.nand_assign(other);
            self
        }
    );
    func!( // nor_assign
        #[doc("Bitwise NOR with assignment.")]
        const fn nor_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                self.masks[i] = !self.masks[i] & !other.masks[i];
            }
            self.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            self
        }
    );
    func!( // nor
        #[doc("Bitwise NOR.")]
        #[inline]
        #[must_use]
        const fn nor(mut self, other: Self) -> Self {
            self.nor_assign(other);
            self
        }
    );
    func!( // xnor_assign
        #[doc("Bitwise XNOR with assignment.")]
        const fn xnor_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                self.masks[i] = !(self.masks[i] ^ other.masks[i]);
            }
            self
        }
    );
    func!( // xnor
        #[doc("Bitwise XNOR.")]
        #[inline]
        #[must_use]
        const fn xnor(mut self, other: Self) -> Self {
            self.xnor_assign(other);
            self
        }
    );
    func!( // imply_assign
        #[doc("Bitwise IMPLY with assignment.")]
        const fn imply_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let a = self.masks[i];
                let b = other.masks[i];
                self.masks[i] = !a | b;
            }
            self.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            self
        }
    );
    func!( // imply
        #[doc("Bitwise IMPLY.")]
        #[inline]
        #[must_use]
        const fn imply(mut self, other: Self) -> Self {
            self.imply_assign(other);
            self
        }
    );
    func!( // nimply_assign
        #[doc("Bitwise NIMPLY with assignment.")]
        const fn nimply_assign(&mut self, other: Self) -> &mut Self {
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let a = self.masks[i];
                let b = other.masks[i];
                self.masks[i] = a & !b;
            }
            self
        }
    );
    func!( // nimply
        #[doc("Bitwise NIMPLY.")]
        #[inline]
        #[must_use]
        const fn nimply(mut self, other: Self) -> Self {
            self.nimply_assign(other);
            self
        }
    );
    func!( // eq
        #[doc("Test for equality.")]
        #[must_use]
        const fn eq(self, other: Self) -> bool {
            let mut counter = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = counter.next() {
                if self.masks[i] != other.masks[i] {
                    return false;
                }
            }
            true
        }
    );
    func!( // ne
        #[doc("Test for inequality.")]
        #[must_use]
        const fn ne(self, other: Self) -> bool {
            let mut counter = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = counter.next() {
                if self.masks[i] != other.masks[i] {
                    return true;
                }
            }
            false
        }
    );
    func!( // is_empty
        #[doc("Test if there are no flags present.")]
        #[inline]
        #[must_use]
        const fn is_empty(self) -> bool {
            self.eq(Self::NONE)
        }
    );
    func!( // is_not_empty
        #[doc("Test if there any flags present.")]
        #[inline]
        #[must_use]
        const fn is_not_empty(self) -> bool {
            self.ne(Self::NONE)
        }
    );
    func!( // len
        #[doc("Alias for `count_ones`.")]
        #[inline]
        #[must_use]
        const fn len(self) -> usize {
            self.count_ones() as _
        }
    );
    func!( // is_valid
        #[doc("Test if this is a valid bitset. That is, none of the unused bits are set.")]
        #[inline]
        #[must_use]
        const fn is_valid(self) -> bool {
            self.masks[Self::LAST_MASK_INDEX] & !Self::ALL.masks[Self::LAST_MASK_INDEX] == 0
        }
    );
    let inner = functions.into_iter().collect::<proc_macro2::TokenStream>();
    let type_name = input.type_name();
    let mut impl_block: syn::File = syn::parse_quote!(
        impl #type_name {
            #inner
        }
    );
    let mut overrider = Overrider {
        overrides: config,
        stage: OverrideStage::Functions,
    };
    overrider.visit_file_mut(&mut impl_block);
    impl_block
}

fn build_op_impls(input: &FlagsInput) -> syn::File {
    let ty = input.type_name();
    /*
    Not,
    BitAnd, BitAndAssign,
    BitOr, BitOrAssign,
    BitXor, BitXorAssign,
    Add, AddAssign,
    Sub, SubAssign,
    Index<u32, Output = bool>
    Index<usize, Output = bool>
    */
    let mut op_impls: syn::File = syn::parse_quote!(
        impl ::core::ops::Not for #ty {
            type Output = Self;
            #[inline(always)]
            fn not(self) -> Self {
                self.not()
            }
        }
        
        impl ::core::ops::BitAnd<Self> for #ty {
            type Output = Self;
            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self {
                self.and(rhs)
            }
        }
        
        impl ::core::ops::BitAndAssign<Self> for #ty {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = self.and(rhs);
            }
        }
        
        impl ::core::ops::BitOr<Self> for #ty {
            type Output = Self;
            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self {
                self.or(rhs)
            }
        }
        
        impl ::core::ops::BitOrAssign<Self> for #ty {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = self.or(rhs);
            }
        }
        
        impl ::core::ops::BitXor<Self> for #ty {
            type Output = Self;
            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self {
                self.xor(rhs)
            }
        }
        
        impl ::core::ops::BitXorAssign<Self> for #ty {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = self.xor(rhs);
            }
        }
        
        impl ::core::ops::Add<Self> for #ty {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                self.with(rhs)
            }
        }
        
        impl ::core::ops::AddAssign<Self> for #ty {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.add(rhs);
            }
        }
        
        impl ::core::ops::Sub<Self> for #ty {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                self.without(rhs)
            }
        }
        
        impl ::core::ops::SubAssign<Self> for #ty {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.remove(rhs);
            }
        }
        
        impl ::core::ops::Index<u32> for #ty {
            type Output = bool;
            #[inline(always)]
            fn index(&self, index: u32) -> &bool {
                debug_assert!((index as usize) < Self::SINGLE_FLAG_COUNT, "Index out of bounds.");
                const BOOLS: [bool; 2] = [false, true];
                &BOOLS[self.get(index) as usize]
            }
        }
        
        impl ::core::ops::Index<usize> for #ty {
            type Output = bool;
            #[inline(always)]
            fn index(&self, index: usize) -> &bool {
                debug_assert!(index < Self::SINGLE_FLAG_COUNT, "Index out of bounds.");
                const BOOLS: [bool; 2] = [false, true];
                &BOOLS[self.get(index as u32) as usize]
            }
        }
    );
    let mut overrider = Overrider {
        overrides: &input.config,
        stage: OverrideStage::Operators,
    };
    overrider.visit_file_mut(&mut op_impls);
    op_impls
}