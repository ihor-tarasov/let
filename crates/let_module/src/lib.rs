use std::{
    collections::HashMap,
    fmt,
    hash::Hash,
    io::{Read, Write},
};

use reader::Reader;

use crate::writer::Writer;

#[derive(Hash, PartialEq, Eq)]
struct U8Str<'a>(&'a [u8]);

impl<'a> fmt::Display for U8Str<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.iter().cloned() {
            if c != b' ' {
                write!(f, "{}", c as char)?;
            }
        }
        Ok(())
    }
}

mod reader;
mod writer;

fn resolve<T>(
    links: &mut HashMap<T, Vec<u32>>,
    labels: &HashMap<T, u32>,
    opcodes: &mut [u8],
) -> let_result::Result
where
    T: Hash + PartialEq + Eq,
{
    for (name, &address) in labels.iter() {
        if let Some(links) = links.get_mut(name) {
            for link in links.iter().cloned() {
                for (i, b) in address.to_be_bytes().iter().cloned().enumerate() {
                    match opcodes.get_mut(link as usize + i) {
                        Some(r) => *r = b,
                        None => {
                            return let_result::raise!(
                                "Unable to resolve links, bytecode is corrupted."
                            )
                        }
                    }
                }
            }
            links.clear();
        }
    }
    links.retain(|_name, links| !links.is_empty());
    Ok(())
}

pub struct NamedLabels(HashMap<Box<[u8]>, u32>);

impl NamedLabels {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, name: &[u8], address: u32) -> let_result::Result {
        if self.0.contains_key(name) {
            let_result::raise!("Label \"{}\" already exists.", U8Str(name))
        } else {
            self.0.insert(Vec::from(name).into_boxed_slice(), address);
            Ok(())
        }
    }

    pub fn write<W>(&self, write: &mut W) -> let_result::Result
    where
        W: Write,
    {
        self.0.write(write)
    }

    pub fn read<R>(read: &mut R) -> let_result::Result<Self>
    where
        R: Read,
    {
        Ok(Self(HashMap::<Box<[u8]>, u32>::read(read)?))
    }

    pub fn merge(&mut self, other: Self) -> let_result::Result {
        for name in other.0.keys() {
            if self.0.contains_key(name) {
                return let_result::raise!("Duplicate label \"{}\".", U8Str(name));
            }
        }

        self.0.extend(other.0);
        Ok(())
    }

    pub fn get(&self, name: &[u8]) -> Option<u32> {
        self.0.get(name).cloned()
    }
}

pub struct NamedLinks(HashMap<Box<[u8]>, Vec<u32>>);

impl NamedLinks {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, name: &[u8], address: u32) {
        if let Some(links) = self.0.get_mut(name) {
            links.push(address);
        } else {
            self.0
                .insert(Vec::from(name).into_boxed_slice(), vec![address]);
        }
    }

    pub fn resolve(&mut self, labels: &NamedLabels, opcodes: &mut [u8]) -> let_result::Result {
        resolve(&mut self.0, &labels.0, opcodes)
    }

    pub fn write<W>(&self, write: &mut W) -> let_result::Result
    where
        W: Write,
    {
        self.0.write(write)
    }

    pub fn read<R>(read: &mut R) -> let_result::Result<Self>
    where
        R: Read,
    {
        Ok(Self(HashMap::<Box<[u8]>, Vec<u32>>::read(read)?))
    }

    pub fn merge(&mut self, other: Self) {
        for (name, src_links) in other.0 {
            if let Some(links) = self.0.get_mut(&name) {
                links.extend(src_links);
            } else {
                self.0.insert(name, src_links);
            }
        }
    }
}

pub struct IndexedLabels(HashMap<u32, u32>);

impl IndexedLabels {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, index: u32, address: u32) -> let_result::Result {
        if self.0.contains_key(&index) {
            let_result::raise!("Label {index} already exists.")
        } else {
            self.0.insert(index, address);
            Ok(())
        }
    }
}

pub struct IndexedLinks(HashMap<u32, Vec<u32>>);

impl IndexedLinks {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, index: u32, address: u32) {
        if let Some(links) = self.0.get_mut(&index) {
            links.push(address);
        } else {
            self.0.insert(index, vec![address]);
        }
    }

    pub fn resolve(&mut self, labels: &IndexedLabels, opcodes: &mut [u8]) -> let_result::Result {
        resolve(&mut self.0, &labels.0, opcodes)
    }
}

pub struct Module {
    pub opcodes: Vec<u8>,
    pub labels: NamedLabels,
    pub links: NamedLinks,
}

impl Module {
    pub fn resolve(&mut self) -> let_result::Result {
        self.links.resolve(&self.labels, &mut self.opcodes)
    }

    pub fn write<W>(&self, write: &mut W) -> let_result::Result
    where
        W: std::io::Write,
    {
        write.write_all(&[b'L', b'E', b'T', 38])?;
        self.opcodes.write(write)?;
        self.labels.write(write)?;
        self.links.write(write)?;
        Ok(())
    }

    pub fn read<R>(read: &mut R) -> let_result::Result<Self>
    where
        R: std::io::Read,
    {
        let mut magic = [0u8; 4];
        read.read_exact(&mut magic)?;
        if magic != [b'L', b'E', b'T', 38] {
            return let_result::raise!("Unknown format.");
        }
        let opcodes = Vec::<u8>::read(read)?;
        let labels = NamedLabels::read(read)?;
        let links = NamedLinks::read(read)?;

        Ok(Self {
            opcodes,
            labels,
            links,
        })
    }

    pub fn merge(&mut self, other: Self) -> let_result::Result {
        if self.opcodes.len() + other.opcodes.len() > u32::MAX as usize {
            return let_result::raise!("Program to big");
        }

        let offset = self.opcodes.len();
        self.opcodes.reserve(other.opcodes.len());
        let mut i = 0;
        while i < other.opcodes.len() {
            let opcode = other.opcodes[i];
            i += 1;
            self.opcodes.push(opcode);
            match opcode {
                0x00..=0x2F => (),
                0x30..=0x4F => {
                    let b = other.opcodes.get(i).unwrap().clone();
                    i += 1;
                    self.opcodes.push(b);
                }
                0x50..=0x6F => {
                    let bytes = &other.opcodes[i..(i + 2)];
                    i += 2;
                    if bytes.len() != 2 {
                        panic!()
                    }
                    self.opcodes.extend(bytes);
                }
                0x70..=0x8F => {
                    let mut bytes = [0; 4];
                    for j in 0..4 {
                        bytes[j] = other.opcodes.get(i + j).unwrap().clone();
                    }
                    i += 4;
                    if opcode == let_opcodes::JPF
                        || opcode == let_opcodes::JP
                        || opcode == let_opcodes::PTR
                    {
                        self.opcodes
                            .extend(&(u32::from_be_bytes(bytes) + offset as u32).to_be_bytes());
                    } else {
                        self.opcodes.extend(bytes);
                    }
                }
                0x90..=0xFF => {
                    let mut bytes = [0; 8];
                    for j in 0..8 {
                        bytes[j] = other.opcodes.get(i + j).unwrap().clone();
                    }
                    i += 8;
                    self.opcodes.extend(bytes);
                }
            };
        }

        self.labels.merge(other.labels)?;
        self.links.merge(other.links);

        Ok(())
    }
}
