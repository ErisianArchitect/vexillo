use std::{sync::Mutex};

use quote::{quote, ToTokens};
use syn::{Ident, Path, Token, parse::Parse, visit_mut::VisitMut};

use crate::{const_block::{ConstBlock, ConstBuildResult}, override_block::OverrideBlock, type_def::TypeDef};

pub struct FlagsInput {
    pub vexillo_crate: Path,
    pub type_def: TypeDef,
    pub config: Mutex<OverrideBlock>,
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
            config: Mutex::new(config),
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
        let config = self.config.lock().unwrap();
        let add_fn = config.get_alt(&syn::parse_quote!(add)).unwrap_or(&add_fn);
        let all_builder = self.consts.singles.iter()
            .map(|single| {
                let ident = &single.ident;
                quote!(
                    builder.#add_fn(#type_name::#ident);
                )
            }).collect::<proc_macro2::TokenStream>();
        let flag_consts = self.consts.tokenize(&config);
        drop(config);
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

// [tag]: build arrays
// ################################
// #         BUILD ARRAYS         #
// ################################
fn build_const_arrays(const_build: &ConstBuildResult) -> proc_macro2::TokenStream {
    let mut all_flag_names = Vec::with_capacity(
        const_build.singles.len() + const_build.groups.len(),
    );
    all_flag_names.extend(
        const_build.singles.iter().map(|item| &item.ident)
    );
    let singles = all_flag_names.as_slice();
    let groups_start = all_flag_names.len();
    all_flag_names.extend(
        const_build.groups.iter().map(|item| &item.ident)
    );
    let groups = &all_flag_names[groups_start..];
    
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

fn build_builtin_functions(input: &FlagsInput) -> proc_macro2::TokenStream {
    let mut functions = Vec::<proc_macro2::TokenStream>::with_capacity(64);
    let mut config = input.config.lock().unwrap();
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
        #[inline]
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
        #[inline]
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
    func!( // add
        #[doc("Add all of the bits present in `flag`.")]
        #[inline]
        const fn add(&mut self, flag: Self) -> &mut Self {
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = index.next() {
                self.masks[i] |= flag.masks[i];
            }
            self
        }
    );
    func!( // add_all
        #[doc("Add all of the bits present in all `flags`.")]
        #[inline]
        const fn add_all(&mut self, flags: &[Self]) -> &mut Self {
            let mut flag_index = 0usize;
            while flag_index < flags.len() {
                self.add(flags[flag_index]);
                flag_index += 1;
            }
            self
        }
    );
    func!( // remove
        #[doc("Remove all of the bits present in `flag`.")]
        #[inline]
        const fn remove(&mut self, flag: Self) -> &mut Self {
            let mut index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = index.next() {
                self.masks[i] &= !flag.masks[i];
            }
            self
        }
    );
    func!( // remove_all
        #[doc("Remove all of the bits present in all `flags`.")]
        #[inline]
        const fn remove_all(&mut self, flags: &[Self]) -> &mut Self {
            let mut flag_index = 0usize;
            while flag_index < flags.len() {
                self.remove(flags[flag_index]);
                flag_index += 1;
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
    func!( // with_all
        #[doc("Join with all `flags`.")]
        #[inline]
        #[must_use]
        const fn with_all(mut self, flags: &[Self]) -> Self {
            self.add_all(flags);
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
    func!( // without_all
        #[doc("Exclude all `flags`.")]
        #[inline]
        #[must_use]
        const fn without_all(mut self, flags: &[Self]) -> Self {
            self.remove_all(flags);
            self
        }
    );
    func!( // has_all
        #[doc("Test if all bits of `flag` are present in `self`.")]
        #[inline]
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
        #[inline]
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
        #[inline]
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
        #[inline]
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
        #[inline]
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
        #[inline]
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
        #[inline]
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
        #[inline]
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
        #[inline]
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
    func!( // not
        #[doc("Bitwise NOT.")]
        #[inline]
        #[must_use]
        const fn not(self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                tmp.masks[i] = !tmp.masks[i];
            }
            // Ensure that the unused bits are not set.
            tmp.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            tmp
        }
    );
    func!( // and
        #[doc("Bitwise AND.")]
        #[inline]
        #[must_use]
        const fn and(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                tmp.masks[i] &= other.masks[i];
            }
            tmp
        }
    );
    func!( // or
        #[doc("Bitwise OR.")]
        #[inline]
        #[must_use]
        const fn or(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                tmp.masks[i] |= other.masks[i];
            }
            tmp
        }
    );
    func!( // xor
        #[doc("Bitwise XOR.")]
        #[inline]
        #[must_use]
        const fn xor(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                tmp.masks[i] ^= other.masks[i];
            }
            tmp
        }
    );
    func!( // nand
        #[doc("Bitwise NAND.")]
        #[inline]
        #[must_use]
        const fn nand(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                tmp.masks[i] = !(tmp.masks[i] & other.masks[i]);
            }
            tmp.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            tmp
        }
    );
    func!( // nor
        #[doc("Bitwise NOR.")]
        #[inline]
        #[must_use]
        const fn nor(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                tmp.masks[i] = !tmp.masks[i] & !other.masks[i];
            }
            tmp.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            tmp
        }
    );
    func!( // xnor
        #[doc("Bitwise XNOR.")]
        #[inline]
        #[must_use]
        const fn xnor(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                tmp.masks[i] = !(tmp.masks[i] ^ other.masks[i]);
            }
            tmp
        }
    );
    func!( // imply
        #[doc("Bitwise IMPLY.")]
        #[inline]
        #[must_use]
        const fn imply(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let a = tmp.masks[i];
                let b = other.masks[i];
                tmp.masks[i] = !a | b;
            }
            tmp.masks[Self::LAST_MASK_INDEX] &= Self::ALL.masks[Self::LAST_MASK_INDEX];
            tmp
        }
    );
    func!( // nimply
        #[doc("Bitwise NIMPLY.")]
        #[inline]
        #[must_use]
        const fn nimply(self, other: Self) -> Self {
            let mut tmp = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                let a = tmp.masks[i];
                let b = other.masks[i];
                tmp.masks[i] = a & !b;
            }
            tmp
        }
    );
    func!( // eq
        #[doc("Test for equality.")]
        #[inline]
        #[must_use]
        const fn eq(self, other: Self) -> bool {
            let me = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                if me.masks[i] != other.masks[i] {
                    return false;
                }
            }
            true
        }
    );
    func!( // ne
        #[doc("Test for inequality.")]
        #[inline]
        #[must_use]
        const fn ne(self, other: Self) -> bool {
            let me = self;
            let mut mask_index = #vexillo::internal::ConstCounter::new(0usize);
            while let i @ 0..Self::MASK_COUNT = mask_index.next() {
                if me.masks[i] != other.masks[i] {
                    return true;
                }
            }
            false
        }
    );
    
    let inner = functions.into_iter().collect::<proc_macro2::TokenStream>();
    let type_name = input.type_name();
    let mut impl_block = quote!(
        impl #type_name {
            #inner
        }
    );
    config.visit_token_stream_mut(&mut impl_block);
    impl_block
}

fn build_op_impls(input: &FlagsInput) -> proc_macro2::TokenStream {
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
    let mut op_impls = quote!(
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
                debug_assert!((index as usize) < Self::SINGLE_FLAG_COUNT, "Out of bounds.");
                const BOOLS: [bool; 2] = [false, true];
                &BOOLS[self.get(index) as usize]
            }
        }
        
        impl ::core::ops::Index<usize> for #ty {
            type Output = bool;
            #[inline(always)]
            fn index(&self, index: usize) -> &bool {
                debug_assert!(index < Self::SINGLE_FLAG_COUNT, "Out of bounds.");
                const BOOLS: [bool; 2] = [false, true];
                &BOOLS[self.get(index as u32) as usize]
            }
        }
    );
    let mut config = input.config.lock().unwrap();
    config.visit_token_stream_mut(&mut op_impls);
    op_impls
}