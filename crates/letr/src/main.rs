use std::fs::File;

#[derive(Debug, Clone, Copy)]
enum Value {
    Void,
    Boolean(bool),
    Integer(i64),
    Address(u32),
}

struct State {
    pc: u32,
    stack: [Value; 256],
    calls: [(u32, u32); 256],
    cp: u32,
    sp: u32,
    locals: u32,
    message: Option<String>,
}

#[derive(Debug)]
enum VMError {
    StackUnderflow,
    StackOverflow,
    FetchOpcodeError,
    Custom,
}

type VMResult<T = ()> = Result<T, VMError>;

impl State {
    /// Pop one element from stack and forget them.
    /// Raise error if stack is empty.
    fn drop(&mut self) -> VMResult {
        if self.sp == 0 {
            Err(VMError::StackUnderflow)
        } else {
            self.sp -= 1;
            Ok(())
        }
    }

    /// Push value to stack.
    fn push(&mut self, value: Value) -> VMResult {
        if self.sp >= self.stack.len() as u32 {
            Err(VMError::StackOverflow)
        } else {
            self.stack[self.sp as usize] = value;
            self.sp += 1;
            Ok(())
        }
    }

    /// Pop value from stack.
    fn pop(&mut self) -> VMResult<Value> {
        if self.sp == 0 {
            Err(VMError::StackUnderflow)
        } else if self.sp >= self.stack.len() as u32 {
            Err(VMError::StackOverflow)
        } else {
            self.sp -= 1;
            Ok(self.stack[self.sp as usize])
        }
    }

    /// Fetch next one byte from opcodes.
    fn fetch(&self, opcodes: &[u8], offset: u32) -> VMResult<u8> {
        if let Some(opcode) = opcodes.get((self.pc + offset) as usize).cloned() {
            Ok(opcode)
        } else {
            Err(VMError::FetchOpcodeError)
        }
    }

    fn error<T>(&mut self, m: String) -> VMResult<T> {
        self.message = Some(m);
        Err(VMError::Custom)
    }

    fn bin_ls(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_le(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_gr(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_eq(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_add(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_add(r))),
            _ => self.error(format!("Unable to addict {l:?} and {r:?} values.")),
        }
    }

    fn bin_sub(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_sub(r))),
            _ => self.error(format!("Unable to addict {l:?} and {r:?} values.")),
        }
    }

    fn bin_mul(&mut self, l: Value, r: Value) -> VMResult<Value> {
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_mul(r))),
            _ => self.error(format!("Unable to addict {l:?} and {r:?} values.")),
        }
    }

    /// Execute functor F as binary operator for this state.
    fn binary<F>(&mut self, f: F) -> VMResult
    where
        F: Fn(&mut Self, Value, Value) -> VMResult<Value>,
    {
        if self.sp < 2 || self.sp > self.stack.len() as u32 {
            return Err(VMError::StackOverflow);
        }
        self.sp -= 1;
        self.stack[(self.sp - 1) as usize] = f(
            self,
            self.stack[(self.sp - 1) as usize],
            self.stack[(self.sp) as usize],
        )?;
        Ok(())
    }

    fn op_drop(&mut self) -> VMResult<bool> {
        self.drop()?; // Do main stuff.
        self.pc += 1; // Skip one byte, Size of opcode = one byte.
        Ok(true)
    }

    fn op_binary<F>(&mut self, f: F) -> VMResult<bool>
    where
        F: Fn(&mut Self, Value, Value) -> VMResult<Value>,
    {
        self.binary(f)?;
        self.pc += 1;
        Ok(true)
    }

    fn op_int1(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        self.push(Value::Integer(self.fetch(opcodes, 1)? as i64))?;
        self.pc += 2;
        Ok(true)
    }

    fn op_ptr(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        self.push(Value::Address(u32::from_be_bytes([
            self.fetch(opcodes, 1)?,
            self.fetch(opcodes, 2)?,
            self.fetch(opcodes, 3)?,
            self.fetch(opcodes, 4)?,
        ])))?;
        self.pc += 5;
        Ok(true)
    }

    fn op_jpf(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let value = self.pop()?;
        match value {
            Value::Boolean(value) => {
                if !value {
                    self.pc = u32::from_be_bytes([
                        self.fetch(opcodes, 1)?,
                        self.fetch(opcodes, 2)?,
                        self.fetch(opcodes, 3)?,
                        self.fetch(opcodes, 4)?,
                    ]);
                    Ok(true)
                } else {
                    self.pc += 5;
                    Ok(true)
                }
            }
            _ => self.error(format!("Expected bool value, found {value:?}.")),
        }
    }

    fn op_jp(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        self.pc = u32::from_be_bytes([
            self.fetch(opcodes, 1)?,
            self.fetch(opcodes, 2)?,
            self.fetch(opcodes, 3)?,
            self.fetch(opcodes, 4)?,
        ]);
        Ok(true)
    }

    fn op_call(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let params_count = self.fetch(opcodes, 1)?;
        if self.sp < params_count as u32 + 1 {
            return Err(VMError::StackUnderflow);
        }
        let (save_pc, save_locals) = match self.calls.get_mut(self.cp as usize) {
            Some(d) => d,
            None => return Err(VMError::StackOverflow),
        };
        self.cp += 1;
        *save_pc = self.pc + 2;
        *save_locals = self.locals;
        let address = self.stack[(self.sp - params_count as u32 - 1) as usize];
        match address {
            Value::Address(address) => self.pc = address,
            _ => return self.error(format!("Expected address, found {address:?}")),
        }
        self.locals = self.sp - params_count as u32;
        Ok(true)
    }

    fn op_ret(&mut self) -> VMResult<bool> {
        if self.cp == 0 {
            return Ok(false);
        }
        let result = self.pop()?;
        self.sp = self.locals - 1;
        self.push(result)?;
        let (new_pc, new_locals) = match self.calls.get((self.cp - 1) as usize) {
            Some(d) => d.clone(),
            None => return Err(VMError::StackOverflow),
        };
        self.cp -= 1;
        self.pc = new_pc;
        self.locals = new_locals;
        Ok(true)
    }

    fn op_ld1(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let index = self.fetch(opcodes, 1)?;
        if self.locals + index as u32 >= self.stack.len() as u32 {
            panic!("Stack overflow");
        }
        self.push(self.stack[(self.locals + index as u32) as usize])?;
        self.pc += 2;
        Ok(true)
    }

    /// Executes one opcode.
    /// Returns Ok(false) if VM is stopped.
    fn step(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let opcode = self.fetch(opcodes, 0)?;
        match opcode {
            let_opcodes::DROP => self.op_drop(),
            let_opcodes::LS => self.op_binary(Self::bin_ls),
            let_opcodes::GR => self.op_binary(Self::bin_gr),
            let_opcodes::EQ => self.op_binary(Self::bin_eq),
            let_opcodes::ADD => self.op_binary(Self::bin_add),
            let_opcodes::LE => self.op_binary(Self::bin_le),
            let_opcodes::SUB => self.op_binary(Self::bin_sub),
            let_opcodes::MUL => self.op_binary(Self::bin_mul),
            let_opcodes::INT1 => self.op_int1(opcodes),
            let_opcodes::PTR => self.op_ptr(opcodes),
            let_opcodes::JPF => self.op_jpf(opcodes),
            let_opcodes::JP => self.op_jp(opcodes),
            let_opcodes::CALL => self.op_call(opcodes),
            let_opcodes::RET => self.op_ret(),
            let_opcodes::LD1 => self.op_ld1(opcodes),
            _ => self.error(format!("Unknown opcode 0x{opcode:02X}")),
        }
    }

    fn run(&mut self, opcodes: &[u8]) -> VMResult<Value> {
        while self.step(opcodes)? {}
        self.pop()
    }
}

fn run<R>(read: &mut R) -> let_result::Result
where
    R: std::io::Read,
{
    let module = let_module::Module::read(read)?;

    let mut state = State {
        pc: 0,
        stack: [Value::Void; 256],
        calls: [(0, 0); 256],
        cp: 0,
        sp: 0,
        locals: 0,
        message: None,
    };

    if let Some(pc) = module.labels.get(b"main.__ctor__") {
        state.pc = pc;
    } else {
        panic!();
    }

    let result = state.run(&module.opcodes).unwrap();

    println!("{:?}", result);
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
