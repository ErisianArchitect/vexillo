use vexillo::*;

flags!(
    pub struct Flags(pub [u32]);
    override {
        pub not: bitnot
    }
    pub const {
        FLAG0
        FLAG1
        FLAG2
        GROUP0: [
            FLAG3
            SUBGROUP0: [
                FLAG4
                SUBGROUP1: [
                    FLAG5
                    + FLAG0
                ]
            ]
            - FLAG0
        ]
    }
);

#[test]
fn test_consts() {
    assert_eq!(
        Flags::SINGLE_FLAG_COUNT,
        Flags::USED_BITS as usize,
    );
    assert_eq!(
        Flags::SINGLE_FLAG_COUNT,
        6,
    );
    assert_eq!(
        Flags::GROUP_FLAG_COUNT,
        3,
    );
    assert_eq!(
        Flags::TOTAL_FLAG_COUNT,
        9,
    );
    assert_eq!(
        Flags::LAST_MASK_INDEX,
        0,
    );
    assert_eq!(
        Flags::MASK_COUNT,
        1,
    );
    assert_eq!(
        Flags::MASK_BITS,
        32,
    );
    assert_eq!(
        Flags::BITS,
        32,
    );
    assert_eq!(
        Flags::UNUSED_BITS as usize,
        32 - Flags::SINGLE_FLAG_COUNT,
    );
    assert_eq!(
        Flags::MASK_SIZE,
        4,
    );
    assert_eq!(
        size_of::<Flags>(),
        4,
    );
}

#[test]
fn test_functions() {
    assert_eq!(
        Flags::ALL.count_ones(),
        Flags::USED_BITS,
    );
    assert_eq!(
        Flags::ALL.count_zeros(),
        0
    );
    assert_eq!(
        Flags::NONE.count_ones(),
        0,
    );
    assert_eq!(
        Flags::NONE.count_zeros(),
        Flags::USED_BITS,
    );
    assert_eq!(
        Flags::SUBGROUP1.count_ones(),
        2,
    );
}

#[test]
fn test_ops() {
    let flag: Flags = Flags::FLAG0 | Flags::FLAG1;
    assert!(
        flag.has_all(Flags::union(&[Flags::FLAG0, Flags::FLAG1]))
    );
}

/*
IMPLY: !a | b
a   b   r
0   0   1
0   1   1
1   0   0
1   1   1
EXAMPLE

"These bits must be 1"
NIMPLY: a & !b
a   b   r
0   0   0
0   1   0
1   0   1
1   1   0
"These bits must be 0"
has_any():
0   0   
0   1
1   0
1   1   0
EXAMPLE:
BANNED      1   
MUTED       1
CHANNELS    0:  0   0
BAN         0:  0   0
UNBAN       0:  
GRANT       0:
REVOKE      0:
*/

#[test]
fn imply_test() {
    flags!(
        struct Perms([u8]);
        const {
            // experimental
            // @reserve(5)
            PRIVATE
            PUBLIC
            WRITE
        }
    );
    _=Perms::ALL;
}