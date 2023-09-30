use std::fs::File;

struct Linker(Option<let_module::Module>);

impl Linker {
    fn new() -> Self {
        Self(None)
    }

    fn link<R>(&mut self, mut read: R) -> let_result::Result
    where
        R: std::io::Read,
    {
        let other = let_module::Module::read(&mut read)?;
        if let Some(current) = &mut self.0 {
            current.merge(other)?;
        } else {
            self.0 = Some(other)
        }

        Ok(())
    }

    fn finish(self, path: &str) -> let_result::Result {
        if let Some(current) = self.0 {
            current.write(std::fs::File::create(path)?)?;
            Ok(())
        } else {
            let_result::raise!("Zero input files provided.")
        }
    }
}

fn main() -> std::process::ExitCode {
    println!("Let Linker");
    let mut output_path = None;
    let mut waiting_output = false;
    let mut linker = Linker::new();
    for arg in std::env::args().skip(1) {
        if arg == "-o" || arg == "--output" {
            if output_path.is_none() {
                waiting_output = true;
                continue;
            } else {
                eprintln!("Multiple using of \"--output\" parameter.");
                return std::process::ExitCode::FAILURE;
            }
        }
        if waiting_output {
            waiting_output = false;
            output_path = Some(arg);
            continue;
        }
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
    if let Some(output_path) = output_path {
        match linker.finish(&output_path) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{error}");
                return std::process::ExitCode::FAILURE;
            }
        }
    } else {
        eprintln!("Output path is not provided.");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
