use std::path::Path;

fn assemble(input_path: &str, output_path: &str) -> let_result::Result {
    let start = std::time::Instant::now();
    let module_name = Path::new(input_path).file_stem().unwrap().to_str().unwrap();
    match std::fs::File::open(input_path) {
        Ok(file) => {
            let mut emitter = let_object_emitter::ObjectEmitter::new();
            let_assembler::assemble(
                std::io::BufReader::new(file),
                &mut emitter,
            )?;
            emitter.finish(output_path, module_name)?;
            println!(
                "Assembled \"{input_path}\", time: {} seconds",
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
            match assemble(&input_path, &arg) {
                Ok(_) => (),
                Err(error) => {
                    eprintln!("Error: {error:?}");
                    return std::process::ExitCode::FAILURE;
                }
            }
        } else {
            input_path = Some(arg)
        }
    }
    if input_path.is_some() {
        eprintln!("Expected output path");
        return std::process::ExitCode::FAILURE
    }
    std::process::ExitCode::SUCCESS
}
