use std::collections::HashMap;

use crate::utils::{self, Info};

pub struct Index(HashMap<u64, Info>);

impl Index {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn label(&mut self, index: u64, address: u64) -> let_result::Result {
        if let Some(label) = self.0.get_mut(&index) {
            if label.address.is_some() {
                let_result::raise!("Indexed label \"{}\" already present.", index)
            } else {
                label.address = Some(address);
                Ok(())
            }
        } else {
            self.0.insert(
                index,
                Info {
                    address: Some(address),
                    links: Vec::new(),
                },
            );
            Ok(())
        }
    }

    pub fn link(&mut self, index: u64, address: u64) {
        if let Some(label) = self.0.get_mut(&index) {
            label.links.push(address);
        } else {
            self.0.insert(
                index,
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
}
