use std::{iter::Peekable, vec::IntoIter};
use crate::lexer::{position::Located, tokens::Token};

pub type Parser = Peekable<IntoIter<Located<Token>>>;
pub trait Parsable: Sized {
    type Error;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>>;
}