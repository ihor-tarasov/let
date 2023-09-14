use parser::Parser;
use print_compiler::PrintCompiler;

mod read_iter;
mod string_array;
mod token;
mod operators;
mod lexer;
mod precedence;
mod error;
mod compiler;
mod parser;
mod print_compiler;

fn main() {
    let code = r"
    
    fn add(a b)
        if a < 2
            a
        elif a > 5
            b
        elif a == 4
            c
        end
    end

    print(add(2 4))

    ";

    let mut parser = Parser::new(code.as_bytes().iter().cloned(), PrintCompiler);

    parser.parse().unwrap();
}
