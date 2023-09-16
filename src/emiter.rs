use crate::error::Error;

pub type EmiterResult = Result<(), Error>;

pub trait Emiter {
    fn integer(&mut self, value: u64) -> EmiterResult;
    fn real(&mut self, value: f64) -> EmiterResult;
    fn load(&mut self, index: u64) -> EmiterResult;
    fn pointer(&mut self, name: &[u8]) -> EmiterResult;
    fn call(&mut self, arguments: u8) -> EmiterResult;
    fn binary(&mut self, operator: [u8; 3]) -> EmiterResult;
    fn ret(&mut self) -> EmiterResult;
    fn end_of_statement(&mut self) -> EmiterResult;
    fn lable(&mut self, id: u64) -> EmiterResult;
    fn lable_named(&mut self, lable: &[u8]) -> EmiterResult;
    fn jump(&mut self, id: u64) -> EmiterResult;
    fn jump_name(&mut self, name: &[u8]) -> EmiterResult;
    fn jump_false(&mut self, id: u64) -> EmiterResult;
    fn jump_false_name(&mut self, name: &[u8]) -> EmiterResult;
}
