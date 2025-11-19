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
    flags!(
        struct F(pub [u8]);
        const {
            F0
            F1
            F2
            F3
            F4
            F5
            F6
            F7
            F8
            F9
        }
    );
    //   __
    // 0b001
    assert_eq!(F::F0.trailing_zeros(), 0);
    assert_eq!(F::F1.trailing_zeros(), 1);
    assert_eq!(F::F2.trailing_zeros(), 2);
    assert_eq!(F::F3.trailing_zeros(), 3);
    assert_eq!(F::F4.trailing_zeros(), 4);
    
    assert_eq!(F::F9.leading_zeros(), 0);
    assert_eq!(F::F8.leading_zeros(), 1);
    assert_eq!(F::F7.leading_zeros(), 2);
    assert_eq!(F::F6.leading_zeros(), 3);
    assert_eq!(F::F5.leading_zeros(), 4);
    assert_eq!(F::F4.leading_zeros(), 5);
    assert_eq!(F::F3.leading_zeros(), 6);
    assert_eq!(F::F2.leading_zeros(), 7);
    assert_eq!(F::F0.leading_zeros(), 9);
    
    let first_three = F::union(&[F::F0, F::F1, F::F2]);
    let last_three = F::union(&[F::F7, F::F8, F::F9]);
    assert_eq!(first_three.trailing_ones(), 3);
    assert_eq!(last_three.leading_ones(), 3);
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