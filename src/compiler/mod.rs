use self::compiler::{Compilable, Compiler};

pub mod bytecode;
pub mod compiler;

pub fn compile<A: Compilable>(ast: A) -> Result<A::Output, A::Error> {
    ast.compile(&mut Compiler::default())
}
