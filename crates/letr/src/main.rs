use std::fs::File;

fn run<R>(read: &mut R) -> let_result::Result
where
    R: std::io::Read,
{
    let module = let_module::Module::read(read)?;

    let mut state = let_vm::State::new();

    if let Some(pc) = module.labels.get(b"main") {
        state.set_pc(pc);
    } else {
        return let_result::raise!("Unable to find \"main\" module.");
    }

    match state.run(&module.opcodes) {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(error) => match error {
            let_vm::VMError::StackUnderflow => let_result::raise!("Stack underflow."),
            let_vm::VMError::StackOverflow => let_result::raise!("Stack overflow."),
            let_vm::VMError::FetchOpcodeError => let_result::raise!("Fetch opcode error."),
            let_vm::VMError::Custom => let_result::raise!("{}", state.message().unwrap()),
        },
    }
}

fn run_file(path: &str) -> let_result::Result {
    run(&mut File::open(path)?)
}

fn main() -> std::process::ExitCode {
    println!("Let Runtime");
    for arg in std::env::args().skip(1) {
        match run_file(arg.as_str()) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{error}");
                return std::process::ExitCode::FAILURE;
            }
        }
    }
    std::process::ExitCode::SUCCESS
}
