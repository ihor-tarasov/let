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

pub struct ObjectEmitter<W, M> {
    write: W,
    meta: M,
    resolver: let_resolver::Resolver,
}

impl<O, M> ObjectEmitter<O, M> {
    pub fn new(write: O, meta: M) -> Self {
        Self {
            write,
            meta,
            resolver: let_resolver::Resolver::new(),
        }
    }
}

impl<W, M> let_emitter::Emitter for ObjectEmitter<W, M>
where
    W: std::io::Write + std::io::Seek,
    M: std::io::Write,
{
    fn integer(&mut self, value: u64) -> let_emitter::Result {
        if value <= u8::MAX as u64 {
            self.write.write_all(&[let_opcodes::INT1, value as u8])?;
        } else if value <= 0xFFFFFF {
            self.write.write_all(&[
                let_opcodes::INT3,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
        } else {
            self.write.write_all(&[let_opcodes::INT8])?;
            self.write.write_all(&value.to_be_bytes())?;
        }
        Ok(())
    }

    fn real(&mut self, value: f64) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::REAL])?;
        self.write.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn call(&mut self, arguments: u8) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::CALL, arguments])?;
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
        self.write.write_all(&[opcode])?;
        Ok(())
    }

    fn drop(&mut self) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::DROP])?;
        Ok(())
    }

    fn label(&mut self, id: u64) -> let_emitter::Result {
        self.resolver
            .push_label_index(id, self.write.stream_position()?)?;
        Ok(())
    }

    fn label_named(&mut self, name: &[u8]) -> let_emitter::Result {
        self.resolver
            .push_label_name(name, self.write.stream_position()?)?;
        Ok(())
    }

    fn jump(&mut self, id: u64) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::JP])?;
        self.resolver
            .push_link_index(id, self.write.stream_position()?);
        self.write.write_all(&[0, 0, 0, 0, 0, 0, 0, 0])?;
        Ok(())
    }

    fn jump_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::JP])?;
        self.resolver
            .push_link_name(name, self.write.stream_position()?);
        self.write.write_all(&[0, 0, 0, 0, 0, 0, 0, 0])?;
        Ok(())
    }

    fn jump_false(&mut self, id: u64) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::JPF])?;
        self.resolver
            .push_link_index(id, self.write.stream_position()?);
        self.write.write_all(&[0, 0, 0, 0, 0, 0, 0, 0])?;
        Ok(())
    }

    fn jump_false_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::JPF])?;
        self.resolver
            .push_link_name(name, self.write.stream_position()?);
        self.write.write_all(&[0, 0, 0, 0, 0, 0, 0, 0])?;
        Ok(())
    }

    fn load(&mut self, index: u64) -> let_emitter::Result {
        if index <= u8::MAX as u64 {
            self.write.write_all(&[let_opcodes::LD1, index as u8])?;
        } else if index <= 0xFFFFFF {
            self.write.write_all(&[
                let_opcodes::LD3,
                (index >> 16) as u8,
                (index >> 8) as u8,
                index as u8,
            ])?;
        } else {
            self.write.write_all(&[let_opcodes::INT8])?;
            self.write.write_all(&index.to_be_bytes())?;
        }
        Ok(())
    }

    fn pointer(&mut self, name: &[u8]) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::PTR])?;
        self.resolver
            .push_link_name(name, self.write.stream_position()?);
        self.write.write_all(&[0, 0, 0, 0, 0, 0, 0, 0])?;
        Ok(())
    }

    fn ret(&mut self) -> let_emitter::Result {
        self.write.write_all(&[let_opcodes::RET])?;
        Ok(())
    }

    fn finish(&mut self) -> let_result::Result {
        self.resolver.resolve(&mut self.write)?;
        self.resolver.save_labels(&mut self.meta)
    }
}

pub fn open(path: &str) -> let_result::Result<impl let_emitter::Emitter> {
    let mut obj_path = path[..(path.len() - 4)].to_string();
    let mut meta_path = obj_path.clone();
    obj_path.push_str(".obj");
    meta_path.push_str(".meta");
    match std::fs::File::create(obj_path.as_str()) {
        Ok(obj) => match std::fs::File::create(meta_path.as_str()) {
            Ok(meta) => Ok(ObjectEmitter::new(
                std::io::BufWriter::new(obj),
                std::io::BufWriter::new(meta),
            )),
            Err(error) => {
                let_result::raise!("Unable to create file \"{meta_path}\", error: {error:?}")
            }
        },
        Err(error) => {
            let_result::raise!("Unable to create file \"{obj_path}\", error: {error:?}")
        }
    }
}
