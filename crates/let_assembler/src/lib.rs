struct Assembler<'a, E> {
    emiter: &'a mut E,
}

impl<'a, E> Assembler<'a, E>
where
    E: let_emitter::Emitter,
{
    fn new(emiter: &'a mut E) -> Self {
        Self { emiter }
    }

    fn assemble(&mut self, mut line: &str) -> let_result::Result {
        line = line.trim();

        // Handle lables
        while let Some(index) = line.find(':') {
            self.emiter.label_named(line[0..index].trim().as_bytes())?;
            line = &line[(index + 1)..];

            if line.is_empty() {
                return Ok(());
            }
        }

        if line.starts_with("LD ") {
            line = line[3..].trim();
            self.emiter.load(line.parse::<u64>()?)
        } else if line.starts_with("PTR ") {
            line = line[3..].trim();
            self.emiter.pointer(line.as_bytes())
        } else if line.starts_with("INT ") {
            line = line[4..].trim();
            self.emiter.integer(line.parse::<u64>()?)
        } else if line.starts_with("OP ") {
            line = line[3..].trim();
            if line.len() > 3 || line.is_empty() {
                return let_result::raise!("Wrong operator triplet");
            }
            let operator = [
                line.as_bytes().get(0).cloned().unwrap_or(b' '),
                line.as_bytes().get(1).cloned().unwrap_or(b' '),
                line.as_bytes().get(2).cloned().unwrap_or(b' '),
            ];
            self.emiter.binary(operator)
        } else if line.starts_with("JPF ") {
            line = line[4..].trim();
            self.emiter.jump_false_name(line.as_bytes())
        } else if line.starts_with("JP ") {
            line = line[3..].trim();
            self.emiter.jump_name(line.as_bytes())
        } else if line == "RET" {
            self.emiter.ret()
        } else if line == "DROP" {
            self.emiter.drop()
        } else if line.starts_with("CALL ") {
            line = line[5..].trim();
            self.emiter.call(line.parse::<u8>()?)
        } else {
            let_result::raise!("UnexpeEted line \"{}\"", line)
        }
    }
}

pub fn assemble<R, E>(mut read: R, emiter: &mut E) -> let_result::Result
where
    R: std::io::BufRead,
    E: let_emitter::Emitter,
{
    let mut assembler = Assembler::new(emiter);
    let mut line = String::new();
    loop {
        line.clear();
        if read.read_line(&mut line)? == 0 {
            break Ok(());
        }
        assembler.assemble(line.as_str())?;
    }
}
