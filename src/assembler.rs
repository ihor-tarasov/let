use crate::{error::{Error, raise}, compiler::Compiler};

pub type AssemblerResult = Result<(), Error>;

struct Assembler<C> {
    compiler: C,
}

impl<C> Assembler<C>
where
    C: Compiler,
{
    pub fn new(compiler: C) -> Self {
        Self {
            compiler,
        }
    }

    pub fn assemble(&mut self, mut line: &str) -> AssemblerResult {
        line = line.trim();

        // Handle lables
        while let Some(index) = line.find(':') {
            self.compiler.lable_named(line[0..index].trim().as_bytes())?;
            line = &line[(index + 1)..];

            if line.is_empty() {
                return Ok(())
            }
        }

        if line.starts_with("LD ") {
            line = line[3..].trim();
            self.compiler.load(line.parse::<u64>()?)
        } else if line.starts_with("PTR ") {
            line = line[3..].trim();
            self.compiler.pointer(line.as_bytes())
        } else if line.starts_with("INT ") {
            line = line[4..].trim();
            self.compiler.integer(line.parse::<u64>()?)
        } else if line.starts_with("OP ") {
            line = line[3..].trim();
            if line.len() > 3 || line.is_empty() {
                return raise("Wrong operator triplet".to_string())
            }
            let operator = [
                line.as_bytes().get(0).cloned().unwrap_or(b' '),
                line.as_bytes().get(1).cloned().unwrap_or(b' '),
                line.as_bytes().get(2).cloned().unwrap_or(b' '),
            ];
            self.compiler.binary(operator)
        } else if line.starts_with("JPF ") {
            line = line[4..].trim();
            self.compiler.jump_false_name(line.as_bytes())
        } else if line.starts_with("JP ") {
            line = line[3..].trim();
            self.compiler.jump_name(line.as_bytes())
        } else if line == "RET" {
            self.compiler.ret()
        } else if line.starts_with("CALL ") {
            line = line[5..].trim();
            self.compiler.call(line.parse::<u8>()?)
        } else {
            raise(format!("Unexpected line \"{}\"", line))
        }
    }
}

pub fn assemble<R, C>(mut read: R, compiler: C) -> AssemblerResult
where
    R: std::io::BufRead,
    C: Compiler,
{
    let mut assembler = Assembler::new(compiler);
    let mut line = String::new();
    loop {
        line.clear();
        if read.read_line(&mut line)? == 0 {
            break Ok(());
        }
        assembler.assemble(line.as_str())?;
    }
}
