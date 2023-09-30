use crate::{
    operators::{is_double_operator, is_single_operator, is_triple_operator},
    token::Token,
};

pub struct Lexer<I: Iterator> {
    iter: std::iter::Peekable<I>,
    offset: usize,
    buffer: Vec<u8>,
}

impl<I: Iterator> From<I> for Lexer<I> {
    fn from(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            offset: 0,
            buffer: Vec::new(),
        }
    }
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    fn current(&mut self) -> Option<u8> {
        self.iter.peek().cloned()
    }

    fn next(&mut self) -> Option<u8> {
        self.iter.next().map(|c| {
            self.offset += 1;
            c
        })
    }

    pub fn skip_whitespaces(&mut self) {
        while let Some(c) = self.current() {
            if c.is_ascii_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn number(&mut self, c: u8) -> Token {
        let mut has_dot = false;
        self.buffer.clear();
        self.buffer.push(c);
        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                self.buffer.push(c);
                self.next();
            } else if c == b'.' {
                if has_dot {
                    break;
                } else {
                    has_dot = true;
                }
                self.buffer.push(c);
                self.next();
            } else {
                break;
            }
        }
        if has_dot {
            Token::Real
        } else {
            Token::Integer
        }
    }

    fn identifier(&mut self, c: u8) -> Token {
        self.buffer.clear();
        self.buffer.push(c);
        while let Some(c) = self.current() {
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' {
                self.buffer.push(c);
                self.next();
            } else {
                break;
            }
        }
        Token::Identifier
    }

    fn operator(&mut self, c0: u8) -> Token {
        self.buffer.clear();
        self.buffer.push(c0);
        if let Some(c1) = self.current() {
            if is_double_operator(c0, c1) {
                self.buffer.push(c1);
                self.next();
                if let Some(c2) = self.current() {
                    if is_triple_operator(c0, c1, c2) {
                        self.next();
                        self.buffer.push(c2);
                    }
                }
            }
        }
        Token::Operator
    }

    pub fn lex(&mut self) -> Option<Token> {
        let c = self.next()?;

        Some(if c.is_ascii_digit() {
            self.number(c)
        } else if c.is_ascii_alphanumeric() || c == b'_' {
            self.identifier(c)
        } else if is_single_operator(c) {
            self.operator(c)
        } else {
            self.buffer.clear();
            self.buffer.push(c);
            Token::Unknown
        })
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}
