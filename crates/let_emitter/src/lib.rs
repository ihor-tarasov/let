use std::fmt;

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

pub struct Emitter {
    opcodes: Vec<u8>,
    named_labels: let_module::NamedLabels,
    named_links: let_module::NamedLinks,
    indexed_labels: let_module::IndexedLabels,
    indexed_links: let_module::IndexedLinks,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            named_labels: let_module::NamedLabels::new(),
            named_links: let_module::NamedLinks::new(),
            indexed_labels: let_module::IndexedLabels::new(),
            indexed_links: let_module::IndexedLinks::new(),
        }
    }

    pub fn integer(&mut self, value: u64) -> let_result::Result {
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

    pub fn real(&mut self, value: f64) -> let_result::Result {
        self.opcodes.extend(&[let_opcodes::REAL]);
        self.opcodes.extend(&value.to_be_bytes());
        Ok(())
    }

    pub fn call(&mut self, arguments: u8) -> let_result::Result {
        self.opcodes.extend(&[let_opcodes::CALL, arguments]);
        Ok(())
    }

    pub fn binary(&mut self, operator: [u8; 3]) -> let_result::Result {
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

    pub fn drop(&mut self) -> let_result::Result {
        self.opcodes.extend(&[let_opcodes::DROP]);
        Ok(())
    }

    pub fn label(&mut self, id: u32) -> let_result::Result {
        self.indexed_labels.push(id, self.opcodes.len() as u32)?;
        Ok(())
    }

    pub fn label_named(&mut self, name: Box<[u8]>) -> let_result::Result {
        self.named_labels.push(name, self.opcodes.len() as u32)?;
        Ok(())
    }

    pub fn jump(&mut self, id: u32) -> let_result::Result {
        self.opcodes.extend(&[let_opcodes::JP]);
        self.indexed_links.push(id, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    pub fn jump_false(&mut self, id: u32) -> let_result::Result {
        self.opcodes.extend(&[let_opcodes::JPF]);
        self.indexed_links.push(id, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    pub fn load(&mut self, index: u32) {
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

    pub fn pointer(&mut self, name: &[u8]) -> let_result::Result {
        self.opcodes.extend(&[let_opcodes::PTR]);
        self.named_links.push(name, self.opcodes.len() as u32);
        self.opcodes.extend(&[0, 0, 0, 0]);
        Ok(())
    }

    pub fn ret(&mut self) -> let_result::Result {
        self.opcodes.extend(&[let_opcodes::RET]);
        Ok(())
    }

    /*
    
    function_name:
        DB 3  ; Arguments count
        DD 23 ; Stack size
        ... ; Other opcodes
     */
    pub fn function(&mut self, args_count: u8) -> let_result::Result<u32> {
        let address = self.opcodes.len() + 1;
        self.opcodes.extend(&[args_count, 0, 0, 0, 0]);
        Ok(address as u32)
    }

    pub fn set(&mut self, address: u32, value: u8) {
        self.opcodes[address as usize] = value;
    }

    pub fn store(&mut self, index: u32) {
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

    pub fn offset(&self) -> u32 {
        self.opcodes.len() as u32
    }

    pub fn jump_to(&mut self, address: u32) {
        self.opcodes.push(let_opcodes::JP);
        self.opcodes.extend(&address.to_be_bytes());
    }

    pub fn void(&mut self) {
        self.opcodes.push(let_opcodes::VOID);
    }

    pub fn list(&mut self) {
        self.opcodes.push(let_opcodes::LIST);
    }

    pub fn resolve(&mut self) -> let_result::Result {
        self.indexed_links
            .resolve(&self.indexed_labels, &mut self.opcodes)?;
        self.indexed_labels.clear();
        self.named_links
            .resolve(&self.named_labels, &mut self.opcodes)?;
        Ok(())
    }

    pub fn into_module(self) -> let_module::Module {
        let_module::Module {
            opcodes: self.opcodes,
            labels: self.named_labels,
            links: self.named_links,
        }
    }

    pub fn write<W>(self, write: &mut W) -> let_result::Result
    where
        W: std::io::Write,
    {
        let module = self.into_module();
        module.write(write)
    }

    pub fn finish(self, path: &str) -> let_result::Result {
        match std::fs::File::create(path) {
            Ok(mut file) => self.write(&mut file)?,
            Err(error) => {
                return let_result::raise!("Unable to create file {:?}, error: {error}.", path)
            }
        }
        Ok(())
    }
}
