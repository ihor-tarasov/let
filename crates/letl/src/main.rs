struct Linker {
    opcodes: Vec<u8>,
    resolver: let_resolver::Resolver,
}

impl Linker {
    fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            resolver: let_resolver::Resolver::new(),
        }
    }

    fn link<R, M>(
        &mut self,
        object_read: &mut R,
        meta_read: &mut M,
    ) -> let_result::Result
    where
        R: std::io::Read,
        M: std::io::BufRead,
    {
        let offset = self.write.stream_position()?;

        let mut line = String::new();
        loop {
            line.clear();

            if meta_read.read_line(&mut line)? == 0 {
                break;
            }

            let mut tokens = line.trim().split_ascii_whitespace();

            let name = tokens.next().unwrap().trim();
            let address = tokens.next().unwrap().trim();

            if address == "None" {
                for token in tokens {
                    self.resolver
                        .push_link_name(name.as_bytes(), token.parse::<u64>()? + offset);
                }
            } else {
                if tokens.next().is_some() {
                    panic!(); // TODO: raise
                }

                self.resolver
                    .push_label_name(name.as_bytes(), address.parse::<u64>()? + offset)?;
            }
        }

        loop {
            let opcode = {
                let mut buf = [0; 1];
                match object_read.read_exact(&mut buf) {
                    Ok(_) => buf[0],
                    Err(error) => {
                        if error.kind() == std::io::ErrorKind::UnexpectedEof {
                            break;
                        } else {
                            return Err(error.into());
                        }
                    }
                }
            };

            match opcode {
                0x00..=0x2F => self.write.write_all(&[opcode])?,
                0x30..=0x4F => {
                    let mut buf = [0; 1];
                    object_read.read_exact(&mut buf)?;
                    self.write.write_all(&[opcode, buf[0]])?;
                }
                0x50..=0x6F => {
                    let mut buf = [0; 3];
                    object_read.read_exact(&mut buf)?;
                    self.write.write_all(&[opcode, buf[0], buf[1], buf[2]])?;
                }
                0x70..=0xFF => {
                    let mut buf = [0; 8];
                    object_read.read_exact(&mut buf)?;
                    self.write.write_all(&[opcode])?;
                    if opcode == let_opcodes::JPF
                        || opcode == let_opcodes::JP
                        || opcode == let_opcodes::PTR
                    {
                        self.write
                            .write_all(&(u64::from_be_bytes(buf) + offset).to_be_bytes())?;
                    } else {
                        self.write.write_all(&buf)?;
                    }
                }
            };
        }

        Ok(())
    }
}

fn link(path: &str) -> let_result::Result {
    Ok(())
}

fn main() -> std::process::ExitCode {
    println!("Let Linker");
    for arg in std::env::args().skip(1) {
        match link(&arg) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{error}");
                return std::process::ExitCode::FAILURE;
            }
        }
    }
    std::process::ExitCode::SUCCESS
}
