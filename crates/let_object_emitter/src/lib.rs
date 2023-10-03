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

pub struct ObjectEmitter {
    opcodes: Vec<u8>,
    named_labels: let_module::NamedLabels,
    named_links: let_module::NamedLinks,
    indexed_labels: let_module::IndexedLabels,
    indexed_links: let_module::IndexedLinks,
}

impl ObjectEmitter {
    pub fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            named_labels: let_module::NamedLabels::new(),
            named_links: let_module::NamedLinks::new(),
            indexed_labels: let_module::IndexedLabels::new(),
            indexed_links: let_module::IndexedLinks::new(),
        }
    }

    pub fn finish(mut self, path: &str, module_name: &[u8]) -> let_result::Result {
        self.indexed_links
            .resolve(&self.indexed_labels, &mut self.opcodes)?;
        self.named_links
            .resolve(&self.named_labels, &mut self.opcodes)?;

        let module = let_module::Module {
            opcodes: self.opcodes,
            labels: self.named_labels,
            links: self.named_links,
        };

        match std::fs::File::create(path) {
            Ok(mut file) => module.write_prefixed(module_name, &mut file)?,
            Err(error) => {
                return let_result::raise!("Unable to create file {:?}, error: {error}.", path)
            }
        }
        Ok(())
    }
}

impl let_emitter::Emitter for ObjectEmitter {
    fn integer(&mut self, value: u64) -> let_emitter::Result {
        if value <= u8::MAX as u64 {
            self.opcodes.extend(&[let_opcodes::INT1, value as u8]);
        } else if value <= 0xFFFF {
            self.opcodes
                .extend(&[let_opcodes::INT2, (value >> 8) as u8, value as u8]);
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

    fn label(&mut self, id: u32) -> let_emitter::Result {
        self.indexed_labels.push(id, self.opcodes.len() as u32)?;
        Ok(())
    }

    fn label_named(&mut self, name: &[u8]) -> let_emitter::Result {
        self.named_labels.push(name, self.opcodes.len() as u32)?;
        Ok(())
    }

    fn jump(&mut self, id: u32) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JP]);
        self.indexed_links.push(id, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    fn jump_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JP]);
        self.named_links.push(name, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    fn jump_false(&mut self, id: u32) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JPF]);
        self.indexed_links.push(id, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    fn jump_false_name(&mut self, name: &[u8]) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::JPF]);
        self.named_links.push(name, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    fn load(&mut self, index: u32) {
        if index <= u8::MAX as u32 {
            self.opcodes.extend(&[let_opcodes::LD1, index as u8]);
        } else if index <= 0xFFFF {
            self.opcodes
                .extend(&[let_opcodes::LD2, (index >> 8) as u8, index as u8]);
        } else {
            self.opcodes.push(let_opcodes::LD4);
            self.opcodes.extend(&index.to_le_bytes());
        }
    }

    fn pointer(&mut self, name: &[u8]) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::PTR]);
        self.named_links.push(name, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    fn ret(&mut self) -> let_emitter::Result {
        self.opcodes.extend(&[let_opcodes::RET]);
        Ok(())
    }

    /*
    
    function_name:
        DB 3  ; Arguments count
        DD 23 ; Stack size
        ... ; Other opcodes
     */
    fn function(&mut self, args_count: u8) -> let_result::Result<u32> {
        let address = self.opcodes.len() + 1;
        self.opcodes.extend(&[args_count, 0, 0, 0, 0]);
        Ok(address as u32)
    }

    fn set(&mut self, address: u32, value: u8) {
        self.opcodes[address as usize] = value;
    }

    fn store(&mut self, index: u32) {
        if index <= u8::MAX as u32 {
            self.opcodes.extend(&[let_opcodes::ST1, index as u8]);
        } else if index <= 0xFFFF {
            self.opcodes
                .extend(&[let_opcodes::ST2, (index >> 8) as u8, index as u8]);
        } else {
            self.opcodes.push(let_opcodes::ST4);
            self.opcodes.extend(&index.to_le_bytes());
        }
    }
}
