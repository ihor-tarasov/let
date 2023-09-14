const SINGLE_CHARS: [u8; 23] = [
    b'!', b'"', b'%', b'&', b'\'', b'(', b')', b'*', b'+', b'-', b'.', b'/', b':', b'<', b'=',
    b'>', b'?', b'[', b']', b'^', b'{', b'|', b'}',
];

pub fn is_single_operator(c: u8) -> bool {
    SINGLE_CHARS.binary_search(&c).is_ok()
}

const DOUBLE_CHARS: [(u8, u8); 15] = [
    (b'!', b'='),
    (b'%', b'='),
    (b'&', b'&'),
    (b'&', b'='),
    (b'*', b'='),
    (b'+', b'='),
    (b'-', b'='),
    (b'/', b'='),
    (b'<', b'<'),
    (b'<', b'='),
    (b'=', b'='),
    (b'>', b'='),
    (b'^', b'='),
    (b'|', b'='),
    (b'|', b'|'),
];

pub fn is_double_operator(c0: u8, c1: u8) -> bool {
    DOUBLE_CHARS.binary_search(&(c0, c1)).is_ok()
}

const TRIPLE_CHARS: [(u8, u8, u8); 2] = [(b'<', b'<', b'='), (b'>', b'>', b'=')];

pub fn is_triple_operator(c0: u8, c1: u8, c2: u8) -> bool {
    TRIPLE_CHARS.binary_search(&(c0, c1, c2)).is_ok()
}
