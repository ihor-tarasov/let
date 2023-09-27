use std::{
    fs::File,
    io::{SeekFrom, Write},
    path::PathBuf,
};

struct Linker {
    opcodes: Vec<u8>,
    resolver: let_resolver::Name,
}

fn read_file<R>(read: &mut R) -> let_result::Result<(Vec<u8>, String)>
where
    R: std::io::Read + std::io::Seek,
{
    let size_offset = read.seek(SeekFrom::End(-8))?;
    let mut size = [0; 8];
    read.read_exact(&mut size)?;
    let size = u64::from_be_bytes(size);
    read.seek(SeekFrom::Start(0))?;
    let mut opcodes = vec![0u8; size as usize];
    read.read_exact(&mut opcodes)?;
    let meta_size = size_offset - read.stream_position()?;
    let mut meta = vec![0u8; meta_size as usize];
    read.read_exact(&mut meta)?;
    Ok((opcodes, String::from_utf8(meta)?))
}

impl Linker {
    fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            resolver: let_resolver::Name::new(),
        }
    }

    fn link<R>(&mut self, read: &mut R) -> let_result::Result
    where
        R: std::io::Read + std::io::Seek,
    {
        let size = self.opcodes.len();
        let (opcodes, meta) = read_file(read)?;

        for line in meta.lines() {
            let mut tokens = line.split(' ');

            let name = tokens.next().unwrap().trim();
            let address = tokens.next().unwrap().trim();

            if address == "None" {
                for token in tokens {
                    self.resolver
                        .link(name.as_bytes(), token.parse::<u64>()? + size as u64);
                }
            } else {
                if tokens.next().is_some() {
                    panic!(); // TODO: raise
                }

                self.resolver
                    .label(name.as_bytes(), address.parse::<u64>()? + size as u64)?;
            }
        }

        let mut offset = 0;

        while offset < opcodes.len() {
            let opcode = opcodes[offset];
            offset += 1;
            match opcode {
                0x00..=0x2F => self.opcodes.push(opcode),
                0x30..=0x4F => {
                    let b = opcodes.get(offset).unwrap().clone();
                    offset += 1;
                    self.opcodes.extend([opcode, b]);
                }
                0x50..=0x6F => {
                    let bytes = &opcodes[offset..(offset + 3)];
                    offset += 3;
                    if bytes.len() != 3 {
                        panic!()
                    }
                    self.opcodes.push(opcode);
                    self.opcodes.extend(bytes);
                }
                0x70..=0xFF => {
                    let mut bytes = [0; 8];
                    for i in 0..8 {
                        bytes[i] = opcodes.get(offset + i).unwrap().clone();
                    }
                    offset += 8;
                    self.opcodes.push(opcode);
                    if opcode == let_opcodes::JPF
                        || opcode == let_opcodes::JP
                        || opcode == let_opcodes::PTR
                    {
                        self.opcodes
                            .extend(&(u64::from_be_bytes(bytes) + size as u64).to_be_bytes());
                    } else {
                        self.opcodes.extend(bytes);
                    }
                }
            };
        }

        Ok(())
    }

    fn finish(&mut self) -> let_result::Result {
        self.resolver.resolve(&mut self.opcodes)?;
        let size = self.opcodes.len();
        self.resolver.save(None, &mut self.opcodes)?;
        self.opcodes.extend((size as u64).to_be_bytes());
        let mut path = PathBuf::new();
        path.push("build");
        if !path.exists() {
            std::fs::create_dir(path.as_path())?;
        }
        path.push("linked");
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

fn main() -> std::process::ExitCode {
    println!("Let Linker");
    let mut linker = Linker::new();
    for arg in std::env::args().skip(1) {
        match File::open(&arg) {
            Ok(mut file) => match linker.link(&mut file) {
                Ok(_) => (),
                Err(error) => {
                    eprintln!("{error}");
                    return std::process::ExitCode::FAILURE;
                }
            },
            Err(error) => {
                eprintln!("{error}");
                return std::process::ExitCode::FAILURE;
            }
        }
    }
    match linker.finish() {
        Ok(_) => (),
        Err(error) => {
            eprintln!("{error}");
            return std::process::ExitCode::FAILURE;
        }
    }
    std::process::ExitCode::SUCCESS
}
