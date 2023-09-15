use core::fmt;

use crate::compiler::{Compiler, CompilerResult};

pub struct ToAssemblerCompiler<W>(W);

impl<W> From<W> for ToAssemblerCompiler<W> {
    fn from(value: W) -> Self {
        Self(value)
    }
}

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

impl<W: std::io::Write> Compiler for ToAssemblerCompiler<W> {
    fn integer(&mut self, value: u64) -> CompilerResult {
        writeln!(self.0, "\tINT {value}")?;
        Ok(())
    }

    fn real(&mut self, value: f64) -> CompilerResult {
        writeln!(self.0, "\tREAL {value}")?;
        Ok(())
    }

    fn call(&mut self, arguments: u8) -> CompilerResult {
        writeln!(self.0, "\tCALL {arguments}")?;
        Ok(())
    }

    fn binary(&mut self, operator: [u8; 3]) -> CompilerResult {
        writeln!(self.0, "\tOP {}", Slice(&operator))?;
        Ok(())
    }

    fn end_of_statement(&mut self) -> CompilerResult {
        writeln!(self.0, "\tDROP")?;
        Ok(())
    }

    fn lable(&mut self, id: u64) -> CompilerResult {
        writeln!(self.0, "@lbl_{id}:")?;
        Ok(())
    }

    fn lable_named(&mut self, lable: &[u8]) -> CompilerResult {
        writeln!(self.0, "{}:", Slice(lable))?;
        Ok(())
    }

    fn jump(&mut self, id: u64) -> CompilerResult {
        writeln!(self.0, "\tJP @lbl_{id}")?;
        Ok(())
    }

    fn jump_name(&mut self, name: &[u8]) -> CompilerResult {
        writeln!(self.0, "\tJP {}", Slice(name))?;
        Ok(())
    }

    fn jump_false(&mut self, id: u64) -> CompilerResult {
        writeln!(self.0, "\tJPF @lbl_{id}")?;
        Ok(())
    }

    fn jump_false_name(&mut self, name: &[u8]) -> CompilerResult {
        writeln!(self.0, "\tJPF {}", Slice(name))?;
        Ok(())
    }

    fn load(&mut self, index: u64) -> CompilerResult {
        writeln!(self.0, "\tLD {}", index)?;
        Ok(())
    }

    fn pointer(&mut self, name: &[u8]) -> CompilerResult {
        writeln!(self.0, "\tPTR {}", Slice(name))?;
        Ok(())
    }

    fn ret(&mut self) -> CompilerResult {
        writeln!(self.0, "\tRET")?;
        Ok(())
    }
}
