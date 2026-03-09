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
                    | FLAG1
                ]
            ]
            - FLAG0
        ]
    }
);

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

#[inline]
#[must_use]
pub const fn perms<const LEN: usize>(perms: [Perms; LEN]) -> Perms {
    Perms::union(&perms)
}

macro_rules! perms {
    ($perm:ident) => {
        Perms::$perm
    };
    ($($perm:ident),+$(,)?) => {
        Perms::union(&[
            $(
                Perms::$perm,
            )*
        ])
    };
}

#[test]
fn test_perms() {
    let ban1 = perms([
        Perms::BAN_USER,
        Perms::UNBAN_USER,
    ]);
    let ban = perms!(BAN_USER, UNBAN_USER);
    assert_eq!(ban, ban1);
    assert!(Perms::OWNER.has_all(ban));
    assert!(Perms::OWNER.has_all(Perms::MOD));
    assert!(Perms::OWNER.has_all(Perms::ADMIN));
    assert!(Perms::OWNER.has_all(Perms::SUPER));
    assert!(Perms::OWNER.has_all(Perms::USER));
    assert!(Perms::OWNER.has_all(Perms::GUEST));
    assert_eq!(ban.count_ones(), 2);
}

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
            GROUP: [
                F10
                F11
                SUBGROUP: [
                    F12
                    F13
                ]
                F14
            ]
        }
    );
    //   __
    // 0b001
    assert_eq!(F::F0.trailing_zeros(), 0);
    assert_eq!(F::F1.trailing_zeros(), 1);
    assert_eq!(F::F2.trailing_zeros(), 2);
    assert_eq!(F::F3.trailing_zeros(), 3);
    assert_eq!(F::F4.trailing_zeros(), 4);
    
    assert_eq!(F::F14.leading_zeros(), 0);
    assert_eq!(F::F13.leading_zeros(), 1);
    assert_eq!(F::F12.leading_zeros(), 2);
    assert_eq!(F::F11.leading_zeros(), 3);
    assert_eq!(F::F10.leading_zeros(), 4);
    assert_eq!(F::F9.leading_zeros(), 5);
    assert_eq!(F::F8.leading_zeros(), 6);
    assert_eq!(F::F7.leading_zeros(), 7);
    assert_eq!(F::F6.leading_zeros(), 8);
    assert_eq!(F::F5.leading_zeros(), 9);
    assert_eq!(F::F4.leading_zeros(), 10);
    
    let first_three = F::union(&[F::F0, F::F1, F::F2]);
    let last_three = F::union(&[F::F12, F::F13, F::F14]);
    assert_eq!(first_three.trailing_ones(), 3);
    assert_eq!(last_three.leading_ones(), 3);
    assert_eq!(F::GROUP.count_ones(), 5);
    
    let flags = F::NONE
        .with_if(F::F0, true)
        .with_if(F::F1, true)
        .with_if(F::F2, true);
    assert!(flags.eq(F::union(&[
        F::F0,
        F::F1,
        F::F2,
    ])));
    
    let flags = F::union(&[F::F0, F::F1, F::F3, F::F7, F::F9]);
    let mut work_counter = 0usize;
    // 1000 iterations to ensure that nothing wonky happens.
    // something wonky might happen on the 1001st iteration,
    // but I can't keep testing forever.
    for _ in 0..1000 {
        { // be_bytes
            let bytes = flags.to_be_bytes();
            assert_eq!(bytes.len(), size_of::<F>());
            let ret_flags = F::from_be_bytes(bytes);
            assert_eq!(flags, ret_flags);
        }
        { // le_bytes
            let bytes = flags.to_le_bytes();
            assert_eq!(bytes.len(), size_of::<F>());
            let ret_flags = F::from_le_bytes(bytes);
            assert_eq!(flags, ret_flags);
        }
        { // ne_bytes
            let bytes = flags.to_ne_bytes();
            assert_eq!(bytes.len(), size_of::<F>());
            let ret_flags = F::from_ne_bytes(bytes);
            assert_eq!(flags, ret_flags);
        }
        work_counter += 1;
    }
    assert_eq!(work_counter, 1000);
}

#[test]
fn test_to_and_from_bytes() {
    flags!{
        struct F(pub [u16]);
        override {
            /// test
            pub new
        }
        const {
            F0
            FIRST: [+F0]
            F1
            F2
            F3
            F4
            F5
            F6
            F7
            F8
            F9
            F10
            F11
            F12
            F13
            F14
            F15
            F16
            F17
            F18
            F19
            F20
            F21
            F22
            F23
            F24
            F25
            F26
            F27
            F28
            F29
            F30
            F31
            LAST: [+F31]
        }
    }
    _=F::new();
    // 0101 1010 1110 0111
    let flags = F::union(&[
        
    ]);
}

#[test]
fn test_flag_updates_and_groups() {
    {
        flags!{
            struct F(pub [u64]);
            const {
                F0
                F1
                F2
            }
        }
    }
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