//! Vexillo is a crate for creating bitflags types.
//! 
//! # Example
//! ___
//! ````````rust,no_run
//! use vexillo::flags;
//! 
//! flags!(
//!     // Specify struct visibility, struct name, masks visibility, and masks type.
//!     pub struct MyFlags(pub [u16]);
//!     // Declare flag constants.
//!     // Specify visibility of constants (use priv for private).
//!     pub const {
//!         // This will be a public flag because the const block is public.
//!         FLAG0
//!         GROUP0: [
//!             // use `+` to add flags, `-` to remove them.
//!             // Order of additions and removals does not matter,
//!             // they will be reordered so that additions come before removals.
//!             + FLAG0
//!             FLAG1
//!             FLAG2
//!             FLAG3
//!         ]
//!         // This will create a constant named GROUP1 with the following flags:
//!         // FLAG3
//!         // FLAG4
//!         // FLAG5
//!         // FLAG6
//!         GROUP1: [
//!             + GROUP0
//!             // Use `|` to join flags together.
//!             - FLAG0
//!             | FLAG1
//!             | FLAG2
//!             FLAG4
//!             SUBGROUP: [
//!                 FLAG5
//!                 FLAG6
//!             ]
//!         ]
//!     }
//! );
//! ````````
//! ___

mod internal;
mod shared;
pub use internal::{
    ConstCounter,
    MaskIndex,
    MustBeUnsignedInt,
    const_cmp_str,
    mask_type_check,
    subslice,
    subslice_mut,
};
pub use shared::*;
pub use vexproc::{flags};
#[doc(hidden)]
pub use vexmacro::const_binary_search_fn;

// flags!{
//     // Define type with `vis struct Name(vis [FlagIntType]);
//     // FlagIntType must be one of the following: u8, u16, u32, or u64.
//     // The FlagIntType determines the type to use for bit masks. `vis` determines
//     // visibility. Default visibility is private.
//     /// Example flags struct.
//     pub struct ExampleFlags(pub [u64]);
//     // Optional:
//     override {
//         // Change name or visibility of builtin functions/constants.
//         // You can not remove these builtin functions as they might be necessary for certain
//         // functionality to work. You can hide them by modifying their visibility.
//         // Creation
//         // vis original_name: new_name
//         // use priv to make it private.
//         // no visibility modifier means that it will use the default visibility, which is `pub` for
//         // most of the builtin functions.
//         pub none: empty
//         pub all: full
//         pub new: create
//         pub union
//         pub union_without
//         pub try_find
//         pub find
//         pub find_or
//         pub find_or_none
//         pub count_ones
//         pub count_zeros
//         pub add
//         pub add_all
//         pub remove
//         pub remove_all
//         pub with
//         pub with_all
//         pub without
//         pub without_all
//         pub get
//         pub set
//         pub from_index
//         pub has_all
//         pub has_none
//         pub has_any
//         pub masks
//         pub masks_mut
//         pub into_inner
//         pub as_bytes
//         pub as_mut_bytes
//         pub to_be_bytes
//         pub to_le_bytes
//         pub to_ne_bytes
//         pub from_be_bytes
//         pub from_le_bytes
//         pub from_ne_bytes
//         // Bitwise logic
//         pub not
//         pub and
//         pub or
//         pub nand
//         pub nor
//         pub xor
//         pub xnor
//         pub imply
//         pub nimply
//         // Comparisons
//         pub eq
//         pub ne
//     }
//     pub const {
//         FLAG0
//         // Declaration
//         priv DECLARATION
//         // Group
//         #[doc = "hello, world"]
//         pub GROUP: [
//             + FLAG0
//             ALPHA
//             BETA
//             CAPPA
//         ]
//         // Empty group to specify no flags set.
//         pub TEST: []
//         pub FLAGS: [
//             // [vis] <identifier> is a flag declaration. These flags will be assigned a single bit, in the order that they appear.
//             APPLE
//             BANANA
//             STRAWBERRY
//             pub FRUIT: [
//                 // Use + to add flags, use | to join them together.
//                 + APPLE | BANANA | STRAWBERRY
//                 // Alternatively, it looks nice when you put them all on the same line:
//                 // + APPLE
//                 // | BANANA
//                 // | STRAWBERRY
//                 // Use - to remove flags.
//                 // This removes BANANA and STRAWBERRY.
//                 - BANANA
//                 | STRAWBERRY
//             ]
//             // Alternatively, you could have declared the fruit inside of the FRUIT group.
//             pub PUBLIC: [
//                 // All flags in PUBLIC will be pub unless otherwise specified.
//                 // These are all public single flag consts.
//                 ONE
//                 TWO
//                 THREE
//             ]
//         ]
//         // You can bind a flag to another name with this simple trick:
//         pub FULL: [+ALL]
//     }
// }

// flags!(
//     pub struct Flags(pub [u8]);
//     pub const {
//         // Hide first flag for no apparent reason.
//         priv FLAG0
//         priv FLAG1
//         GROUP0: [
//             + FLAG0
//             | FLAG1
//         ]
//     }
// );

// flags!{
//     #[doc = "Permissions"]
//     pub struct Perms(pub [u8]);
//     // Since the root is pub, all flags within the root without an
//     // explicit visibility modifier will also be pub.
//     pub const {
//         OWNER: [
//             GRANT_ADMIN
//             REVOKE_ADMIN
//             SHUTDOWN_SERVER
//             CLEAR_LOG
//             ADMIN: [
//                 GRANT_SUPER
//                 REVOKE_SUPER
//                 CREATE_CHANNEL
//                 DELETE_CHANNEL
//                 RENAME_CHANNEL
//                 RESTART_SERVER
//                 SUPER: [
//                     GRANT_MOD
//                     REVOKE_MOD
//                     MOD: [
//                         /// Gives access to mod channels
//                         MOD_CHANNELS
//                         BAN_USER
//                         UNBAN_USER
//                         APPROVE_USER
//                         USER: [
//                             /// Gives access to user channels.
//                             USER_CHANNELS
//                             GUEST: [
//                                 /// Gives access to the lobby.
//                                 LOBBY
//                                 /// Allows to message the mods.
//                                 MESSAGE_MODS
//                             ]
//                         ]
//                     ]
//                 ]
//             ]
//         ]
//     }
// }

flags!(
    pub struct Flags(pub [u64]);
    pub const {
        UNUSED
        FLAG
        GROUP: [
            + FLAG
            FOO
            BAR
            BAZ
        ]
    }
);

// TODO: Remove this test before publish.
#[test]
fn flags_test() {
    // This is just so I can check the crate-level documentation with hover.
    #[allow(unused)]
    use crate as vexillo;
    let flags = Flags::FLAG;
    let ones = flags.count_ones();
    assert_eq!(ones, 1);
    let bytes = flags.to_be_bytes();
    let ser_flags = Flags::from_be_bytes(bytes);
    assert_eq!(flags, ser_flags);
    let group = Flags::GROUP;
    assert!(group.has_all(Flags::FLAG));
    assert!(group.has_all(Flags::FOO));
    assert!(group.has_all(Flags::BAR));
    assert!(group.has_all(Flags::BAZ));
    assert!(group.has_all(Flags::GROUP));
    let not_group = group.not();
    assert!(group.has_none(not_group));
    assert!(group.has_none(Flags::UNUSED));
    assert!(group.eq(group));
    assert!(group.ne(flags));
    assert!(group.with(Flags::UNUSED).has_all(Flags::union(&[Flags::GROUP, Flags::UNUSED])));
}

/*


// Builtin flag constants:
// - `NONE` (No bits set)
// - `ALL` (All used bits set)
// Builtin constants in the form of functions, so as to not pollute the constants.
// - `flag_count` (Number of single bit flags)
// - `mask_count` (The number of masks in the masks array)
// - `mask_bit_size` (The size in bits of the mask type)
// - `index_order_flags` (An array of single flags ordered by index ascending)
//      pub const index_order_flags() -> &'static [(&'static str, Self)]
// - `bit_size_order_flags` (An array of flags ordered by bit count descending)
//      pub const bit_size_order_flags() -> &'static [(&'static str, Self)]
// - `lowercase_names` (An array of lowercase names for single flags in the order of their index)
// - `uppercase_names` (An array of uppercase names for single flags in the order of their index)

Functions that are built-in internals:
    /// (mask_index, bit_index)
    const fn mask_indices(index: u32) -> (usize, u32)
*/