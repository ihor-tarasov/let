use std::{collections::HashMap, fmt};

use crate::utils::{Info, self};

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

pub struct Name(HashMap<Box<[u8]>, Info>);

impl Name {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn label(&mut self, name: &[u8], address: u64) -> let_result::Result {
        if let Some(label) = self.0.get_mut(name) {
            if let Some(label_address) = &mut label.address {
                *label_address = address;
                Ok(())
            } else {
                let_result::raise!("Symbol \"{}\" already present.", Slice(name))
            }
        } else {
            self.0.insert(
                Vec::from(name).into_boxed_slice(),
                Info {
                    address: Some(address),
                    links: Vec::new(),
                },
            );
            Ok(())
        }
    }

    pub fn link(&mut self, name: &[u8], address: u64) {
        if let Some(label) = self.0.get_mut(name) {
            label.links.push(address);
        } else {
            self.0.insert(
                Vec::from(name).into_boxed_slice(),
                Info {
                    address: None,
                    links: vec![address],
                },
            );
        }
    }

    pub fn resolve(&self, opcodes: &mut [u8]) -> let_result::Result {
        utils::resolve(self.0.values(), opcodes)
    }

    pub fn save<W>(&self, module: Option<&str>, write: &mut W) -> let_result::Result
    where
        W: std::io::Write,
    {
        for (name, label) in self.0.iter() {
            if let Some(address) = label.address {
                if let Some(module) = module {
                    write!(write, "{module}.")?;
                }
                write!(write, "{}", Slice(name))?;
                write!(write, " {address}")?;
            } else {
                if name.contains(&b'.') {
                    write!(write, "{}", Slice(name))?;
                } else {
                    write!(write, "{}.__ctor__", Slice(name))?;
                }
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
