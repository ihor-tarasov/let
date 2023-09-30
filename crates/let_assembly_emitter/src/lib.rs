use core::fmt;

pub struct AssemblyEmitter<W>(W);

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

impl<W: std::io::Write> AssemblyEmitter<W> {
    pub fn new(write: W) -> Self {
        Self(write)
    }
}

impl<W: std::io::Write> let_emitter::Emitter for AssemblyEmitter<W> {
    fn integer(&mut self, value: u64) -> let_emitter::Result {
        writeln!(self.0, "\tINT {value}")?;
        Ok(())
    }

    fn real(&mut self, value: f64) -> let_emitter::Result {
        writeln!(self.0, "\tREAL {value}")?;
        Ok(())
    }

    fn call(&mut self, arguments: u8) -> let_emitter::Result {
        writeln!(self.0, "\tCALL {arguments}")?;
        Ok(())
    }

    fn binary(&mut self, operator: [u8; 3]) -> let_emitter::Result {
        writeln!(self.0, "\tOP {}", Slice(&operator))?;
        Ok(())
    }

    fn drop(&mut self) -> let_emitter::Result {
        writeln!(self.0, "\tDROP")?;
        Ok(())
    }

    fn label(&mut self, id: u32) -> let_emitter::Result {
        writeln!(self.0, "@lbl_{id}:")?;
        Ok(())
    }

    fn label_named(&mut self, lable: &[u8]) -> let_emitter::Result {
        writeln!(self.0, "{}:", Slice(lable))?;
        Ok(())
    }

    fn jump(&mut self, id: u32) -> let_emitter::Result {
        writeln!(self.0, "\tJP @lbl_{id}")?;
        Ok(())
    }

    fn jump_name(&mut self, name: &[u8]) -> let_emitter::Result {
        writeln!(self.0, "\tJP {}", Slice(name))?;
        Ok(())
    }

    fn jump_false(&mut self, id: u32) -> let_emitter::Result {
        writeln!(self.0, "\tJPF @lbl_{id}")?;
        Ok(())
    }

    fn jump_false_name(&mut self, name: &[u8]) -> let_emitter::Result {
        writeln!(self.0, "\tJPF {}", Slice(name))?;
        Ok(())
    }

    fn load(&mut self, index: u32) -> let_emitter::Result {
        writeln!(self.0, "\tLD {}", index)?;
        Ok(())
    }

    fn pointer(&mut self, name: &[u8]) -> let_emitter::Result {
        writeln!(self.0, "\tPTR {}", Slice(name))?;
        Ok(())
    }

    fn ret(&mut self) -> let_emitter::Result {
        writeln!(self.0, "\tRET")?;
        Ok(())
    }
}

pub fn open(path: &str) -> let_result::Result<impl let_emitter::Emitter> {
    match std::fs::File::create(path) {
        Ok(file) => Ok(AssemblyEmitter::new(std::io::BufWriter::new(file))),
        Err(error) => let_result::raise!("Unable to create file \"{path}\", error: {error}"),
    }
}
