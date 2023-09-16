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
    ADD: 0x13
    LE: 0x14
    SUB: 0x15
    MUL: 0x16

    // 2 bytes opcodes
    LD1: 0x30
    INT1: 0x31
    CALL: 0x32

    // 4 bytes opcodes
    INT3: 0x50
    LD3: 0x51

    // 9 byte opcodes
    JPF: 0x70
    JP: 0x71
    PTR: 0x72
    INT8: 0x90
    REAL: 0x91
);