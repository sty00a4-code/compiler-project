use std::{iter::Peekable, str::Chars};

use super::position::{Located, Position};

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    pub chars: Peekable<Chars<'a>>,
    pub ln: usize,
    pub col: usize,
}
impl<'a> Iterator for Lexer<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.ln += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some(c)
    }
}
impl<'a> Lexer<'a> {
    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    pub fn pos(&self) -> Position {
        Position::new(self.ln, self.col)
    }
}
pub trait Lexable: Sized {
    type Error;
    fn lex_next<'a>(lexer: &mut Lexer<'a>) -> Result<Option<Located<Self>>, Located<Self::Error>>;
    fn lex(text: &str) -> Result<Vec<Located<Self>>, Located<Self::Error>> {
        let mut lexer = Lexer {
            chars: text.chars().peekable(),
            ln: 0,
            col: 0,
        };
        let mut tokens = vec![];
        while let Some(token) = Self::lex_next(&mut lexer)? {
            tokens.push(token)
        }
        Ok(tokens)
    }
}
