mod lexer;
mod operators;
mod precedence;
mod token;

use std::{collections::HashMap, ops::Range};

pub use let_result::Result;

pub struct Parser<'a, I: Iterator, E> {
    lexer: lexer::Lexer<I>,
    token: Option<token::Token>,
    range: Range<usize>,
    emitter: &'a mut E,
    lable_id: usize,
    local_id: usize,
    locals: Vec<HashMap<Box<[u8]>, usize>>, // TODO: Optimize
}

impl<'a, I, E> Parser<'a, I, E>
where
    I: Iterator<Item = u8>,
    E: let_emitter::Emitter,
{
    pub fn new(iter: I, emitter: &'a mut E) -> Self {
        Self {
            lexer: iter.into(),
            token: None,
            range: 0..0,
            emitter,
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
            1 => precedence::get((buf[0], b' ', b' ')),
            2 => precedence::get((buf[0], buf[1], b' ')),
            3 => precedence::get((buf[0], buf[1], buf[2])),
            _ => 0,
        }
    }

    fn token_is(&self, token: token::Token) -> bool {
        if let Some(current) = self.token {
            if current == token {
                return true;
            }
        }
        false
    }

    fn token_is_buf(&self, token: token::Token, s: &[u8]) -> bool {
        if let Some(current) = self.token {
            if current == token && self.lexer.buffer() == s {
                return true;
            }
        }
        false
    }

    fn integer(&mut self) -> let_result::Result {
        self.emitter
            .integer(std::str::from_utf8(self.lexer.buffer())?.parse()?)?;
        self.next(); // Skip integer token.
        Ok(())
    }

    fn real(&mut self) -> let_result::Result {
        self.emitter
            .real(std::str::from_utf8(self.lexer.buffer())?.parse()?)?;
        self.next(); // Skip real token.
        Ok(())
    }

    fn paren(&mut self) -> let_result::Result {
        self.next(); // Skip '(' token.
        self.expression()?;
        if !self.token_is_buf(token::Token::Operator, &[b')']) {
            return let_result::raise!("Expected ')'");
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

    fn identifier(&mut self) -> let_result::Result {
        if let Some(index) = self.find_variable(self.lexer.buffer()) {
            self.emitter.load(index as u32)?;
        } else {
            self.emitter.pointer(self.lexer.buffer())?;
        }
        self.next(); // Skip identifier.

        if !self.token_is_buf(token::Token::Operator, &[b'(']) {
            return Ok(());
        }

        // Call.
        self.next(); // Skip '('.

        let mut arguments = 0;
        loop {
            if arguments >= u8::MAX as usize {
                return let_result::raise!("Reached maximum function argumens number");
            }

            if self.token_is_buf(token::Token::Operator, &[b')']) {
                break;
            }

            self.expression()?;
            arguments += 1;
        }

        self.next(); // Skip ')'.

        self.emitter.call(arguments as u8)
    }

    fn primary(&mut self) -> let_result::Result {
        match (self.token, self.lexer.buffer()) {
            (Some(token::Token::Identifier), b"if") => self.p_if(),
            (Some(token::Token::Identifier), _) => self.identifier(),
            (Some(token::Token::Integer), _) => self.integer(),
            (Some(token::Token::Real), _) => self.real(),
            (Some(token::Token::Operator), b"(") => self.paren(),
            _ => let_result::raise!("Unknown token."),
        }
    }

    fn binary(&mut self, precedence: u8) -> let_result::Result {
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

            self.emitter.binary(operator)?;

            let next_precedence = self.precedence();
            if current_precedence < next_precedence {
                return self.binary(current_precedence + 1);
            }
        }
    }

    fn block(&mut self, ends: &[&[u8]]) -> let_result::Result {
        let mut first = true;

        loop {
            let mut found = false;
            for &end in ends {
                if self.token_is_buf(token::Token::Identifier, end) {
                    found = true;
                }
            }

            if found {
                break;
            }

            if first {
                first = false;
            } else {
                self.emitter.drop()?;
            }

            self.expression()?;
        }

        Ok(())
    }

    fn expression(&mut self) -> let_result::Result {
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

    fn add_local(&mut self) -> let_result::Result {
        let index = self.get_local_id();
        if self
            .locals
            .last_mut()
            .unwrap()
            .insert(Vec::from(self.lexer.buffer()).into_boxed_slice(), index)
            .is_some()
        {
            return let_result::raise!("Variable \"{:?}\" already exists.", self.lexer.buffer());
        }
        Ok(())
    }

    fn p_if(&mut self) -> let_result::Result {
        let end_if_id = self.get_lable_id();
        self.next(); // Skip "if"
        
        // Condition
        self.expression()?;
        let mut next_id = self.get_lable_id();
        self.emitter.jump_false(next_id as u32)?;

        // Block.
        self.enter_block();
        self.block(&[b"end", b"else", b"elif"])?;
        self.exit_block();

        loop {
            if self.token_is_buf(token::Token::Identifier, b"end") {
                self.emitter.label(next_id as u32)?;
                self.emitter.label(end_if_id as u32)?;
                self.next(); // Skip "end"
                break;
            } else if self.token_is_buf(token::Token::Identifier, b"elif") {
                self.emitter.jump(end_if_id as u32)?;
                self.emitter.label(next_id as u32)?;
                self.next(); // Skip "elif"
                self.expression()?; // Condition.
                next_id = self.get_lable_id();
                self.emitter.jump_false(next_id as u32)?;
                // Block.
                self.enter_block();
                self.block(&[b"end", b"else", b"elif"])?;
                self.exit_block();
            } else if self.token_is_buf(token::Token::Identifier, b"else") {
                self.emitter.jump(end_if_id as u32)?;
                self.emitter.label(next_id as u32)?;
                // Block.
                self.enter_block();
                self.next(); // Skip "else"
                self.block(&[b"end"])?;
                self.exit_block();
                self.emitter.label(end_if_id as u32)?;
                self.next(); // Skip "end"
                break;
            }
        }

        Ok(())
    }

    fn prototype(&mut self) -> let_result::Result {
        if !self.token_is(token::Token::Identifier) {
            return let_result::raise!("Expected function name.");
        }

        self.emitter.label_named(self.lexer.buffer())?;
        self.next();

        if !self.token_is_buf(token::Token::Operator, &[b'(']) {
            return let_result::raise!("Expected '('.");
        }
        self.next();

        while self.token_is(token::Token::Identifier) {
            self.add_local()?;
            self.next();
        }

        if !self.token_is_buf(token::Token::Operator, &[b')']) {
            return let_result::raise!("Expected ')'.");
        }

        self.next();

        Ok(())
    }

    fn function(&mut self) -> let_result::Result {
        self.next(); // Skip "fn"

        self.enter_block();
        self.prototype()?;

        self.block(&[b"end"])?;
        self.next();

        self.emitter.ret()?;
        self.exit_block();

        Ok(())
    }

    fn global_code(&mut self) -> let_result::Result {
        self.emitter.label_named(b"__ctor__")?;
        self.expression()?;
        self.emitter.ret()
    }

    pub fn parse(&mut self) -> let_result::Result {
        self.next();
        loop {
            match (self.token, self.lexer.buffer()) {
                (None, _) => break,
                (Some(token::Token::Identifier), b"fn") => self.function()?,
                _ => self.global_code()?,
            }
        }
        Ok(())
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.range.clone()
    }
}
