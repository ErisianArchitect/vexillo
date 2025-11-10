Vexillo is a crate for creating bitflags types.

# Example
````````rust,no_run

vexillo::flags!(
    // Specify struct visibility, struct name, masks visibility, and masks type.
    pub struct MyFlags(pub [u16]);
    // Declare flag constants.
    // Specify visibility of constants (use priv for private).
    pub const {
        // This will be a public flag because the const block is public.
        FLAG0
        GROUP0: [
            // use `+` to add flags, `-` to remove them.
            // Order of additions and removals does not matter,
            // they will be reordered so that additions come before removals.
            + FLAG0
            FLAG1
            FLAG2
            FLAG3
        ]
        // This will create a constant named GROUP1 with the following flags:
        // FLAG3
        // FLAG4
        // FLAG5
        // FLAG6
        GROUP1: [
            + GROUP0
            // Use `|` to join flags together.
            - FLAG0
            | FLAG1
            | FLAG2
            FLAG4
            SUBGROUP: [
                FLAG5
                FLAG6
            ]
        ]
    }
);

assert!(
    MyFlags::SUBGROUP.has_all(MyFlags::union(&[
        MyFlags::FLAG5,
        MyFlags::FLAG6,
    ]))
);

assert!(
    MyFlags::GROUP1.has_all(MyFlags::union(&[
        MyFlags::FLAG3,
        MyFlags::FLAG4,
        MyFlags::FLAG5,
        MyFlags::FLAG6,
        MyFlags::SUBGROUP,
    ]))
    &&
    MyFlags::GROUP1.has_none(MyFlags::union(&[
        MyFlags::FLAG0,
        MyFlags::FLAG1,
        MyFlags::FLAG2,
    ]))
);
assert_eq!(MyFlags::TOTAL_FLAG_COUNT, 10);
assert_eq!(MyFlags::SINGLE_FLAG_COUNT, 7);
assert_eq!(MyFlags::GROUP_FLAG_COUNT, 3);

````````
___