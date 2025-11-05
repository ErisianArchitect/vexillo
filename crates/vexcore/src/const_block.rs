use std::collections::HashSet;

use syn::{
    Attribute, Error, Ident, Token, braced, bracketed, parse::Parse, token::Colon
};
use crate::vis::Vis;

pub struct AddFlagsItem {
    pub flags: Vec<Ident>,
}

pub struct RemoveFlagsItem {
    pub flags: Vec<Ident>,
}

pub struct DeclareFlagItem {
    pub attrs: Vec<Attribute>,
    pub vis: Vis,
    pub ident: Ident,
}

pub struct DeclareGroupItem {
    pub attrs: Vec<Attribute>,
    pub vis: Vis,
    pub ident: Ident,
    pub items: Vec<GroupItem>,
}

pub enum DeclareItem {
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
    pub fn verify<'a>(&'a self, declared_idents: &mut HashSet<&'a Ident>) -> syn::Result<()> {
        match self {
            DeclareItem::Single(declare_flag_item) => {
                if let Some(&repeat) = declared_idents.get(&declare_flag_item.ident) {
                    let mut first_err = syn::Error::new(
                        repeat.span(),
                        format!("`{}` first defined here.", repeat)
                    );
                    let second_err = syn::Error::new(
                        declare_flag_item.ident.span(),
                        format!("`{}` repeat definition.", declare_flag_item.ident)
                    );
                    first_err.combine(second_err);
                    return Err(first_err);
                } else {
                    declared_idents.insert(&declare_flag_item.ident);
                }
            },
            DeclareItem::Group(declare_group_item) => {
                if let Some(&repeat) = declared_idents.get(&declare_group_item.ident) {
                    let mut first_err = syn::Error::new(
                        repeat.span(),
                        format!("`{}` first defined here.", repeat)
                    );
                    let second_err = syn::Error::new(
                        declare_group_item.ident.span(),
                        format!("`{}` repeat definition.", declare_group_item.ident)
                    );
                    first_err.combine(second_err);
                    return Err(first_err);
                } else {
                    declared_idents.insert(&declare_group_item.ident);
                }
                declare_group_item.items
                    .iter()
                    .try_for_each(|item| {
                        match item {
                            GroupItem::Declare(declare_item) => declare_item.verify(declared_idents),
                            _ => Ok(()),
                        }
                    })?;
            },
        }
        Ok(())
    }
}

pub enum GroupItem {
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

pub struct NamedGroup {
    pub vis: Vis,
    pub ident: Ident,
    pub items: Vec<GroupItem>,
}

impl Parse for NamedGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        _=input.parse::<Colon>()?;
        let content;
        bracketed!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self {
            vis,
            ident,
            items,
        })
    }
}

pub struct DeclareGroup {
    pub vis: Vis,
    pub declarations: Vec<DeclareFlagItem>,
}

impl Parse for DeclareGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        _ = input.parse::<Token![:]>()?;
        let group;
        bracketed!(group in input);
        let mut declarations = Vec::new();
        while !group.is_empty() {
            declarations.push(group.parse()?);
        }
        Ok(Self {
            vis,
            declarations,
        })
    }
}

pub enum FlagGroup {
    Named(NamedGroup),
    Declare(DeclareGroup),
}

impl Parse for FlagGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let _vis = fork.parse::<Vis>()?;
        if fork.peek(Ident) && fork.peek2(Token![:]) {
            Ok(FlagGroup::Named(input.parse()?))
        } else {
            Ok(FlagGroup::Declare(input.parse()?))
        }
    }
}

pub struct ConstBlock {
    pub vis: Vis,
    pub items: Vec<DeclareItem>,
}

impl ConstBlock {
    pub fn verify(self) -> syn::Result<Self> {
        let mut declared_idents = HashSet::<&Ident>::new();
        // Check for repeat declarations.
        self.items.iter().try_for_each(|item| {
            item.verify(&mut declared_idents)
        })?;
        Ok(self)
    }
}

impl Parse for ConstBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        _=input.parse::<Token![const]>()?;
        let inner;
        braced!(inner in input);
        let mut groups = Vec::new();
        while !inner.is_empty() {
            groups.push(inner.parse()?);
        }
        if groups.is_empty() {
            return Err(Error::new(input.span(), "Must declare at least one flag constant."));
        }
        Self {
            vis,
            items: groups,
        }.verify()
    }
}

pub enum ConstantGroupItem<'a> {
    Add(&'a Ident),
    Remove(&'a Ident),
}

pub struct ConstantGroup<'a> {
    pub ident: &'a Ident,
    pub items: Vec<ConstantGroupItem<'a>>,
}

pub struct FlagConstants<'a> {
    pub singles: Vec<&'a Ident>,
    pub groups: Vec<ConstantGroup<'a>>,
}

impl<'a> FlagConstants<'a> {
    pub fn build(const_block: &'a ConstBlock) -> Self {
        // First collect all single flags
        let mut builder = FlagConstants {
            singles: Vec::new(),
            groups: Vec::new(),
        };
        const_block.items
            .iter()
            .for_each(|item| {
                
            });
        todo!()
    }
}