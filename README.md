Vexillo is a crate for creating bitflags types.

# Example
___
````````rust,no_run
use vexillo::flags;

flags!(
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
````````
___