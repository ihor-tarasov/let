use core::fmt;
use std::fs::File;

const DUMP_OPCODE: bool = true;
const DUMP_STACK: bool = true;

macro_rules! dumpop {
    () => {
        if DUMP_OPCODE {
            println!();
        }
    };
    ($($arg:tt)*) => {{
        if DUMP_OPCODE {
            println!($($arg)*);
        }
    }};
}

fn fetch_u8(opcodes: &[u8], offset: u32) -> VMResult<u8> {
    if let Some(opcode) = opcodes.get(offset as usize).cloned() {
        Ok(opcode)
    } else {
        Err(VMError::FetchOpcodeError)
    }
}

fn fetch_u32(opcodes: &[u8], offset: u32) -> VMResult<u32> {
    Ok(u32::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, offset + 1)?,
        fetch_u8(opcodes, offset + 2)?,
        fetch_u8(opcodes, offset + 3)?,
    ]))
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Void,
    Boolean(bool),
    Integer(i64),
    Address(u32),
    CallState(u32, u32),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Void => write!(f, "()"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Integer(value) => write!(f, "{value}"),
            Value::Address(value) => write!(f, "{value}"),
            Value::CallState(pc, locals) => write!(f, "(PC:{pc} LC:{locals})"),
        }
    }
}

const STACK_SIZE: usize = 32;

struct State {
    pc: u32,
    stack: [Value; STACK_SIZE],
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

    /// Peek last value in stack.
    fn peek(&mut self) -> VMResult<Value> {
        if self.sp == 0 {
            Err(VMError::StackUnderflow)
        } else if self.sp >= self.stack.len() as u32 {
            Err(VMError::StackOverflow)
        } else {
            Ok(self.stack[(self.sp - 1) as usize])
        }
    }

    fn error<T>(&mut self, m: String) -> VMResult<T> {
        self.message = Some(m);
        Err(VMError::Custom)
    }

    fn bin_ls(&mut self, l: Value, r: Value) -> VMResult<Value> {
        dumpop!("LS");
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_le(&mut self, l: Value, r: Value) -> VMResult<Value> {
        dumpop!("LE");
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_gr(&mut self, l: Value, r: Value) -> VMResult<Value> {
        dumpop!("GR");
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_eq(&mut self, l: Value, r: Value) -> VMResult<Value> {
        dumpop!("EQ");
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
            _ => self.error(format!("Unable to compare {l:?} and {r:?} values.")),
        }
    }

    fn bin_add(&mut self, l: Value, r: Value) -> VMResult<Value> {
        dumpop!("ADD");
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_add(r))),
            _ => self.error(format!("Unable to addict {l:?} and {r:?} values.")),
        }
    }

    fn bin_sub(&mut self, l: Value, r: Value) -> VMResult<Value> {
        dumpop!("SUB");
        match (l, r) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_sub(r))),
            _ => self.error(format!("Unable to addict {l:?} and {r:?} values.")),
        }
    }

    fn bin_mul(&mut self, l: Value, r: Value) -> VMResult<Value> {
        dumpop!("MUL");
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
        dumpop!("DROP");
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

    fn op_void(&mut self) -> VMResult<bool> {
        dumpop!("VOID");
        self.push(Value::Void)?;
        self.pc += 1;
        Ok(true)
    }

    fn op_int1(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let val = fetch_u8(opcodes, self.pc + 1)?;
        dumpop!("INT {val}");
        self.push(Value::Integer(val as i64))?;
        self.pc += 2;
        Ok(true)
    }

    fn op_ptr(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let address = fetch_u32(opcodes, self.pc + 1)?;
        dumpop!("PTR {address}");
        self.push(Value::Address(address))?;
        self.pc += 5;
        Ok(true)
    }

    fn op_jpf(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let value = self.pop()?;
        match value {
            Value::Boolean(value) => {
                if !value {
                    self.pc = fetch_u32(opcodes, self.pc + 1)?;
                    dumpop!("JPF {}", self.pc);
                    Ok(true)
                } else {
                    dumpop!(
                        "JPF {}",
                        fetch_u32(opcodes, self.pc + 1)?,
                    );
                    self.pc += 5;
                    Ok(true)
                }
            }
            _ => self.error(format!("Expected bool value, found {value:?}.")),
        }
    }

    fn op_jp(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        self.pc = fetch_u32(opcodes, self.pc + 1)?;
        dumpop!("JP {}", self.pc);
        Ok(true)
    }

    fn op_call(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let params_count = fetch_u8(opcodes, self.pc + 1)?;
        dumpop!("CALL {params_count}");
        if self.sp < params_count as u32 + 1 {
            return Err(VMError::StackUnderflow);
        }
        let in_stack_offset = self.sp - params_count as u32 - 1;
        let address = self.stack[in_stack_offset as usize];
        self.stack[in_stack_offset as usize] = Value::CallState(self.pc + 2, self.locals);
        match address {
            Value::Address(address) => self.pc = address,
            _ => return self.error(format!("Expected address, found {address:?}")),
        }
        self.locals = self.sp - params_count as u32;
        let params_count_for_check = fetch_u8(opcodes, self.pc)?;
        if params_count != params_count_for_check {
            return self.error(format!("Expected {params_count_for_check} function call arguments, found {params_count}."));
        }
        let stack_size = fetch_u32(opcodes, self.pc + 1)?;
        self.sp += stack_size;
        dumpop!("Call info: parameters count: {params_count}, stack_size: {stack_size}");
        self.pc += 5;
        Ok(true)
    }

    fn op_ret(&mut self) -> VMResult<bool> {
        dumpop!("RET");
        if self.locals == 0 {
            return Ok(false);
        }
        let result = self.pop()?;
        self.sp = self.locals - 1;
        let call_state = self.stack[self.sp as usize];
        match call_state {
            Value::CallState(new_pc, new_locals) => {
                self.push(result)?;
                self.pc = new_pc;
                self.locals = new_locals;
            }
            _ => return self.error(format!("Expected CallState, found {call_state:?}")),
        }
        Ok(true)
    }

    fn op_ld1(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let index = fetch_u8(opcodes, self.pc + 1)?;
        dumpop!("LD {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            panic!("Stack overflow");
        }
        self.push(self.stack[(self.locals + index as u32) as usize])?;
        self.pc += 2;
        Ok(true)
    }

    fn op_st1(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let index = fetch_u8(opcodes, self.pc + 1)?;
        dumpop!("ST {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            panic!("Stack overflow");
        }
        self.stack[(self.locals + index as u32) as usize] = self.peek()?;
        self.pc += 2;
        Ok(true)
    }

    /// Executes one opcode.
    /// Returns Ok(false) if VM is stopped.
    fn step(&mut self, opcodes: &[u8]) -> VMResult<bool> {
        let opcode = fetch_u8(opcodes, self.pc)?;
        match opcode {
            let_opcodes::DROP => self.op_drop(),
            let_opcodes::LS => self.op_binary(Self::bin_ls),
            let_opcodes::GR => self.op_binary(Self::bin_gr),
            let_opcodes::EQ => self.op_binary(Self::bin_eq),
            let_opcodes::ADD => self.op_binary(Self::bin_add),
            let_opcodes::LE => self.op_binary(Self::bin_le),
            let_opcodes::SUB => self.op_binary(Self::bin_sub),
            let_opcodes::MUL => self.op_binary(Self::bin_mul),
            let_opcodes::VOID => self.op_void(),
            let_opcodes::INT1 => self.op_int1(opcodes),
            let_opcodes::PTR => self.op_ptr(opcodes),
            let_opcodes::JPF => self.op_jpf(opcodes),
            let_opcodes::JP => self.op_jp(opcodes),
            let_opcodes::CALL => self.op_call(opcodes),
            let_opcodes::RET => self.op_ret(),
            let_opcodes::LD1 => self.op_ld1(opcodes),
            let_opcodes::ST1 => self.op_st1(opcodes),
            _ => self.error(format!("Unknown opcode 0x{opcode:02X}")),
        }
    }

    fn dump_stack(&self) {
        if self.sp == 0 {
            print!("[]");
        }
        for i in 0..self.sp {
            print!("[ {} ] ", self.stack[i as usize]);
        }
        println!();
    }

    fn run(&mut self, opcodes: &[u8]) -> VMResult<Value> {
        while self.step(opcodes)? {
            if DUMP_STACK {
                self.dump_stack();
                println!();
            }
        }
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
        stack: [Value::Void; STACK_SIZE],
        sp: 0,
        locals: 0,
        message: None,
    };

    if let Some(pc) = module.labels.get(b"main") {
        state.pc = pc;
    } else {
        return let_result::raise!("Unable to find \"main\" module.");
    }

    match state.run(&module.opcodes) {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(error) => match error {
            VMError::StackUnderflow => let_result::raise!("Stack underflow."),
            VMError::StackOverflow => let_result::raise!("Stack overflow."),
            VMError::FetchOpcodeError => let_result::raise!("Fetch opcode error."),
            VMError::Custom => let_result::raise!("{}", state.message.unwrap()),
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
