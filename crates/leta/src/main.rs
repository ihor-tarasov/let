use std::path::Path;

fn assemble(path: &str) -> let_result::Result {
    let start = std::time::Instant::now();
    let module = Path::new(path).file_stem().unwrap().to_str().unwrap();
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut emitter = let_object_emitter::ObjectEmitter::new();
            let_assembler::assemble(
                std::io::BufReader::new(file),
                &mut emitter,
            )?;
            emitter.finish(module)?;
            println!(
                "Assembled \"{path}\", time: {} seconds",
                (std::time::Instant::now() - start).as_secs_f64()
            );
            Ok(())
        }
        Err(error) => let_result::raise!("Unable to open file \"{path}\", error: {error}"),
    }
}

fn main() -> std::process::ExitCode {
    println!("Let Compiler");
    for arg in std::env::args().skip(1) {
        match assemble(&arg) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Error: {error:?}");
                return std::process::ExitCode::FAILURE;
            }
        }
    }
    std::process::ExitCode::SUCCESS
}
