use std::{collections::HashMap, fmt};

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

pub struct Resolver {
    labels: HashMap<Box<[u8]>, LabelInfo>,
    indexes: HashMap<u64, LabelInfo>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            indexes: HashMap::new(),
        }
    }

    pub fn push_label_name(&mut self, name: &[u8], address: u64) -> let_result::Result {
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

    pub fn push_label_index(&mut self, index: u64, address: u64) -> let_result::Result {
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

    pub fn push_link_name(&mut self, name: &[u8], address: u64) {
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

    pub fn push_link_index(&mut self, index: u64, address: u64) {
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

    pub fn resolve<W>(&mut self, write: &mut W) -> let_result::Result
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
        for label in self.labels.values_mut() {
            if let Some(address) = label.address {
                for link in label.links.iter().cloned() {
                    write.seek(std::io::SeekFrom::Start(link as u64))?;
                    write.write_all(&address.to_be_bytes())?;
                }
                label.links.clear();
            }
        }
        Ok(())
    }

    pub fn save_labels<W>(&self, write: &mut W) -> let_result::Result
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
