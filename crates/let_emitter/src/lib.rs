pub use let_result::Result;

pub trait Emitter {
    fn integer(&mut self, value: u64) -> let_result::Result;
    fn real(&mut self, value: f64) -> let_result::Result;
    fn load(&mut self, index: u32);
    fn pointer(&mut self, name: &[u8]) -> let_result::Result;
    fn call(&mut self, arguments: u8) -> let_result::Result;
    fn binary(&mut self, operator: [u8; 3]) -> let_result::Result;
    fn ret(&mut self) -> let_result::Result;
    fn drop(&mut self) -> let_result::Result;
    fn label(&mut self, id: u32) -> let_result::Result;
    fn label_named(&mut self, lable: &[u8]) -> let_result::Result;
    fn jump(&mut self, id: u32) -> let_result::Result;
    fn jump_name(&mut self, name: &[u8]) -> let_result::Result;
    fn jump_false(&mut self, id: u32) -> let_result::Result;
    fn jump_false_name(&mut self, name: &[u8]) -> let_result::Result;

    // Returns pointer to stack size.
    fn function(&mut self, args_count: u8) -> let_result::Result<u32>;

    fn set(&mut self, address: u32, value: u8);
    fn store(&mut self, local_id: u32);
    fn offset(&self) -> u32;
    fn jump_to(&mut self, address: u32);
    fn void(&mut self);
    fn list(&mut self);
}
