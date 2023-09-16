mod assembler;

fn assemble(path: &str) -> let_result::Result {
    let start = std::time::Instant::now();
    match std::fs::File::open(path) {
        Ok(file) => {
            assembler::assemble(
                std::io::BufReader::new(file),
                let_object_emitter::open(path)?,
            )?;
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
