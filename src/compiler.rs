use crate::error::Error;

pub type CompilerResult = Result<(), Error>;

pub trait Compiler {
    fn integer(&mut self, value: i64) -> CompilerResult;
    fn real(&mut self, value: f64) -> CompilerResult;
    fn variable(&mut self, name: &[u8]) -> CompilerResult;
    fn call(&mut self, arguments: usize) -> CompilerResult;
    fn binary(&mut self, operator: [u8; 3]) -> CompilerResult;
    fn start_function(&mut self, name: &[u8]) -> CompilerResult;
    fn argument(&mut self, name: &[u8]) -> CompilerResult;
    fn end_function(&mut self) -> CompilerResult;
    fn end_of_statement(&mut self) -> CompilerResult;
    fn extern_symbol(&mut self) -> CompilerResult;
    fn lable(&mut self, id: usize) -> CompilerResult;
    fn jump(&mut self, id: usize) -> CompilerResult;
    fn jump_false(&mut self, id: usize) -> CompilerResult;
}
