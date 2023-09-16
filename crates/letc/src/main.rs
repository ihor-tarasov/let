mod assembly_emiter;
mod read_iter;

fn compile(path: &str, compile_assembly: bool) -> let_result::Result {
    let start = std::time::Instant::now();
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut iter = read_iter::ReadIter::new(file, 1024);
            if compile_assembly {
                let_parser::parse(&mut iter, assembly_emiter::open(path)?)?;
            } else {
                let_parser::parse(&mut iter, let_object_emitter::open(path)?)?;
            }
            if let Some(error) = iter.get_error() {
                return let_result::raise!("Error reading file \"{path}\", IOError: {error}");
            }
            println!(
                "Compiled \"{path}\", time: {} seconds",
                (std::time::Instant::now() - start).as_secs_f64()
            );
            Ok(())
        }
        Err(error) => let_result::raise!("Unable to open file \"{path}\", error: {error}"),
    }
}

fn main() -> std::process::ExitCode {
    println!("Let Compiler");
    let mut compile_assembly = false;
    for arg in std::env::args().skip(1) {
        if arg == "-a" || arg == "--assembly" {
            compile_assembly = true;
        } else if arg == "-o" || arg == "--object" {
            compile_assembly = false;
        } else {
            match compile(&arg, compile_assembly) {
                Ok(_) => (),
                Err(error) => {
                    eprintln!("Error: {error:?}");
                    return std::process::ExitCode::FAILURE;
                }
            }
        }
    }
    std::process::ExitCode::SUCCESS
}
