macro_rules! impl_opcodes {
    ($($name:ident:$value:literal)*) => {
        $(pub const $name: u8 = $value;)*
    };
}

impl_opcodes!(
    // 1 byte opcodes
    RET: 0x00

    // Operators opcodes is in range 0x10..=0x1F
    LS: 0x10
    GR: 0x11
    EQ: 0x12

    // 2 bytes opcodes
    LD1: 0x30
    INT1: 0x31
    CALL: 0x32

    // 8 bytes opcodes
    JPF: 0x50
    JP: 0x51
);
