pub struct Info {
    pub address: Option<u64>,
    pub links: Vec<u64>,
}

pub fn resolve<'a, I>(iter: I, opcodes: &mut [u8]) -> let_result::Result
where
    I: Iterator<Item = &'a Info>,
{
    for label in iter {
        if let Some(address) = label.address {
            for link in label.links.iter().cloned() {
                address
                    .to_be_bytes()
                    .iter()
                    .cloned()
                    .enumerate()
                    .for_each(|(i, b)| {
                        opcodes[link as usize + i] = b;
                    });
            }
        }
    }
    Ok(())
}
