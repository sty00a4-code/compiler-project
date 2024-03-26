use std::fmt::Display;

use crate::lexer::{position::{Located, Position}, tokens::Token};

use super::parser::{Parsable, Parser};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chunk(pub Vec<Located<Statement>>);
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Block(pub Vec<Located<Statement>>);

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Block(Block),
    Let {
        ident: Located<String>,
        expr: Located<Expression>,
    },
    Assign {
        ident: Located<String>,
        expr: Located<Expression>,
    },
    Call {
        ident: Located<String>,
        args: Vec<Located<Self>>,
    },

    Def {
        ident: Located<String>,
        params: Vec<Located<String>>,
        body: Located<Block>
    },
    If {
        cond: Located<Expression>,
        case: Located<Block>,
        else_case: Option<Located<Block>>,
    },
    While {
        cond: Located<Expression>,
        body: Located<Block>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Atom(Atom),
    Binary {
        op: BinaryOperator,
        left: Box<Located<Self>>,
        right: Box<Located<Self>>,
    },
    Unary {
        op: UnaryOperator,
        right: Box<Located<Self>>,
    },
    Call {
        head: Box<Located<Self>>,
        args: Vec<Located<Self>>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Plus,             // +
    Minus,            // -
    Star,             // *
    Slash,            // /
    Percent,          // %
    Exponent,         // ^
    EqualEqual,       // ==
    ExclamationEqual, // !=
    Less,             // <
    Greater,          // >
    LessEqual,        // <=
    GreaterEqual,     // >=
    Ampersand,        // &
    Pipe,             // |
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Minus,       // -
    Exclamation, // !
}
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Ident(String),
    Number(f64),
    String(String),
    Expression(Box<Located<Expression>>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    UnexpectedEOF,
    UnexpectedToken(Token),
    ExpectedToken {
        expected: Token,
        got: Token,
    },
}

macro_rules! expected {
    ($parser:ident) => {
        {
            let Some(token) = $parser.next() else {
                return Err(Located::new(ParserError::UnexpectedEOF, Position::default()))
            };
            token
        }
    };
    ($parser:ident : &) => {
        {
            let Some(token) = $parser.peek() else {
                return Err(Located::new(ParserError::UnexpectedEOF, Position::default()))
            };
            token
        }
    };
    ($parser:ident : $token:ident) => {
        {
            let Some(Located { value: token, pos }) = $parser.next() else {
                return Err(Located::new(ParserError::UnexpectedEOF, Position::default()))
            };
            if token != Token::$token {
                return Err(Located::new(ParserError::ExpectedToken {
                    expected: Token::$token,
                    got: token,
                }, pos))
            }
            Located { value: token, pos }
        }
    };
}
impl Parsable for Chunk {
    type Error = ParserError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let mut stats = vec![];
        while parser.peek().is_some() {
            stats.push(Statement::parse(parser)?);
        }
        Ok(Located::new(Self(stats), Position::default()))
    }
}
impl Parsable for Block {
    type Error = ParserError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let Located { value: _, pos } = expected!(parser: BraceLeft);
        let mut stats = vec![];
        while let Some(Located { value: token, pos: _ }) = parser.peek() {
            if token == &Token::BraceRight {
                break;
            }
            stats.push(Statement::parse(parser)?);
        }
        expected!(parser: BraceRight);
        Ok(Located::new(Self(stats), pos))
    }
}
impl Parsable for Statement {
    type Error = ParserError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let Located { value: token, pos } = expected!(parser:&);
        match token {
            
            _ => Err(Located::new(ParserError::UnexpectedToken(token.clone()), pos.clone()))
        }
    }
}
impl Parsable for Expression {
    type Error = ParserError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        Self::call(parser)
    }
}
impl Expression {
    pub fn binary(parser: &mut Parser, layer: usize) -> Result<Located<Self>, Located<ParserError>> {
        todo!()
    }
    pub fn unary(parser: &mut Parser, layer: usize) -> Result<Located<Self>, Located<ParserError>> {
        todo!()
    }
    pub fn call(parser: &mut Parser) -> Result<Located<Self>, Located<ParserError>> {
        let mut head = Atom::parse(parser)?.map(Self::Atom);
        while let Some(Located { value: token, pos: _ }) = parser.peek() {
            match token {
                Token::ParanLeft => {
                    parser.next();
                    let pos = head.pos.clone();
                    let mut args = vec![];
                    while let Some(Located { value: token, pos: _ }) = parser.peek() {
                        if token == &Token::ParanRight {
                            break;
                        }
                        args.push(Expression::parse(parser)?);
                    }
                    expected!(parser: ParanRight);
                    head = Located::new(Self::Call { head: Box::new(head), args }, pos)
                }
                _ => break,
            }
        }
        Ok(head)
    }
}
impl Parsable for Atom {
    type Error = ParserError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let Located { value: token, pos } = expected!(parser);
        match token {
            Token::Ident(ident) => Ok(Located::new(Self::Ident(ident), pos)),
            Token::Number(number) => Ok(Located::new(Self::Number(number), pos)),
            Token::String(string) => Ok(Located::new(Self::String(string), pos)),
            Token::ParanLeft => {
                let expr = Expression::parse(parser)?;
                expected!(parser: ParanRight);
                Ok(Located::new(Self::Expression(Box::new(expr)), pos))
            }
            token => Err(Located::new(ParserError::UnexpectedToken(token), pos))
        }
    }
}
impl Atom {

}
impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::UnexpectedEOF => write!(f, "unexpected end of file"),
            ParserError::UnexpectedToken(token) => write!(f, "unexpected token {}", token.name()),
            ParserError::ExpectedToken { expected, got } => write!(f, "expected {}, got {}", expected.name(), got.name()),
        }
    }
}
