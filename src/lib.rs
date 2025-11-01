extern crate vexcore as vecproccore;
/*

vexillo::flags!{
    // Define type with `vis struct Name(vis [FlagIntType]);
    // FlagIntType must be one of the following: u8, u16, u32, or u64.
    // The FlagIntType determines the type to use for bit masks. `vis` determines
    // visibility. Default visibility is private.
    /// Example flags struct.
    pub struct ExampleFlags(pub [u64]);
    // Optional:
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
    // You must have at least one flag group.
    // Place flags in groups. Use `priv` for private flags.
    // Each flag group can have attributes attached, which will be applied to all flag
    // constants within the group.
    priv: [
        PRIVATE_FLAG
    ]
    // Use `pub` for public flags.
    pub: [
        /// You can put attributes, including doc comments, on flags.
        FLAG_NAME
        // You can also create union flags:
        UNION_FLAGS = [
            PRIVATE_FLAG
            FLAG_NAME
        ]
        OTHER_UNION = [
            // flag unions can contains other flag unions
            UNION_FLAGS
            // And you can remove flags that have been added with `-`.
            // Flag additions and removals happen in the order that they
            // appear, so if it's important that a union does not have
            // a flag, you should place the removal after all additions.
            // This ordering happens so that conditional flags can add flags that were previously removed.
            -PRIVATE_FLAG
        ]
    ]
}

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