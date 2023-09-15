use std::fmt;

use crate::{
    compiler::{Compiler, CompilerResult},
    opcodes,
};

struct Slice<'a>(&'a [u8]);

impl<'a> fmt::Display for Slice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.iter().cloned() {
            if c != b' ' {
                write!(f, "{}", c as char)?;
            }
        }
        Ok(())
    }
}

pub struct ToObjectCompiler<O, M> {
    object_write: O,
    metadata_write: M,
    counter: u64,
}

impl<O, M> ToObjectCompiler<O, M> {
    pub fn new(object: O, metadata: M) -> Self {
        Self {
            object_write: object,
            metadata_write: metadata,
            counter: 0,
        }
    }
}

impl<O, M> Compiler for ToObjectCompiler<O, M>
where
    O: std::io::Write,
    M: std::io::Write,
{
    fn integer(&mut self, value: u64) -> CompilerResult {
        if value <= u8::MAX as u64 {
            self.object_write.write_all(&[opcodes::INT1, value as u8])?;
            self.counter += 2;
        } else if value <= 0xFFFFFF {
            self.object_write.write_all(&[
                opcodes::INT3,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
            self.counter += 4;
        } else {
            self.object_write.write_all(&[opcodes::INT8])?;
            self.object_write.write_all(&value.to_be_bytes())?;
            self.counter += 9;
        }
        Ok(())
    }

    fn real(&mut self, value: f64) -> CompilerResult {
        self.object_write.write_all(&[opcodes::REAL])?;
        self.object_write.write_all(&value.to_be_bytes())?;
        self.counter += 9;
        Ok(())
    }

    fn call(&mut self, arguments: u8) -> crate::compiler::CompilerResult {
        self.object_write.write_all(&[opcodes::CALL, arguments])?;
        self.counter += 2;
        Ok(())
    }

    fn binary(&mut self, operator: [u8; 3]) -> crate::compiler::CompilerResult {
        let opcode = match &operator {
            b"+  " => opcodes::ADD,
            b"<  " => opcodes::LS,
            b">  " => opcodes::GR,
            b"== " => opcodes::EQ,
            b"<= " => opcodes::LE,
            b"-  " => opcodes::SUB,
            b"*  " => opcodes::MUL,
            _ => panic!("Unknown operator {operator:?}"),
        };
        self.object_write.write_all(&[opcode])?;
        self.counter += 1;
        Ok(())
    }

    fn end_of_statement(&mut self) -> crate::compiler::CompilerResult {
        todo!()
    }

    fn lable(&mut self, id: u64) -> crate::compiler::CompilerResult {
        writeln!(self.metadata_write, "lable @lbl_{id} {}", self.counter)?;
        Ok(())
    }

    fn lable_named(&mut self, lable: &[u8]) -> crate::compiler::CompilerResult {
        writeln!(self.metadata_write, "lable {} {}", Slice(lable), self.counter)?;
        Ok(())
    }

    fn jump(&mut self, id: u64) -> crate::compiler::CompilerResult {
        self.object_write
            .write_all(&[opcodes::JP, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(self.metadata_write, "link @lbl_{id} {}", self.counter + 1)?;
        self.counter += 9;
        Ok(())
    }

    fn jump_name(&mut self, name: &[u8]) -> crate::compiler::CompilerResult {
        self.object_write
            .write_all(&[opcodes::JP, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(
            self.metadata_write,
            "link {} {}",
            Slice(name),
            self.counter + 1
        )?;
        self.counter += 9;
        Ok(())
    }

    fn jump_false(&mut self, id: u64) -> crate::compiler::CompilerResult {
        self.object_write
            .write_all(&[opcodes::JPF, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(self.metadata_write, "link @lbl_{id} {}", self.counter + 1)?;
        self.counter += 9;
        Ok(())
    }

    fn jump_false_name(&mut self, name: &[u8]) -> crate::compiler::CompilerResult {
        self.object_write
            .write_all(&[opcodes::JPF, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(
            self.metadata_write,
            "link {} {}",
            Slice(name),
            self.counter + 1
        )?;
        self.counter += 9;
        Ok(())
    }

    fn load(&mut self, index: u64) -> CompilerResult {
        if index <= u8::MAX as u64 {
            self.object_write.write_all(&[opcodes::LD1, index as u8])?;
            self.counter += 2;
        } else if index <= 0xFFFFFF {
            self.object_write.write_all(&[
                opcodes::LD3,
                (index >> 16) as u8,
                (index >> 8) as u8,
                index as u8,
            ])?;
            self.counter += 4;
        } else {
            self.object_write.write_all(&[opcodes::INT8])?;
            self.object_write.write_all(&index.to_be_bytes())?;
            self.counter += 9;
        }
        Ok(())
    }

    fn pointer(&mut self, name: &[u8]) -> CompilerResult {
        self.object_write
            .write_all(&[opcodes::PTR, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(
            self.metadata_write,
            "link {} {}",
            Slice(name),
            self.counter + 1
        )?;
        self.counter += 9;
        Ok(())
    }

    fn ret(&mut self) -> CompilerResult {
        self.object_write.write_all(&[opcodes::RET])?;
        Ok(())
    }
}
