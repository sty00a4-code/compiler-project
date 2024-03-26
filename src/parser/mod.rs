use crate::lexer::{position::Located, tokens::Token};
use ast::{Chunk, ParseError};
use parser::Parsable;

pub mod ast;
pub mod parser;

pub fn parse(tokens: Vec<Located<Token>>) -> Result<Located<Chunk>, Located<ParseError>> {
    Chunk::parse(&mut tokens.into_iter().peekable())
}
