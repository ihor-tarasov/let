const ASSIGNMENT_PRECEDENCE: u8 = 2;
const OR_PRECEDENCE: u8 = 3;
const AND_PRECEDENCE: u8 = 4;
const COMPARISON_PRECEDENCE: u8 = 5;
const BITWISE_XOR_PRECEDENCE: u8 = 6;
const BITWISE_OR_PRECEDENCE: u8 = 7;
const BITWISE_AND_PRECEDENCE: u8 = 8;
const SHIFTS_PRECEDENCE: u8 = 9;
const TERM_PRECEDENCE: u8 = 10;
const FACTOR_PRECEDENCE: u8 = 11;

const PRECEDENCES: [(u8, u8, u8, u8); 29] = [
    (b'!', b'=', b' ', COMPARISON_PRECEDENCE),
    (b'%', b' ', b' ', FACTOR_PRECEDENCE),
    (b'%', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'&', b' ', b' ', BITWISE_AND_PRECEDENCE),
    (b'&', b'&', b' ', AND_PRECEDENCE),
    (b'&', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'*', b' ', b' ', FACTOR_PRECEDENCE),
    (b'*', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'+', b' ', b' ', TERM_PRECEDENCE),
    (b'+', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'-', b' ', b' ', TERM_PRECEDENCE),
    (b'-', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'/', b' ', b' ', FACTOR_PRECEDENCE),
    (b'/', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'<', b' ', b' ', COMPARISON_PRECEDENCE),
    (b'<', b'<', b' ', SHIFTS_PRECEDENCE),
    (b'<', b'<', b'=', ASSIGNMENT_PRECEDENCE),
    (b'<', b'=', b' ', COMPARISON_PRECEDENCE),
    (b'=', b' ', b' ', ASSIGNMENT_PRECEDENCE),
    (b'=', b'=', b' ', COMPARISON_PRECEDENCE),
    (b'>', b' ', b' ', COMPARISON_PRECEDENCE),
    (b'>', b'=', b' ', COMPARISON_PRECEDENCE),
    (b'>', b'>', b' ', SHIFTS_PRECEDENCE),
    (b'>', b'>', b'=', ASSIGNMENT_PRECEDENCE),
    (b'^', b' ', b' ', BITWISE_XOR_PRECEDENCE),
    (b'^', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'|', b' ', b' ', BITWISE_OR_PRECEDENCE),
    (b'|', b'=', b' ', ASSIGNMENT_PRECEDENCE),
    (b'|', b'|', b' ', OR_PRECEDENCE),
];

pub fn get(c: (u8, u8, u8)) -> u8 {
    match PRECEDENCES.binary_search_by_key(&c, |&c| (c.0, c.1, c.2)) {
        Ok(index) => PRECEDENCES[index].3,
        Err(_) => 0,
    }
}
