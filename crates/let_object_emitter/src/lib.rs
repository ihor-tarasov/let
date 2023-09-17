use std::{collections::HashMap, fmt};

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

struct LabelInfo {
    address: Option<u64>,
    links: Vec<u64>,
}

struct Resolver {
    labels: HashMap<Box<[u8]>, LabelInfo>,
    indexes: HashMap<u64, LabelInfo>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            labels: HashMap::new(),
            indexes: HashMap::new(),
        }
    }

    fn push_label_name(&mut self, name: &[u8], address: u64) -> let_result::Result {
        if let Some(label) = self.labels.get_mut(name) {
            if let Some(label_address) = &mut label.address {
                *label_address = address;
                Ok(())
            } else {
                let_result::raise!("Symbol \"{}\" already present.", Slice(name))
            }
        } else {
            self.labels.insert(
                Vec::from(name).into_boxed_slice(),
                LabelInfo {
                    address: Some(address),
                    links: Vec::new(),
                },
            );
            Ok(())
        }
    }

    fn push_label_index(&mut self, index: u64, address: u64) -> let_result::Result {
        if let Some(label) = self.indexes.get_mut(&index) {
            if label.address.is_some() {
                let_result::raise!("Indexed label \"{}\" already present.", index)
            } else {
                label.address = Some(address);
                Ok(())
            }
        } else {
            self.indexes.insert(
                index,
                LabelInfo {
                    address: Some(address),
                    links: Vec::new(),
                },
            );
            Ok(())
        }
    }

    fn push_link_name(&mut self, name: &[u8], address: u64) {
        if let Some(label) = self.labels.get_mut(name) {
            label.links.push(address);
        } else {
            self.labels.insert(
                Vec::from(name).into_boxed_slice(),
                LabelInfo {
                    address: None,
                    links: vec![address],
                },
            );
        }
    }

    fn push_link_index(&mut self, index: u64, address: u64) {
        if let Some(label) = self.indexes.get_mut(&index) {
            label.links.push(address);
        } else {
            self.indexes.insert(
                index,
                LabelInfo {
                    address: None,
                    links: vec![address],
                },
            );
        }
    }

    fn resolve<W>(&mut self, write: &mut W) -> let_result::Result
    where
        W: std::io::Write + std::io::Seek,
    {
        for label in self.indexes.values() {
            if let Some(address) = label.address {
                for link in label.links.iter().cloned() {
                    write.seek(std::io::SeekFrom::Start(link as u64))?;
                    write.write_all(&address.to_be_bytes())?;
                }
            }
        }
        self.indexes.clear();
        for label in self.labels.values() {
            if let Some(address) = label.address {
                for link in label.links.iter().cloned() {
                    write.seek(std::io::SeekFrom::Start(link as u64))?;
                    write.write_all(&address.to_be_bytes())?;
                }
            }
        }
        Ok(())
    }

    fn save_labels<W>(&self, write: &mut W) -> let_result::Result
    where
        W: std::io::Write,
    {
        for (name, label) in self.labels.iter() {
            write!(write, "{}", Slice(name))?;
            if let Some(address) = label.address {
                write!(write, " {address}")?;
            } else {
                write!(write, " None")?;
            }
            for link in label.links.iter() {
                write!(write, " {link}")?;
            }
            writeln!(write)?;
        }
        Ok(())
    }
}

pub struct ObjectEmitter<W, M> {
    write: W,
    meta: M,
    resolver: Resolver,
}

impl<O, M> ObjectEmitter<O, M> {
    pub fn new(write: O, meta: M) -> Self {
        Self {
            write,
            meta,
            resolver: Resolver::new(),
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
