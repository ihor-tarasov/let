mod lexer;
mod operators;
mod precedence;
mod token;

use std::{collections::HashMap, ops::Range};

pub use let_result::Result;

struct Block {
    locals: HashMap<Box<[u8]>, u32>,
}

impl Block {
    fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }

    fn var(&mut self, name: &[u8], id: u32) -> bool {
        if let Some(local) = self.locals.get_mut(name) {
            *local = id;
            true
        } else {
            self.locals.insert(Vec::from(name).into_boxed_slice(), id);
            false
        }
    }

    fn get(&self, name: &[u8]) -> Option<u32> {
        self.locals.get(name).cloned()
    }

    fn len(&self) -> usize {
        self.locals.len()
    }
}

struct Function {
    blocks: Vec<Block>,
    local_counter: u32,
    stack_size: u32,
}

impl Function {
    fn new() -> Self {
        Self {
            blocks: vec![Block::new()],
            local_counter: 0,
            stack_size: 0,
        }
    }

    fn push(&mut self) {
        self.blocks.push(Block::new());
    }

    fn pop(&mut self) {
        let block = self.blocks.pop().unwrap();
        self.local_counter -= block.len() as u32;
    }

    fn var(&mut self, name: &[u8]) -> u32 {
        debug_assert!(!self.blocks.is_empty());
        let len = self.blocks.len();
        let id = self.local_counter;
        if !self.blocks[len - 1].var(name, id) {
            self.local_counter += 1;
            if self.local_counter > self.stack_size {
                self.stack_size = self.local_counter;
            }
        }
        id
    }

    fn get(&self, name: &[u8]) -> Option<u32> {
        for block in self.blocks.iter().rev() {
            if let Some(id) = block.get(name) {
                return Some(id);
            }
        }
        None
    }
}

pub struct Parser<'a, I: Iterator, E> {
    lexer: lexer::Lexer<I>,
    token: Option<token::Token>,
    range: Range<usize>,
    emitter: &'a mut E,
    lable_id: usize,
    functions: Vec<Function>,
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
            functions: vec![Function::new()],
        }
    }

    fn get_lable_id(&mut self) -> usize {
        let result = self.lable_id;
        self.lable_id += 1;
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

    fn find_variable(&self, name: &[u8]) -> Option<u32> {
        self.functions.last().unwrap().get(name)
    }

    fn identifier(&mut self) -> let_result::Result {
        if let Some(index) = self.find_variable(self.lexer.buffer()) {
            self.emitter.load(index);
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
            if arguments > u8::MAX as u32 {
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
            (Some(token::Token::Identifier), b"let") => self.p_let(),
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
        self.functions.last_mut().unwrap().push();
    }

    fn exit_block(&mut self) {
        self.functions.last_mut().unwrap().pop();
    }

    fn add_local(&mut self) -> u32 {
        self.functions.last_mut().unwrap().var(self.lexer.buffer())
    }

    fn p_let(&mut self) -> let_result::Result {
        self.next(); // Skip "let"

        if !self.token_is(token::Token::Identifier) {
            return let_result::raise!("Expected variable name.");
        }

        let local_id = self.add_local();
        self.next(); // Skip variable name.

        if !self.token_is_buf(token::Token::Operator, &[b'=']) {
            return let_result::raise!("Expected '='.");
        }
        self.next(); // Skip '='

        self.expression()?;
        self.emitter.store(local_id);

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

    fn function(&mut self) -> let_result::Result {
        self.next(); // Skip "fn"

        if !self.token_is(token::Token::Identifier) {
            return let_result::raise!("Expected function name.");
        }

        self.emitter.label_named(self.lexer.buffer())?;
        self.next(); // Skip function name.

        if !self.token_is_buf(token::Token::Operator, &[b'(']) {
            return let_result::raise!("Expected '('.");
        }
        self.next(); // Skip '('.

        self.functions.push(Function::new());

        let mut args_count = 0;
        while self.token_is(token::Token::Identifier) {
            if args_count > u8::MAX as u32 {
                return let_result::raise!("Reached maximum function argumens number");
            }
            self.add_local();
            self.next();
            args_count += 1;
        }

        if !self.token_is_buf(token::Token::Operator, &[b')']) {
            return let_result::raise!("Expected ')'.");
        }
        self.next(); // Skip ')'.

        let stack_size_address = self.emitter.function(args_count as u8)?;

        self.block(&[b"end"])?;
        self.next(); // Skip "end".

        self.emitter.ret()?;

        let function = self.functions.pop().unwrap();

        (function.stack_size - args_count)
            .to_be_bytes()
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(i, b)| self.emitter.set(stack_size_address + i as u32, b));

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
