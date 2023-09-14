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
    (b'!', b'=', b'\0', COMPARISON_PRECEDENCE),
    (b'%', b'\0', b'\0', FACTOR_PRECEDENCE),
    (b'%', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'&', b'\0', b'\0', BITWISE_AND_PRECEDENCE),
    (b'&', b'&', b'\0', AND_PRECEDENCE),
    (b'&', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'*', b'\0', b'\0', FACTOR_PRECEDENCE),
    (b'*', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'+', b'\0', b'\0', TERM_PRECEDENCE),
    (b'+', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'-', b'\0', b'\0', TERM_PRECEDENCE),
    (b'-', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'/', b'\0', b'\0', FACTOR_PRECEDENCE),
    (b'/', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'<', b'\0', b'\0', COMPARISON_PRECEDENCE),
    (b'<', b'<', b'\0', SHIFTS_PRECEDENCE),
    (b'<', b'<', b'=', ASSIGNMENT_PRECEDENCE),
    (b'<', b'=', b'\0', COMPARISON_PRECEDENCE),
    (b'=', b'\0', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'=', b'=', b'\0', COMPARISON_PRECEDENCE),
    (b'>', b'\0', b'\0', COMPARISON_PRECEDENCE),
    (b'>', b'=', b'\0', COMPARISON_PRECEDENCE),
    (b'>', b'>', b'\0', SHIFTS_PRECEDENCE),
    (b'>', b'>', b'=', ASSIGNMENT_PRECEDENCE),
    (b'^', b'\0', b'\0', BITWISE_XOR_PRECEDENCE),
    (b'^', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'|', b'\0', b'\0', BITWISE_OR_PRECEDENCE),
    (b'|', b'=', b'\0', ASSIGNMENT_PRECEDENCE),
    (b'|', b'|', b'\0', OR_PRECEDENCE),
];

pub fn get_precedence(c: (u8, u8, u8)) -> u8 {
    match PRECEDENCES.binary_search_by_key(&c, |&c| (c.0, c.1, c.2)) {
        Ok(index) => PRECEDENCES[index].3,
        Err(_) => 0,
    }
}
