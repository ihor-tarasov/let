use std::collections::VecDeque;
use std::fmt::Write;
use std::io::Seek;
use std::path::Path;

mod line;

fn parse<R>(path: &str, file: R, emitter: &mut let_emitter::Emitter) -> let_result::Result
where
    R: std::io::Read + std::io::Seek,
{
    let mut iter = read_iter::ReadIter::new(file, 1024);
    let module_name = Path::new(path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .as_bytes();
    let mut parser = let_parser::Parser::new(&mut iter, module_name, emitter);
    if let Err(error) = parser.parse() {
        let range = parser.range();
        iter.seek(std::io::SeekFrom::Start(0))?;
        let info = line::create(&mut iter, range.start);
        let mut buffer = String::new();
        writeln!(buffer, "File \"{path}\", line: {}:", info.number).unwrap();
        line::print_line(&mut iter, info.start, &mut buffer);
        line::mark_range(info.start, range, &mut buffer);
        writeln!(buffer, "Compile error: {}", error).unwrap();
        return Err(let_result::Error::Custom(Box::new(buffer)));
    }
    if let Some(error) = iter.get_error() {
        return let_result::raise!("Error reading file \"{path}\", IOError: {error}");
    }
    Ok(())
}

fn compile(input_path: &str) -> let_result::Result<let_module::Module> {
    match std::fs::File::open(input_path) {
        Ok(file) => {
            let mut emitter = let_emitter::Emitter::new();
            parse(input_path, file, &mut emitter)?;
            let mut data = VecDeque::new();
            emitter.finish_to_write(&mut data)?;
            let_module::Module::read(data)
        }
        Err(error) => let_result::raise!("Unable to open file \"{input_path}\", error: {error}"),
    }
}

fn main() -> std::process::ExitCode {
    println!("Let Compiler");
    let mut module: Option<let_module::Module> = None;
    for arg in std::env::args().skip(1) {
        match compile(&arg) {
            Ok(result) => {
                if let Some(module) = &mut module {
                    module.merge(result).unwrap();
                } else {
                    module = Some(result)
                }
            }
            Err(error) => {
                eprintln!("{error}");
                return std::process::ExitCode::FAILURE;
            }
        }
    }

    let mut state = let_vm::State::new();

    if let Some(pc) = module.as_ref().unwrap().labels.get(b"main") {
        state.set_pc(pc);
    } else {
        panic!("Unable to find 'main' module")
    }

    match state.run(&module.unwrap().opcodes) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(error) => match error {
            let_vm::VMError::StackUnderflow => println!("Stack underflow."),
            let_vm::VMError::StackOverflow => println!("Stack overflow."),
            let_vm::VMError::FetchOpcodeError => println!("Fetch opcode error."),
            let_vm::VMError::Custom => println!("{}", state.message().unwrap()),
        },
    }
    std::process::ExitCode::SUCCESS
}
