use std::{collections::HashSet, sync::LazyLock};

use quote::quote;
use syn::{
    Attribute, Error, Ident, Token, Visibility, braced, bracketed, parse::Parse
};
use crate::{override_block::OverrideBlock, vis::Vis};

struct AddFlagsItem {
    flags: Vec<Ident>,
}

struct RemoveFlagsItem {
    flags: Vec<Ident>,
}

struct DeclareFlagItem {
    attrs: Vec<Attribute>,
    vis: Vis,
    ident: Ident,
}

struct DeclareGroupItem {
    attrs: Vec<Attribute>,
    vis: Vis,
    ident: Ident,
    items: Vec<GroupItem>,
}

enum DeclareItem {
    Single(DeclareFlagItem),
    Group(DeclareGroupItem),
}

impl Parse for DeclareFlagItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl Parse for DeclareGroupItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        _=input.parse::<Token![:]>()?;
        let content;
        bracketed!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self {
            attrs,
            vis,
            ident,
            items,
        })
    }
}

impl Parse for DeclareItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        _=fork.call(Attribute::parse_outer)?;
        _=fork.parse::<Vis>()?;
        if fork.peek2(Token![:]) {
            // Group
            Ok(Self::Group(input.parse()?))
        } else {
            Ok(Self::Single(input.parse()?))
        }
    }
}


impl DeclareItem {
    pub fn verify<'a>(&'a self, verifier: &mut IdentVerifier<'a>) -> syn::Result<()> {
        fn repeat_definition_err(first: &Ident, second: &Ident) -> syn::Error {
            let mut first_err = syn::Error::new(
                first.span(),
                format!("`{}` first defined here.", first)
            );
            let second = syn::Error::new(
                second.span(),
                format!("`{}` repeat definition.", second)
            );
            first_err.combine(second);
            first_err
        }
        macro_rules! verify_and_insert {
            ($item:ident) => {
                {
                    crate::verify_no_cfg(&$item.attrs, crate::FLAG_CFG_ERR_MSG)?;
                    if let Some(repeat) = verifier.get(&$item.ident) {
                        return Err(repeat_definition_err(repeat, &$item.ident));
                    } else {
                        verifier.insert_and_verify(&$item.ident)?;
                    }
                }
            };
        }
        match self {
            DeclareItem::Single(item) => {
                verify_and_insert!(item);
            },
            DeclareItem::Group(item) => {
                verify_and_insert!(item);
                item.items
                    .iter()
                    .try_for_each(move |item| {
                        match item {
                            GroupItem::Declare(declare_item) => declare_item.verify(verifier),
                            _ => Ok(()),
                        }
                    })?;
            },
        }
        Ok(())
    }
}

enum GroupItem {
    Add(AddFlagsItem),
    Remove(RemoveFlagsItem),
    Declare(DeclareItem),
}

fn read_pipe_separated_idents_into(input: &syn::parse::ParseStream, output: &mut Vec<Ident>) -> syn::Result<()> {
    output.push(input.parse()?);
    while input.peek(Token![|]) {
        _=input.parse::<Token![|]>()?;
        output.push(input.parse()?);
    }
    Ok(())
}

impl Parse for GroupItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![+]) {
            _=input.parse::<Token![+]>()?;
            let mut flags = Vec::new();
            read_pipe_separated_idents_into(&input, &mut flags)?;
            Ok(GroupItem::Add(AddFlagsItem {
                flags,
            }))
        } else if input.peek(Token![-]) {
            _=input.parse::<Token![-]>()?;
            let mut flags = Vec::new();
            read_pipe_separated_idents_into(&input, &mut flags)?;
            Ok(GroupItem::Remove(RemoveFlagsItem {
                flags,
            }))
        } else {
            Ok(GroupItem::Declare(input.parse()?))
        }
    }
}

pub(crate) struct ConstSingle {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
}

pub(crate) struct ConstGroup {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub additions: Vec<Ident>,
    pub removals: Vec<Ident>,
}

struct ConstGroupBuilder {
    vis: Visibility,
    additions: Vec<Ident>,
    removals: Vec<Ident>,
    singles: Vec<ConstSingle>,
    groups: Vec<ConstGroup>,
}

struct ConstBlockBuilder {
    pub vis: Visibility,
    pub singles: Vec<ConstSingle>,
    pub groups: Vec<ConstGroup>,
}

impl ConstBlockBuilder {
    fn build_single(&mut self, item: &DeclareFlagItem) {
        self.singles.push(ConstSingle {
            attrs: item.attrs.clone(),
            vis: item.vis.resolve(Some(&self.vis)),
            ident: item.ident.clone(),
        });
    }
    
    fn build_group(&mut self, item: &DeclareGroupItem) {
        let mut builder = ConstGroupBuilder {
            vis: item.vis.resolve(Some(&self.vis)),
            additions: Vec::new(),
            removals: Vec::new(),
            singles: Vec::new(),
            groups: Vec::new(),
        };
        builder.build_group(item);
        self.singles.extend(builder.singles);
        self.groups.extend(builder.groups);
        self.groups.push(ConstGroup {
            attrs: item.attrs.clone(),
            vis: builder.vis,
            ident: item.ident.clone(),
            additions: builder.additions,
            removals: builder.removals,
        });
    }
}

impl ConstGroupBuilder {
    fn build_single(&mut self, item: &DeclareFlagItem) {
        self.singles.push(ConstSingle {
            attrs: item.attrs.clone(),
            vis: item.vis.resolve(Some(&self.vis)),
            ident: item.ident.clone(),
        });
        self.additions.push(item.ident.clone());
    }
    
    fn build_group(&mut self, item: &DeclareGroupItem) {
        for item in item.items.iter() {
            match item {
                GroupItem::Add(AddFlagsItem { flags }) => {
                    self.additions.extend(flags.iter().cloned());
                },
                GroupItem::Remove(RemoveFlagsItem { flags }) => {
                    self.removals.extend(flags.iter().cloned());
                },
                GroupItem::Declare(declare) => {
                    match declare {
                        DeclareItem::Single(single) => {
                            self.build_single(single);
                        },
                        DeclareItem::Group(group) => {
                            let mut builder = Self {
                                vis: group.vis.resolve(Some(&self.vis)),
                                additions: Vec::new(),
                                removals: Vec::new(),
                                singles: Vec::new(),
                                groups: Vec::new(),
                            };
                            builder.build_group(group);
                            self.singles.extend(builder.singles);
                            self.groups.extend(builder.groups);
                            self.groups.push(ConstGroup {
                                attrs: group.attrs.clone(),
                                vis: builder.vis,
                                ident: group.ident.clone(),
                                additions: builder.additions,
                                removals: builder.removals,
                            });
                            self.additions.push(group.ident.clone());
                        },
                    }
                },
            }
        }
    }
}

impl ConstBlockBuilder {
    #[inline]
    fn new(vis: Visibility) -> Self {
        Self {
            vis,
            singles: Vec::new(),
            groups: Vec::new(),
        }
    }
}

pub(crate) struct ConstBlock {
    vis: Vis,
    items: Vec<DeclareItem>,
}

/*
DeclareItem
    DeclareFlagItem
    DeclareGroupItem
        Add (Vec<Ident>)
        Remove (Vec<Ident>)
        Declare(DeclareItem)
*/

static RESERVED_CONST_NAMES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        // pub
        "SINGLE_FLAG_COUNT",
        // pub
        "GROUP_FLAG_COUNT",
        // pub
        "TOTAL_FLAG_COUNT",
        // pub
        "BITS",
        // pub
        "UNUSED_BITS",
        // pub
        "USED_BITS",
        // pub
        "MASK_BITS",
        // pub
        "MASK_COUNT",
        // pub
        "NONE",
        // pub
        "ALL",
        // priv
        "TABLES",
    ])
});

struct IdentVerifier<'a> {
    ident_buffer: String,
    declared: HashSet<&'a Ident>,
}

impl<'a> IdentVerifier<'a> {
    const STRING_BUF_CAP: usize = 256;
    // I figured that 64 was a good minimum.
    const HASHSET_CAP: usize = 64;
    fn new() -> Self {
        Self {
            ident_buffer: String::with_capacity(Self::STRING_BUF_CAP),
            declared: HashSet::with_capacity(Self::HASHSET_CAP),
        }
    }
    
    fn get(&self, ident: &Ident) -> Option<&'a Ident> {
        self.declared.get(ident).copied()
    }
    
    fn insert_and_verify(&mut self, ident: &'a Ident) -> syn::Result<()> {
        use std::fmt::Write;
        self.ident_buffer.clear();
        write!(self.ident_buffer, "{}", ident).unwrap();
        if RESERVED_CONST_NAMES.contains(self.ident_buffer.as_str()) {
            return Err(syn::Error::new(
                ident.span(),
                format!("`{ident}` is a reserved identifier.")
            ));
        }
        self.declared.insert(ident);
        Ok(())
    }
}

pub(crate) struct ConstBuildResult {
    pub singles: Vec<ConstSingle>,
    pub groups: Vec<ConstGroup>,
}

impl ConstBuildResult {
    pub fn tokenize(&self, override_block: &OverrideBlock) -> proc_macro2::TokenStream {
        let from_index: Ident = syn::parse_quote!(from_index);
        let from_index = override_block.get_alt(&from_index).unwrap_or(&from_index);
        let add: Ident = syn::parse_quote!(add);
        let add = override_block.get_alt(&add).unwrap_or(&add);
        let rem: Ident = syn::parse_quote!(remove);
        let rem = override_block.get_alt(&rem).unwrap_or(&rem);
        let new: Ident = syn::parse_quote!(new);
        let new = override_block.get_alt(&new).unwrap_or(&new);
        let singles = self.singles
            .iter()
            .enumerate()
            .map(|(i, single)| {
                let index = i as u32;
                let ConstSingle { attrs, vis, ident } = single;
                quote!(
                    #(#attrs)*
                    #vis const #ident: Self = Self::#from_index(#index);
                )
            }).collect::<proc_macro2::TokenStream>();
        let groups = self.groups
            .iter()
            .map(|group| {
                let ConstGroup { attrs, vis, ident, additions, removals } = group;
                let additions = additions.iter()
                    .map(|addition| {
                        quote!(
                            builder.#add(Self::#addition);
                        )
                    }).collect::<proc_macro2::TokenStream>();
                let removals = removals.iter()
                    .map(|removal| {
                        quote!(
                            builder.#rem(Self::#removal);
                        )
                    }).collect::<proc_macro2::TokenStream>();
                quote!(
                    #(#attrs)*
                    #vis const #ident: Self = {
                        let mut builder = Self::#new();
                        #additions
                        #removals
                        builder
                    };
                )
            }).collect::<proc_macro2::TokenStream>();
        quote!(
            #singles
            #groups
        )
    }
}

impl ConstBlock {
    fn verify(self) -> syn::Result<Self> {
        let mut verifier = IdentVerifier::new();
        // Check for repeat declarations.
        self.items.iter().try_for_each(|item| {
            item.verify(&mut verifier)
        })?;
        Ok(self)
    }
    
    pub fn build(&self) -> ConstBuildResult {
        let mut builder = ConstBlockBuilder::new(self.vis.resolve(None));
        for item in self.items.iter() {
            match item {
                DeclareItem::Single(single) => {
                    builder.build_single(single);
                },
                DeclareItem::Group(group) => {
                    builder.build_group(group);
                },
            }
        }
        ConstBuildResult {
            singles: builder.singles,
            groups: builder.groups,
        }
    }
}

impl Parse for ConstBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        _=input.parse::<Token![const]>()?;
        let inner;
        braced!(inner in input);
        let mut items = Vec::new();
        while !inner.is_empty() {
            items.push(inner.parse()?);
        }
        if items.is_empty() {
            return Err(Error::new(input.span(), "Must declare at least one flag constant."));
        }
        Self {
            vis,
            items,
        }.verify()
    }
}

// impl<'a> FlagConstants<'a> {
//     pub fn build(const_block: &'a ConstBlock) -> Self {
//         // First collect all single flags
//         let mut builder = FlagConstants {
//             singles: Vec::new(),
//             groups: Vec::new(),
//         };
//         const_block.items
//             .iter()
//             .for_each(|item| {
                
//             });
//         todo!()
//     }
// }