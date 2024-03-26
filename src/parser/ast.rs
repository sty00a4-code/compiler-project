use super::parser::{Parsable, Parser};
use crate::lexer::{
    position::{Located, Position},
    tokens::Token,
};
use std::fmt::Display;

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
        args: Vec<Located<Expression>>,
    },

    Def {
        ident: Located<String>,
        params: Vec<Located<String>>,
        body: Located<Block>,
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
    
    Return(Located<Expression>),
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
    Expression(Box<Located<Expression>>),
}

macro_rules! expected {
    ($parser:ident) => {{
        let Some(token) = $parser.next() else {
            return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
        };
        token
    }};
    ($parser:ident : &) => {{
        let Some(token) = $parser.peek() else {
            return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
        };
        token
    }};
    ($parser:ident : $token:ident) => {{
        let Some(Located { value: token, pos }) = $parser.next() else {
            return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
        };
        if token != Token::$token {
            return Err(Located::new(
                ParseError::ExpectedToken {
                    expected: Token::$token,
                    got: token,
                },
                pos,
            ));
        }
        Located { value: token, pos }
    }};
}
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(Token),
    ExpectedToken { expected: Token, got: Token },
}

impl Parsable for Chunk {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let mut stats = vec![];
        while parser.peek().is_some() {
            stats.push(Statement::parse(parser)?);
        }
        Ok(Located::new(Self(stats), Position::default()))
    }
}
impl Parsable for Block {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let Located { value: _, pos } = expected!(parser: BraceLeft);
        let mut stats = vec![];
        while let Some(Located {
            value: token,
            pos: _,
        }) = parser.peek()
        {
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
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        let Located { value: token, pos } = expected!(parser:&);
        match token {
            Token::Let => {
                let Located { value: _, pos } = expected!(parser);
                let ident = Atom::ident(parser)?;
                expected!(parser: Equal);
                let expr = Expression::parse(parser)?;
                Ok(Located::new(Self::Let { ident, expr }, pos))
            }
            Token::Return => {
                let Located { value: _, pos } = expected!(parser);
                let expr = Expression::parse(parser)?;
                Ok(Located::new(Self::Return(expr), pos))
            }
            Token::Ident(_) => {
                let ident = Atom::ident(parser)?;
                let Located { value: token, pos } = expected!(parser);
                match token {
                    Token::Equal => {
                        let expr = Expression::parse(parser)?;
                        Ok(Located::new(Self::Assign { ident, expr }, pos))
                    }
                    Token::ParanLeft => {
                        let mut args = vec![];
                        while let Some(Located {
                            value: token,
                            pos: _,
                        }) = parser.peek()
                        {
                            if token == &Token::ParanRight {
                                break;
                            }
                            args.push(Expression::parse(parser)?);
                        }
                        expected!(parser: ParanRight);
                        Ok(Located::new(Self::Call { ident, args }, pos))
                    }
                    token => Err(Located::new(ParseError::UnexpectedToken(token), pos))
                }
            }
            _ => Err(Located::new(
                ParseError::UnexpectedToken(token.clone()),
                pos.clone(),
            )),
        }
    }
}
impl Parsable for Expression {
    type Error = ParseError;
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<Self::Error>> {
        Self::call(parser)
    }
}
impl Expression {
    pub fn binary(parser: &mut Parser, layer: usize) -> Result<Located<Self>, Located<ParseError>> {
        todo!()
    }
    pub fn unary(parser: &mut Parser, layer: usize) -> Result<Located<Self>, Located<ParseError>> {
        todo!()
    }
    pub fn call(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        let mut head = Atom::parse(parser)?.map(Self::Atom);
        while let Some(Located {
            value: token,
            pos: _,
        }) = parser.peek()
        {
            match token {
                Token::ParanLeft => {
                    parser.next();
                    let pos = head.pos.clone();
                    let mut args = vec![];
                    while let Some(Located {
                        value: token,
                        pos: _,
                    }) = parser.peek()
                    {
                        if token == &Token::ParanRight {
                            break;
                        }
                        args.push(Expression::parse(parser)?);
                    }
                    expected!(parser: ParanRight);
                    head = Located::new(
                        Self::Call {
                            head: Box::new(head),
                            args,
                        },
                        pos,
                    )
                }
                _ => break,
            }
        }
        Ok(head)
    }
}
impl Parsable for Atom {
    type Error = ParseError;
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
            token => Err(Located::new(ParseError::UnexpectedToken(token), pos)),
        }
    }
}
impl Atom {
    pub fn ident(parser: &mut Parser) -> Result<Located<String>, Located<ParseError>> {
        let Located { value: token, pos } = expected!(parser);
        if let Token::Ident(ident) = token {
            Ok(Located::new(ident, pos))
        } else {
            Err(Located::new(
                ParseError::ExpectedToken {
                    expected: Token::Ident(Default::default()),
                    got: token,
                },
                pos,
            ))
        }
    }
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEOF => write!(f, "unexpected end of file"),
            ParseError::UnexpectedToken(token) => write!(f, "unexpected token {}", token.name()),
            ParseError::ExpectedToken { expected, got } => {
                write!(f, "expected {}, got {}", expected.name(), got.name())
            }
        }
    }
}
