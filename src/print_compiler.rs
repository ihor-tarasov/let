use crate::compiler::{Compiler, CompilerResult};

pub struct PrintCompiler;

fn to_string(s: &[u8]) -> String {
    let mut result = String::new();
    for c in s {
        if *c != b'\0' {
            result.push(*c as char);
        }
    }
    result
}

impl Compiler for PrintCompiler {
    fn integer(&mut self, value: i64) -> CompilerResult {
        println!("  INTEGER {value}");
        Ok(())
    }

    fn real(&mut self, value: f64) -> CompilerResult {
        println!("  REAL {value}");
        Ok(())
    }

    fn variable(&mut self, name: &[u8]) -> CompilerResult {
        println!("  LOAD {}", to_string(name));
        Ok(())
    }

    fn call(&mut self, arguments: usize) -> CompilerResult {
        println!("  CALL {arguments}");
        Ok(())
    }

    fn binary(&mut self, operator: [u8; 3]) -> CompilerResult {
        println!("  OPERATOR[{}]", to_string(&operator));
        Ok(())
    }

    fn start_function(&mut self, name: &[u8]) -> CompilerResult {
        println!("{}:", to_string(name));
        println!("#FUNCTION({})", to_string(name));
        Ok(())
    }

    fn argument(&mut self, name: &[u8]) -> CompilerResult {
        println!("#ARGUMENT({})", to_string(name));
        Ok(())
    }

    fn end_function(&mut self) -> CompilerResult {
        println!("  RET");
        Ok(())
    }

    fn end_of_statement(&mut self) -> CompilerResult {
        println!("  DROP");
        Ok(())
    }

    fn extern_symbol(&mut self) -> CompilerResult {
        println!("#EXTERN");
        Ok(())
    }

    fn lable(&mut self, id: usize) -> CompilerResult {
        println!("lable_{id}:");
        Ok(())
    }

    fn jump(&mut self, id: usize) -> CompilerResult {
        println!("  JUMP lable_{id}");
        Ok(())
    }

    fn jump_false(&mut self, id: usize) -> CompilerResult {
        println!("  JUMP_FALSE lable_{id}");
        Ok(())
    }
}
