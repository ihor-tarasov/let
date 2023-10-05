use std::fmt::Write;
use std::io::Seek;
use std::path::Path;

mod line;

fn parse<R>(path: &str, file: R, emitter: &mut let_emitter::Emitter) -> let_result::Result
where
    R: std::io::Read + std::io::Seek,
{
    let mut iter = read_iter::ReadIter::new(file, 1024);
    let module_name = Path::new(path).file_stem().unwrap().to_str().unwrap().as_bytes();
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

fn compile(input_path: &str, output_path: &str) -> let_result::Result {
    let start = std::time::Instant::now();
    match std::fs::File::open(input_path) {
        Ok(file) => {
            let mut emitter = let_emitter::Emitter::new();
            parse(input_path, file, &mut emitter)?;
            emitter.resolve()?;
            emitter.finish(output_path)?;
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
    for arg in std::env::args().skip(1) {
        if let Some(input_path) = input_path.take() {
            match compile(&input_path, &arg) {
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
