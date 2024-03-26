use lexer::Lexable;
use position::Located;
use tokens::{LexError, Token};

pub mod lexer;
pub mod position;
pub mod tokens;

pub fn lex(text: &str) -> Result<Vec<Located<Token>>, Located<LexError>> {
    Token::lex(text)
}
