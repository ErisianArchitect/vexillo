//! Vexillo is a crate for creating bitflags types.
//! 
//! # Example
//! ````````rust,no_run
//! 
//! vexillo::flags!(
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
//! 
//! assert!(
//!     MyFlags::SUBGROUP.has_all(MyFlags::union(&[
//!         MyFlags::FLAG5,
//!         MyFlags::FLAG6,
//!     ]))
//! );
//! 
//! assert!(
//!     MyFlags::GROUP1.has_all(MyFlags::union(&[
//!         MyFlags::FLAG3,
//!         MyFlags::FLAG4,
//!         MyFlags::FLAG5,
//!         MyFlags::FLAG6,
//!         MyFlags::SUBGROUP,
//!     ]))
//!     &&
//!     MyFlags::GROUP1.has_none(MyFlags::union(&[
//!         MyFlags::FLAG0,
//!         MyFlags::FLAG1,
//!         MyFlags::FLAG2,
//!     ]))
//! );
//! assert_eq!(MyFlags::TOTAL_FLAG_COUNT, 10);
//! assert_eq!(MyFlags::SINGLE_FLAG_COUNT, 7);
//! assert_eq!(MyFlags::GROUP_FLAG_COUNT, 3);
//! 
//! ````````
//! ___

#[doc(hidden)]
pub mod internal;
mod shared;
pub use shared::*;

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

#[macro_export]
macro_rules! flags {
    ($($tokens:tt)*) => {
        $crate::internal::flags_internal!{
            use $crate;
            $($tokens)*
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn flags_test() {
        flags!{
            #[doc = "Permissions"]
            pub struct Perms(pub [u8]);
            // Since the root is pub, all flags within the root without an
            // explicit visibility modifier will also be pub.
            pub const {
                OWNER: [
                    GRANT_ADMIN
                    REVOKE_ADMIN
                    SHUTDOWN_SERVER
                    CLEAR_LOG
                    ADMIN: [
                        GRANT_SUPER
                        REVOKE_SUPER
                        CREATE_CHANNEL
                        DELETE_CHANNEL
                        RENAME_CHANNEL
                        RESTART_SERVER
                        SUPER: [
                            GRANT_MOD
                            REVOKE_MOD
                            MOD: [
                                /// Gives access to mod channels
                                MOD_CHANNELS
                                BAN_USER
                                UNBAN_USER
                                APPROVE_USER
                                USER: [
                                    /// Gives access to user channels.
                                    USER_CHANNELS
                                    GUEST: [
                                        /// Gives access to the lobby.
                                        LOBBY
                                        /// Allows to message the mods.
                                        MESSAGE_MODS
                                    ]
                                ]
                            ]
                        ]
                    ]
                ]
            }
        }
    }
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