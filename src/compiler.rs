use crate::error::Error;

pub type CompilerResult = Result<(), Error>;

pub trait Compiler {
    fn integer(&mut self, value: u64) -> CompilerResult;
    fn real(&mut self, value: f64) -> CompilerResult;
    fn load(&mut self, index: u64) -> CompilerResult;
    fn pointer(&mut self, name: &[u8]) -> CompilerResult;
    fn call(&mut self, arguments: u8) -> CompilerResult;
    fn binary(&mut self, operator: [u8; 3]) -> CompilerResult;
    fn ret(&mut self) -> CompilerResult;
    fn end_of_statement(&mut self) -> CompilerResult;
    fn lable(&mut self, id: u64) -> CompilerResult;
    fn lable_named(&mut self, lable: &[u8]) -> CompilerResult;
    fn jump(&mut self, id: u64) -> CompilerResult;
    fn jump_name(&mut self, name: &[u8]) -> CompilerResult;
    fn jump_false(&mut self, id: u64) -> CompilerResult;
    fn jump_false_name(&mut self, name: &[u8]) -> CompilerResult;
}
