use std::fmt;

pub use let_emitter::Emitter;

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

pub struct ObjectEmitter<O, L, B> {
    object_write: O,
    links_write: L,
    labels_write: B,
    counter: u64,
}

impl<O, L, B> ObjectEmitter<O, L, B> {
    pub fn new(object: O, links: L, labels: B) -> Self {
        Self {
            object_write: object,
            links_write: links,
            labels_write: labels,
            counter: 0,
        }
    }
}

impl<O, L, B> let_emitter::Emitter for ObjectEmitter<O, L, B>
where
    O: std::io::Write,
    L: std::io::Write,
    B: std::io::Write,
{
    fn integer(&mut self, value: u64) -> let_emitter::Result {
        if value <= u8::MAX as u64 {
            self.object_write
                .write_all(&[let_opcodes::INT1, value as u8])?;
            self.counter += 2;
        } else if value <= 0xFFFFFF {
            self.object_write.write_all(&[
                let_opcodes::INT3,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
            self.counter += 4;
        } else {
            self.object_write.write_all(&[let_opcodes::INT8])?;
            self.object_write.write_all(&value.to_be_bytes())?;
            self.counter += 9;
        }
        Ok(())
    }

    fn real(&mut self, value: f64) -> let_emitter::Result {
        self.object_write.write_all(&[let_opcodes::REAL])?;
        self.object_write.write_all(&value.to_be_bytes())?;
        self.counter += 9;
        Ok(())
    }

    fn call(&mut self, arguments: u8) -> let_emitter::Result {
        self.object_write
            .write_all(&[let_opcodes::CALL, arguments])?;
        self.counter += 2;
        Ok(())
    }

    fn binary(&mut self, operator: [u8; 3]) -> let_emitter::Result {
        let opcode = match &operator {
            b"+  " => let_opcodes::ADD,
            b"<  " => let_opcodes::LS,
            b">  " => let_opcodes::GR,
            b"== " => let_opcodes::EQ,
            b"<= " => let_opcodes::LE,
            b"-  " => let_opcodes::SUB,
            b"*  " => let_opcodes::MUL,
            _ => panic!("Unknown operator {operator:?}"),
        };
        self.object_write.write_all(&[opcode])?;
        self.counter += 1;
        Ok(())
    }

    fn drop(&mut self) -> let_emitter::Result {
        self.object_write.write_all(&[let_opcodes::DROP])?;
        Ok(())
    }

    fn label(&mut self, id: u64) -> let_emitter::Result {
        writeln!(self.labels_write, "@lbl_{id} {}", self.counter)?;
        Ok(())
    }

    fn label_named(&mut self, lable: &[u8]) -> let_emitter::Result {
        writeln!(
            self.labels_write,
            "{} {}",
            Slice(lable),
            self.counter
        )?;
        Ok(())
    }

    fn jump(&mut self, id: u64) -> let_emitter::Result {
        self.object_write
            .write_all(&[let_opcodes::JP, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(self.links_write, "@lbl_{id} {}", self.counter + 1)?;
        self.counter += 9;
        Ok(())
    }

    fn jump_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.object_write
            .write_all(&[let_opcodes::JP, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(
            self.links_write,
            "{} {}",
            Slice(name),
            self.counter + 1
        )?;
        self.counter += 9;
        Ok(())
    }

    fn jump_false(&mut self, id: u64) -> let_emitter::Result {
        self.object_write
            .write_all(&[let_opcodes::JPF, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(self.links_write, "@lbl_{id} {}", self.counter + 1)?;
        self.counter += 9;
        Ok(())
    }

    fn jump_false_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.object_write
            .write_all(&[let_opcodes::JPF, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(
            self.links_write,
            "{} {}",
            Slice(name),
            self.counter + 1
        )?;
        self.counter += 9;
        Ok(())
    }

    fn load(&mut self, index: u64) -> let_emitter::Result {
        if index <= u8::MAX as u64 {
            self.object_write
                .write_all(&[let_opcodes::LD1, index as u8])?;
            self.counter += 2;
        } else if index <= 0xFFFFFF {
            self.object_write.write_all(&[
                let_opcodes::LD3,
                (index >> 16) as u8,
                (index >> 8) as u8,
                index as u8,
            ])?;
            self.counter += 4;
        } else {
            self.object_write.write_all(&[let_opcodes::INT8])?;
            self.object_write.write_all(&index.to_be_bytes())?;
            self.counter += 9;
        }
        Ok(())
    }

    fn pointer(&mut self, name: &[u8]) -> let_emitter::Result {
        self.object_write
            .write_all(&[let_opcodes::PTR, 0, 0, 0, 0, 0, 0, 0, 0])?;
        writeln!(
            self.links_write,
            "{} {}",
            Slice(name),
            self.counter + 1
        )?;
        self.counter += 9;
        Ok(())
    }

    fn ret(&mut self) -> let_emitter::Result {
        self.object_write.write_all(&[let_opcodes::RET])?;
        Ok(())
    }
}

pub fn open(path: &str) -> let_result::Result<impl let_emitter::Emitter> {
    let mut obj_path = path[..(path.len() - 4)].to_string();
    let mut lnk_path = obj_path.clone();
    let mut lbl_path = obj_path.clone();
    obj_path.push_str(".obj");
    lnk_path.push_str(".lnk");
    lbl_path.push_str(".lbl");
    match std::fs::File::create(obj_path.as_str()) {
        Ok(obj) => match std::fs::File::create(lnk_path.as_str()) {
            Ok(lnk) => match std::fs::File::create(lbl_path.as_str()) {
                Ok(lbl) => Ok(ObjectEmitter::new(
                    std::io::BufWriter::new(obj),
                    std::io::BufWriter::new(lnk),
                    std::io::BufWriter::new(lbl),
                )),
                Err(error) => {
                    let_result::raise!("Unable to create file \"{lnk_path}\", error: {error:?}")
                }
            },
            Err(error) => {
                let_result::raise!("Unable to create file \"{lnk_path}\", error: {error:?}")
            }
        },
        Err(error) => {
            let_result::raise!("Unable to create file \"{obj_path}\", error: {error:?}")
        }
    }
}
