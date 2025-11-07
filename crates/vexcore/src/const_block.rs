use std::{collections::HashSet, sync::LazyLock};

use syn::{
    Attribute, Error, Ident, Token, braced, bracketed, parse::Parse
};
use crate::vis::Vis;

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
    pub fn verify<'a>(&'a self, declared_idents: &mut HashSet<&'a Ident>) -> syn::Result<()> {
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
                    if let Some(&repeat) = declared_idents.get(&$item.ident) {
                        return Err(repeat_definition_err(repeat, &$item.ident));
                    } else {
                        declared_idents.insert(&$item.ident);
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
                            GroupItem::Declare(declare_item) => declare_item.verify(declared_idents),
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

struct DeclareGroup {
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

struct ConstSingle<'a> {
    pub attrs: &'a [Attribute],
    pub ident: &'a Ident,
}

struct ConstGroup<'a> {
    pub attrs: &'a [Attribute],
    pub ident: &'a Ident,
    pub additions: Vec<&'a Ident>,
    pub removals: Vec<&'a Ident>,
}

struct ConstBlockBuilder<'a> {
    inner: FlagConstants<'a>,
}

impl<'a> ConstSingle<'a> {
    #[inline]
    pub fn new(attrs: &'a [Attribute], ident: &'a Ident) -> Self {
        Self {
            attrs,
            ident,
        }
    }
}

impl<'a> ConstGroup<'a> {
    
}

impl<'a> ConstBlockBuilder<'a> {
    #[inline]
    fn new() -> Self {
        Self {
            inner: FlagConstants {
                singles: Vec::new(),
                groups: Vec::new(),
            },
        }
    }
}

pub struct FlagConstants<'a> {
    pub singles: Vec<ConstSingle<'a>>,
    pub groups: Vec<ConstGroup<'a>>,
}

pub struct ConstBlock {
    pub vis: Vis,
    pub items: Vec<DeclareItem>,
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
        "BITS",
        // pub
        "SINGLE_FLAG_COUNT",
        // pub
        "GROUP_FLAG_COUNT",
        // pub
        "TOTAL_FLAG_COUNT",
        // pub
        "NONE",
        // pub
        "ALL",
        // priv
        "BIT_COUNT_DESCENDING_FLAGS",
        // priv
        "ORDERED_SINGLE_FLAGS",
        // priv
        "ORDERED_GROUP_FLAGS",
        // priv
        "ORDERED_FLAGS",
    ])
});

struct IdentVerifier<'a> {
    ident_buffer: String,
    declared: HashSet<&'a Ident>,
}

impl<'a> IdentVerifier<'a> {
    fn insert_and_verify(&mut self, ident: &'a Ident) -> syn::Result<()> {
        use std::fmt::Write;
        self.ident_buffer.clear();
        write!(self.ident_buffer, "{}", ident).unwrap();
        if RESERVED_CONST_NAMES.contains(&self.ident_buffer) {
            return Err(syn::Error::new(
                ident.span(),
                format!("`{ident}` is a Reserved identifier.")
            ));
        }
        
    }
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
    
    pub fn build<'a>(&'a self) -> FlagConstants<'a> {
        todo!()
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