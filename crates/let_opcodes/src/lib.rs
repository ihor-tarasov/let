macro_rules! impl_opcodes {
    ($($name:ident:$value:literal)*) => {
        $(pub const $name: u8 = $value;)*
    };
}

impl_opcodes!(
    // 0x00..=0x2F 1 byte opcodes
    RET: 0x00
    DROP: 0x01

    // Operators opcodes is in range 0x10..=0x1F
    LS: 0x10
    GR: 0x11
    EQ: 0x12
    ADD: 0x13
    LE: 0x14
    SUB: 0x15
    MUL: 0x16

    // 0x30..=0x4F 2 bytes opcodes
    LD1: 0x30
    INT1: 0x31
    CALL: 0x32

    // 0x50..=0x6F 3 bytes opcodes
    INT2: 0x50
    LD2: 0x51

    // 0x70..=0x8F 5 byte opcodes
    JPF: 0x70
    JP: 0x71
    PTR: 0x72

    // 0x90..=0xFF 9 byte opcodes
    INT8: 0x90
    REAL: 0x91
);
