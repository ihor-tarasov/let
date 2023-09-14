#[derive(Clone, Copy, PartialEq)]
pub enum Token {
    Identifier,
    Real,
    Integer,
    Operator,
    Unknown,
}
