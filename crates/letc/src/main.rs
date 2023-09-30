use std::fmt::Write;
use std::io::Seek;
use std::path::Path;

mod line;

fn parse<R, E>(path: &str, file: R, emitter: &mut E) -> let_result::Result
where
    R: std::io::Read + std::io::Seek,
    E: let_emitter::Emitter,
{
    let mut iter = read_iter::ReadIter::new(file, 1024);
    let mut parser = let_parser::Parser::new(&mut iter, emitter);
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

fn compile(input_path: &str, output_path: &str, compile_assembly: bool) -> let_result::Result {
    let start = std::time::Instant::now();
    match std::fs::File::open(input_path) {
        Ok(file) => {
            if compile_assembly {
                parse(
                    input_path,
                    file,
                    &mut let_assembly_emitter::open(output_path)?,
                )?;
            } else {
                let mut emitter = let_object_emitter::ObjectEmitter::new();
                parse(input_path, file, &mut emitter)?;
                let module_name = Path::new(input_path).file_stem().unwrap().to_str().unwrap();
                emitter.finish(output_path, module_name)?;
            }
            println!(
                "Compiled \"{input_path}\", time: {} seconds",
                (std::time::Instant::now() - start).as_secs_f64()
            );
            Ok(())
        }
        Err(error) => let_result::raise!("Unable to open file \"{input_path}\", error: {error}"),
    }
}

fn main() -> std::process::ExitCode {
    println!("Let Compiler");
    let mut input_path: Option<String> = None;
    let mut compile_assembly = false;
    for arg in std::env::args().skip(1) {
        if arg == "-a" || arg == "--assembly" {
            compile_assembly = true;
        } else if let Some(input_path) = input_path.take() {
            match compile(&input_path, &arg, compile_assembly) {
                Ok(_) => (),
                Err(error) => {
                    eprintln!("{error}");
                    return std::process::ExitCode::FAILURE;
                }
            }
        } else {
            input_path = Some(arg)
        }
    }
    std::process::ExitCode::SUCCESS
}
