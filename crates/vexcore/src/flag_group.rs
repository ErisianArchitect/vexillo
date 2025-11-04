use syn::{Ident, Token, bracketed, parse::Parse, token::Colon};
use vexmacro::ensure_eof;

use crate::vis::Vis;


macro_rules! prototype { ($($_:tt)*) => {} }

prototype!{
    cfg_attrs: [
        example::cfg_attrs::path,
        
    ]
    pub struct Flags(pub [u64]);
    impl {
        // Change name or visibility of builtin functions/constants.
        // You can not remove these builtin functions as they might be necessary for certain
        // functionality to work. You can hide them by modifying their visibility.
        // Creation
        // vis original_name: new_name
        // use priv to make it private.
        // no visibility modifier means that it will use the default visibility, which is `pub` for
        // most of the builtin functions.
        pub new: create
        pub none: empty
        pub all: full
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
        // Bitwise logic
        pub not
        pub and
        pub or
        pub nand
        pub nor
        pub xor
        pub xnor
        pub imply
        pub nimply
        // Comparisons
        pub eq
        pub ne
    }
    // @force_upper
    // vis [ GROUP [ : PREFIX ] ] = [
    //     (
    //         FLAG
    //     )*
    // ]
    pub NONE: []
    // A group with an underscore as a name will not have its own constant for the bitmask of the group.
    _: [
    ]
    pub ALL: [
        
    ]
    // empty group will create a bitmask with all of the bits set to 0.
    
    pub EMPTY: []
    pub MOD: [
        BAN_USER
        UNBAN_USER
        CHANNEL_BROADCAST
    ]
    pub SUPER: [
        + MOD
        
    ]
    pub ADMIN: [
        + MOD
        | SUPER
        | EXAMPLE
        - SOME
        | FLAGS
        | TO
        | REMOVE
        GROUP: [
            // Define flags here
            MOD
            SUPER
            + ADD
            | FLAGS
            | HERE
            - REMOVE
            | FLAGS
            | HERE
            SUBGROUP: [
                SAME
                AS
                IT
                EVER: [
                    WAS
                ]
            ]
        ]
        GRANT_MOD
        GRANT_ADMIN
        REVOKE_MOD
        REVOKE_ADMIN
    ]
}

pub struct AddFlagsItem {
    pub flags: Vec<Ident>,
}

pub struct RemoveFlagsItem {
    pub flags: Vec<Ident>,
}

pub struct DeclareFlagItem {
    pub vis: Vis,
    pub ident: Ident,
}

pub enum FlagItem {
    Add(AddFlagsItem),
    Remove(RemoveFlagsItem),
    Group(Box<FlagGroup>),
    Declare(DeclareFlagItem),
}

impl Parse for FlagItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![+]) {
            _=input.parse::<Token![+]>()?;
            let first = input.parse::<Ident>()?;
            let mut flags = vec![first];
            while input.peek(Token![|]) {
                _=input.parse::<Token![|]>()?;
                flags.push(input.parse()?);
            }
            Ok(FlagItem::Add(AddFlagsItem {
                flags,
            }))
        } else if input.peek(Token![-]) {
            _=input.parse::<Token![-]>()?;
            let first = input.parse::<Ident>()?;
            let mut flags = vec![first];
            while input.peek(Token![|]) {
                _=input.parse::<Token![|]>()?;
                flags.push(input.parse()?);
            }
            Ok(FlagItem::Remove(RemoveFlagsItem {
                flags,
            }))
        } else {
            let fork = input.fork();
            let _vis = fork.parse::<Vis>()?;
            let _ident = fork.parse::<Ident>()?;
            if fork.peek(Token![:]) {
                // group
                let group = input.parse::<FlagGroup>()?;
                Ok(FlagItem::Group(Box::new(group)))
            } else {
                // declaration
                let vis = input.parse::<Vis>()?;
                let ident = fork.parse::<Ident>()?;
                Ok(FlagItem::Declare(DeclareFlagItem {
                    vis,
                    ident,
                }))
            }
        }
    }
}

pub struct FlagGroup {
    pub vis: Vis,
    pub ident: Option<Ident>,
    pub items: Vec<FlagItem>,
}

impl Parse for FlagGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse::<Vis>()?;
        let ident = input.parse::<Option<Ident>>()?;
        let _colon: Colon = input.parse()?;
        let group_content;
        bracketed!(group_content in input);
        let mut items = Vec::new();
        while !group_content.is_empty() {
            items.push(group_content.parse()?);
        }
        Ok(Self {
            vis,
            ident,
            items,
        })
    }
}