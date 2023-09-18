use std::{fmt, io::Write, path::PathBuf};

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

pub struct ObjectEmitter {
    opcodes: Vec<u8>,
    resolver: let_resolver::Resolver,
}

impl ObjectEmitter {
    pub fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            resolver: let_resolver::Resolver::new(),
        }
    }
}

impl let_emitter::Emitter for ObjectEmitter {
    fn integer(&mut self, value: u64) -> let_emitter::Result {
        if value <= u8::MAX as u64 {
            self.opcodes.extend(&[let_opcodes::INT1, value as u8]);
        } else if value <= 0xFFFFFF {
            self.opcodes.extend(&[
                let_opcodes::INT3,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ]);
        } else {
            self.opcodes.extend(&[let_opcodes::INT8]);
            self.opcodes.extend(&value.to_be_bytes());
        }
        Ok(())
    }

    fn real(&mut self, value: f64) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::REAL]);
        self.opcodes.extend(&value.to_be_bytes());
        Ok(())
    }

    fn call(&mut self, arguments: u8) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::CALL, arguments]);
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
        self.opcodes.extend(&[opcode]);
        Ok(())
    }

    fn drop(&mut self) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::DROP]);
        Ok(())
    }

    fn label(&mut self, id: u64) -> let_emitter::Result {
        self.resolver
            .push_label_index(id, self.opcodes.len() as u64)?;
        Ok(())
    }

    fn label_named(&mut self, name: &[u8]) -> let_emitter::Result {
        self.resolver
            .push_label_name(name, self.opcodes.len() as u64)?;
        Ok(())
    }

    fn jump(&mut self, id: u64) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JP]);
        self.resolver.push_link_index(id, self.opcodes.len() as u64);
        self.opcodes.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
        Ok(())
    }

    fn jump_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JP]);
        self.resolver
            .push_link_name(name, self.opcodes.len() as u64);
        self.opcodes.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
        Ok(())
    }

    fn jump_false(&mut self, id: u64) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JPF]);
        self.resolver.push_link_index(id, self.opcodes.len() as u64);
        self.opcodes.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
        Ok(())
    }

    fn jump_false_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JPF]);
        self.resolver
            .push_link_name(name, self.opcodes.len() as u64);
        self.opcodes.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
        Ok(())
    }

    fn load(&mut self, index: u64) -> let_emitter::Result {
        if index <= u8::MAX as u64 {
            self.opcodes.extend(&[let_opcodes::LD1, index as u8]);
        } else if index <= 0xFFFFFF {
            self.opcodes.extend(&[
                let_opcodes::LD3,
                (index >> 16) as u8,
                (index >> 8) as u8,
                index as u8,
            ]);
        } else {
            self.opcodes.extend(&[let_opcodes::INT8]);
            self.opcodes.extend(&index.to_be_bytes());
        }
        Ok(())
    }

    fn pointer(&mut self, name: &[u8]) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::PTR]);
        self.resolver
            .push_link_name(name, self.opcodes.len() as u64);
        self.opcodes.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
        Ok(())
    }

    fn ret(&mut self) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::RET]);
        Ok(())
    }

    fn finish(&mut self, module: &str) -> let_result::Result {
        self.resolver.resolve(&mut self.opcodes)?;
        let size = self.opcodes.len();
        self.resolver.save_labels(module, &mut self.opcodes)?;
        self.opcodes.extend((size as u64).to_be_bytes());
        let mut path = PathBuf::new();
        path.push("build");
        if !path.exists() {
            std::fs::create_dir(path.as_path())?;
        }
        path.push(module);
        path.set_extension("lbin");
        match std::fs::File::create(path.as_path()) {
            Ok(mut file) => file.write_all(&self.opcodes)?,
            Err(error) => {
                return let_result::raise!(
                    "Unable to create file {:?}, error: {error}.",
                    path.as_os_str()
                )
            }
        }
        Ok(())
    }
}
