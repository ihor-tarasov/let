pub mod assembler;
pub mod compiler;
pub mod error;

mod parser;
mod read_iter;
mod to_assembly_compiler;
mod to_object_compiler;

pub use parser::*;
pub use read_iter::ReadIter;
pub use to_assembly_compiler::ToAssemblerCompiler;
pub use to_object_compiler::ToObjectCompiler;

pub mod string_array; // TODO: private
pub mod opcodes; // TODO: private

mod lexer;
mod token;
mod operators;
mod precedence;
