use std::{collections::HashMap, ops::Range};

use crate::{
    compiler::Compiler,
    error::{raise, Error},
    lexer::Lexer,
    precedence::get_precedence,
    token::Token,
};

pub type ParserResult = Result<(), Error>;

pub struct Parser<I: Iterator, C> {
    lexer: Lexer<I>,
    token: Option<Token>,
    range: Range<usize>,
    compiler: C,
    lable_id: usize,
    local_id: usize,
    locals: Vec<HashMap<Box<[u8]>, usize>>, // TODO: Optimize
}

impl<I: Iterator<Item = u8>, C: Compiler> Parser<I, C> {
    pub fn new(iter: I, compiler: C) -> Self {
        Self {
            lexer: iter.into(),
            token: None,
            range: 0..0,
            compiler,
            lable_id: 0,
            local_id: 0,
            locals: Vec::new(),
        }
    }

    fn get_lable_id(&mut self) -> usize {
        let result = self.lable_id;
        self.lable_id += 1;
        result
    }

    fn get_local_id(&mut self) -> usize {
        let result = self.local_id;
        self.local_id += 1;
        result
    }

    fn next(&mut self) {
        self.lexer.skip_whitespaces();
        let start = self.lexer.offset();
        self.token = self.lexer.lex();
        let end = self.lexer.offset();
        self.range = start..end;
    }

    fn precedence(&self) -> u8 {
        let buf = self.lexer.buffer();
        match buf.len() {
            1 => get_precedence((buf[0], b' ', b' ')),
            2 => get_precedence((buf[0], buf[1], b' ')),
            3 => get_precedence((buf[0], buf[1], buf[2])),
            _ => 0,
        }
    }

    fn token_is(&self, token: Token) -> bool {
        if let Some(current) = self.token {
            if current == token {
                return true;
            }
        }
        false
    }

    fn token_is_buf(&self, token: Token, s: &[u8]) -> bool {
        if let Some(current) = self.token {
            if current == token && self.lexer.buffer() == s {
                return true;
            }
        }
        false
    }

    fn integer(&mut self) -> ParserResult {
        self.compiler
            .integer(std::str::from_utf8(self.lexer.buffer())?.parse()?)?;
        self.next(); // Skip integer token.
        Ok(())
    }

    fn real(&mut self) -> ParserResult {
        self.compiler
            .real(std::str::from_utf8(self.lexer.buffer())?.parse()?)?;
        self.next(); // Skip real token.
        Ok(())
    }

    fn paren(&mut self) -> ParserResult {
        self.next(); // Skip '(' token.
        self.expression()?;
        if !self.token_is_buf(Token::Operator, &[b')']) {
            return raise("Expected ')'".to_string());
        }
        self.next();
        Ok(())
    }

    fn find_variable(&self, name: &[u8]) -> Option<usize> {
        for block in self.locals.iter().rev() {
            if let Some(i) = block.get(name).cloned() {
                return Some(i);
            }
        }
        None
    }

    fn identifier(&mut self) -> ParserResult {
        if let Some(index) = self.find_variable(self.lexer.buffer()) {
            self.compiler.load(index as u64)?;
        } else {
            self.compiler.pointer(self.lexer.buffer())?;
        }
        self.next(); // Skip identifier.

        if !self.token_is_buf(Token::Operator, &[b'(']) {
            return Ok(());
        }

        // Call.
        self.next(); // Skip '('.

        let mut arguments = 0;
        loop {
            if arguments >= u8::MAX as usize {
                return raise("Reached maximum function argumens number".to_string())
            }

            self.expression()?;
            arguments += 1;

            if self.token_is_buf(Token::Operator, &[b')']) {
                break;
            }
        }

        self.next(); // Skip ')'.

        self.compiler.call(arguments as u8)
    }

    fn primary(&mut self) -> ParserResult {
        match (self.token, self.lexer.buffer()) {
            (Some(Token::Identifier), b"if") => self.p_if(),
            (Some(Token::Identifier), _) => self.identifier(),
            (Some(Token::Integer), _) => self.integer(),
            (Some(Token::Real), _) => self.real(),
            (Some(Token::Operator), b"(") => self.paren(),
            _ => raise("Unknown token.".to_string()),
        }
    }

    fn binary(&mut self, precedence: u8) -> ParserResult {
        loop {
            let current_precedence = self.precedence();

            if current_precedence < precedence {
                return Ok(());
            }

            let buf = self.lexer.buffer();
            let operator = match buf.len() {
                1 => [buf[0], b' ', b' '],
                2 => [buf[0], buf[1], b' '],
                3 => [buf[0], buf[1], buf[2]],
                _ => [b' ', b' ', b' '],
            };

            self.next(); // Skip operator.

            self.primary()?;

            self.compiler.binary(operator)?;

            let next_precedence = self.precedence();
            if current_precedence < next_precedence {
                return self.binary(current_precedence + 1);
            }
        }
    }

    fn expression(&mut self) -> ParserResult {
        self.primary()?;
        self.binary(1)
    }

    fn enter_block(&mut self) {
        self.locals.push(HashMap::new());
    }

    fn exit_block(&mut self) {
        let block_size = self.locals.last().unwrap().len();
        self.local_id -= block_size;
        self.locals.pop().unwrap();
    }

    fn add_local(&mut self) -> ParserResult {
        let index = self.get_local_id();
        if self
            .locals
            .last_mut()
            .unwrap()
            .insert(Vec::from(self.lexer.buffer()).into_boxed_slice(), index)
            .is_some()
        {
            return raise(format!("Variable \"{:?}\" already exists.", self.lexer.buffer()));
        }
        Ok(())
    }

    fn p_if(&mut self) -> ParserResult {
        self.next(); // Skip "if"

        let end_if_id = self.get_lable_id();

        // Condition
        self.expression()?;
        let mut next_id = self.get_lable_id();
        self.compiler.jump_false(next_id as u64)?;

        // Block.
        self.enter_block();
        self.expression()?;
        self.exit_block();

        loop {
            if self.token_is_buf(Token::Identifier, b"end") {
                self.next(); // Skip "end"
                self.compiler.lable(next_id as u64)?;
                self.compiler.lable(end_if_id as u64)?;
                break;
            } else if self.token_is_buf(Token::Identifier, b"elif") {
                self.next(); // Skip "elif"
                self.compiler.jump(end_if_id as u64)?;
                self.compiler.lable(next_id as u64)?;
                self.expression()?; // Condition.
                next_id = self.get_lable_id();
                self.compiler.jump_false(next_id as u64)?;
                // Block.
                self.enter_block();
                self.expression()?;
                self.exit_block();
            } else if self.token_is_buf(Token::Identifier, b"else") {
                self.next(); // Skip "else"
                self.compiler.jump(end_if_id as u64)?;
                self.compiler.lable(next_id as u64)?;
                // Block.
                self.enter_block();
                self.expression()?;
                self.exit_block();
                if !self.token_is_buf(Token::Identifier, b"end") {
                    return raise("Expected \"end\".".to_string());
                }
                self.next(); // Skip "end"
                self.compiler.lable(end_if_id as u64)?;
                break;
            }
        }

        Ok(())
    }

    fn prototype(&mut self) -> ParserResult {
        if !self.token_is(Token::Identifier) {
            return raise("Expected function name.".to_string());
        }

        self.compiler.lable_named(self.lexer.buffer())?;
        self.next();

        if !self.token_is_buf(Token::Operator, &[b'(']) {
            return raise("Expected '('.".to_string());
        }
        self.next();

        while self.token_is(Token::Identifier) {
            self.add_local()?;
            self.next();
        }

        if !self.token_is_buf(Token::Operator, &[b')']) {
            return raise("Expected ')'.".to_string());
        }

        self.next();

        Ok(())
    }

    fn function(&mut self) -> ParserResult {
        self.next(); // Skip "fn"

        self.enter_block();
        self.prototype()?;

        let mut first = true;

        while !self.token_is_buf(Token::Identifier, b"end") {
            if first {
                first = false;
            } else {
                self.compiler.end_of_statement()?;
            }
            self.expression()?;
        }

        if !self.token_is_buf(Token::Identifier, b"end") {
            return raise("Expected \"end\".".to_string());
        }

        self.next();

        self.compiler.ret()?;
        self.exit_block();

        Ok(())
    }

    fn global_code(&mut self) -> ParserResult {
        self.compiler.lable_named(b"__ctor__")?;
        self.expression()?;
        self.compiler.ret()
    }

    pub fn parse(&mut self) -> ParserResult {
        self.next();
        loop {
            match (self.token, self.lexer.buffer()) {
                (None, _) => break,
                (Some(Token::Identifier), b"fn") => self.function()?,
                _ => self.global_code()?,
            }
        }
        Ok(())
    }
}
