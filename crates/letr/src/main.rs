use std::{fs::File, io::SeekFrom};

fn read_file<R>(read: &mut R) -> let_result::Result<(Vec<u8>, String)>
where
    R: std::io::Read + std::io::Seek,
{
    let size_offset = read.seek(SeekFrom::End(-8))?;
    let mut size = [0; 8];
    read.read_exact(&mut size)?;
    let size = u64::from_be_bytes(size);
    read.seek(SeekFrom::Start(0))?;
    let mut opcodes = vec![0u8; size as usize];
    read.read_exact(&mut opcodes)?;
    let meta_size = size_offset - read.stream_position()?;
    let mut meta = vec![0u8; meta_size as usize];
    read.read_exact(&mut meta)?;
    Ok((opcodes, String::from_utf8(meta)?))
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Void,
    Boolean(bool),
    Integer(i64),
    Address(u64),
}

fn run<R>(read: &mut R) -> let_result::Result
where
    R: std::io::Read + std::io::Seek,
{
    let (opcodes, meta) = read_file(read)?;

    let mut pc = 0;

    for line in meta.lines() {
        let mut tokens = line.split(' ');

        let name = tokens.next().unwrap().trim();
        let address = tokens.next().unwrap().trim();

        if address == "None" {
            let_result::raise!("Unknown symbol \"{name}\"")?;
        } else {
            if name.ends_with("__ctor__") {
                pc = address.parse::<u64>()? as usize;
            }
        }
    }

    let mut stack = [Value::Void; 256];
    let mut calls = [(0, 0); 256];
    let mut cp = 0;
    let mut sp = 0;
    let mut locals = 0;

    loop {
        let opcode = opcodes.get(pc).unwrap().clone();
        match opcode {
            let_opcodes::DROP => {
                if sp == 0 {
                    panic!("Stack underflow");
                }
                sp -= 1;
                pc += 1;
            }
            let_opcodes::LS => {
                if sp < 2 || sp > stack.len() {
                    panic!("Stack overflow");
                }
                let left = stack[sp - 2];
                let right = stack[sp - 1];

                let result = match (left, right) {
                    (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left < right),
                    _ => panic!("Unsupported type"),
                };

                stack[sp - 2] = result;
                sp -= 1;
                pc += 1;
            }
            let_opcodes::GR => {
                if sp < 2 || sp > stack.len() {
                    panic!("Stack overflow");
                }
                let left = stack[sp - 2];
                let right = stack[sp - 1];

                let result = match (left, right) {
                    (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left > right),
                    _ => panic!("Unsupported type"),
                };

                stack[sp - 2] = result;
                sp -= 1;
                pc += 1;
            }
            let_opcodes::EQ => {
                if sp < 2 || sp > stack.len() {
                    panic!("Stack overflow");
                }
                let left = stack[sp - 2];
                let right = stack[sp - 1];

                let result = match (left, right) {
                    (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left == right),
                    _ => panic!("Unsupported type"),
                };

                stack[sp - 2] = result;
                sp -= 1;
                pc += 1;
            }
            let_opcodes::ADD => {
                if sp < 2 || sp > stack.len() {
                    panic!("Stack overflow");
                }
                let left = stack[sp - 2];
                let right = stack[sp - 1];

                let result = match (left, right) {
                    (Value::Integer(left), Value::Integer(right)) => {
                        Value::Integer(left.wrapping_add(right))
                    }
                    _ => panic!("Unsupported type"),
                };

                stack[sp - 2] = result;
                sp -= 1;
                pc += 1;
            }
            let_opcodes::LE => {
                if sp < 2 || sp > stack.len() {
                    panic!("Stack overflow");
                }
                let left = stack[sp - 2];
                let right = stack[sp - 1];

                let result = match (left, right) {
                    (Value::Integer(left), Value::Integer(right)) => Value::Boolean(left <= right),
                    _ => panic!("Unsupported type"),
                };

                stack[sp - 2] = result;
                sp -= 1;
                pc += 1;
            }
            let_opcodes::SUB => {
                if sp < 2 || sp > stack.len() {
                    panic!("Stack overflow");
                }
                let left = stack[sp - 2];
                let right = stack[sp - 1];

                let result = match (left, right) {
                    (Value::Integer(left), Value::Integer(right)) => {
                        Value::Integer(left.wrapping_sub(right))
                    }
                    _ => panic!("Unsupported type"),
                };

                stack[sp - 2] = result;
                sp -= 1;
                pc += 1;
            }
            let_opcodes::MUL => {
                if sp < 2 || sp > stack.len() {
                    panic!("Stack overflow");
                }
                let left = stack[sp - 2];
                let right = stack[sp - 1];

                let result = match (left, right) {
                    (Value::Integer(left), Value::Integer(right)) => {
                        Value::Integer(left.wrapping_mul(right))
                    }
                    _ => panic!("Unsupported type"),
                };

                stack[sp - 2] = result;
                sp -= 1;
                pc += 1;
            }
            let_opcodes::INT1 => {
                if sp >= stack.len() {
                    panic!("Stack overflow");
                }
                stack[sp] = Value::Integer(opcodes.get(pc + 1).unwrap().clone() as i64);
                sp += 1;
                pc += 2;
            }
            let_opcodes::PTR => {
                if sp >= stack.len() {
                    panic!("Stack overflow");
                }
                stack[sp] = Value::Address(u64::from_be_bytes([
                    opcodes.get(pc + 1).unwrap().clone(),
                    opcodes.get(pc + 2).unwrap().clone(),
                    opcodes.get(pc + 3).unwrap().clone(),
                    opcodes.get(pc + 4).unwrap().clone(),
                    opcodes.get(pc + 5).unwrap().clone(),
                    opcodes.get(pc + 6).unwrap().clone(),
                    opcodes.get(pc + 7).unwrap().clone(),
                    opcodes.get(pc + 8).unwrap().clone(),
                ]));
                sp += 1;
                pc += 9;
            }
            let_opcodes::JPF => {
                if sp == 0 || sp >= stack.len() {
                    panic!("Stack overflow");
                }
                match stack[sp - 1] {
                    Value::Boolean(value) => {
                        if !value {
                            pc = u64::from_be_bytes([
                                opcodes.get(pc + 1).unwrap().clone(),
                                opcodes.get(pc + 2).unwrap().clone(),
                                opcodes.get(pc + 3).unwrap().clone(),
                                opcodes.get(pc + 4).unwrap().clone(),
                                opcodes.get(pc + 5).unwrap().clone(),
                                opcodes.get(pc + 6).unwrap().clone(),
                                opcodes.get(pc + 7).unwrap().clone(),
                                opcodes.get(pc + 8).unwrap().clone(),
                            ]) as usize;
                        } else {
                            pc += 9;
                        }
                    }
                    _ => panic!("Unexpected type"),
                }

                sp -= 1;
            }
            let_opcodes::JP => {
                pc = u64::from_be_bytes([
                    opcodes.get(pc + 1).unwrap().clone(),
                    opcodes.get(pc + 2).unwrap().clone(),
                    opcodes.get(pc + 3).unwrap().clone(),
                    opcodes.get(pc + 4).unwrap().clone(),
                    opcodes.get(pc + 5).unwrap().clone(),
                    opcodes.get(pc + 6).unwrap().clone(),
                    opcodes.get(pc + 7).unwrap().clone(),
                    opcodes.get(pc + 8).unwrap().clone(),
                ]) as usize;
            }
            let_opcodes::CALL => {
                let params_count = opcodes.get(pc + 1).unwrap().clone();
                if sp < params_count as usize + 1 {
                    panic!("Stack underflow")
                }
                let (save_pc, save_locals) = calls.get_mut(cp).unwrap();
                cp += 1;
                *save_pc = pc + 2;
                *save_locals = locals;
                match stack[sp - params_count as usize - 1] {
                    Value::Address(address) => pc = address as usize,
                    _ => panic!("Unexpected type"),
                }
                locals = sp - params_count as usize;
            }
            let_opcodes::RET => {
                if cp == 0 {
                    break;
                }
                if sp ==0 || sp >= stack.len() {
                    panic!("Stack overflow");
                }
                let result = stack[sp - 1];
                sp = locals - 1;
                if sp >= stack.len() {
                    panic!("Stack overflow");
                }
                stack[sp] = result;
                sp += 1;
                let (new_pc, new_locals) = calls.get(cp - 1).unwrap().clone();
                cp -= 1;
                pc = new_pc;
                locals = new_locals;
            }
            let_opcodes::LD1 => {
                if sp >= stack.len() {
                    panic!("Stack overflow");
                }
                let index = opcodes.get(pc + 1).unwrap().clone() as usize;
                if locals + index >= stack.len() {
                    panic!("Stack overflow");
                }
                stack[sp] = stack[locals + index];
                sp += 1;
                pc += 2;
            }
            _ => panic!("Unknown opcode 0x{opcode:02X}"),
        }
    }

    println!("{:?}", stack[0]);
    Ok(())
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
