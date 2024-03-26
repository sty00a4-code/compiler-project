use super::{
    lexer::{Lexable, Lexer},
    position::Located,
};
use std::{fmt::Display, num::ParseFloatError};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // word kinds
    Ident(String),
    Number(f64),
    String(String),

    // symbols
    Equal,            // =
    Comma,            // ,
    Dot,              // .
    ParanLeft,        // (
    ParanRight,       // )
    BracketLeft,      // [
    BracketRight,     // ]
    BraceLeft,        // {
    BraceRight,       // }
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
    Exclamation,      // !

    // key words
    Let,
    Def,
    If,
    Else,
    While,
    For,
}
impl Token {
    fn ident(ident: String) -> Self {
        match ident.as_str() {
            "let" => Self::Let,
            "def" => Self::Def,
            "if" => Self::If,
            "else" => Self::Else,
            "while" => Self::While,
            "for" => Self::For,
            _ => Self::Ident(ident),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Self::Ident(_) => "identifier".to_string(),
            Self::Number(_) => "number".to_string(),
            Self::String(_) => "string".to_string(),
            _ => format!("{:?}", self.to_string())
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => write!(f, "{ident}"),
            Self::Number(number) => write!(f, "{number}"),
            Self::String(string) => write!(f, "{string:?}"),
            Self::Equal => write!(f, "="),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::ParanLeft => write!(f, "("),
            Self::ParanRight => write!(f, ")"),
            Self::BracketLeft => write!(f, "["),
            Self::BracketRight => write!(f, "]"),
            Self::BraceLeft => write!(f, "{{"),
            Self::BraceRight => write!(f, "}}"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::Exponent => write!(f, "^"),
            Self::EqualEqual => write!(f, "=="),
            Self::ExclamationEqual => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::Greater => write!(f, ">"),
            Self::LessEqual => write!(f, "<="),
            Self::GreaterEqual => write!(f, ">="),
            Self::Ampersand => write!(f, "&"),
            Self::Pipe => write!(f, "|"),
            Self::Exclamation => write!(f, "!"),
            Self::Let => write!(f, "let"),
            Self::Def => write!(f, "def"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::While => write!(f, "while"),
            Self::For => write!(f, "for"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    BadCharacter(char),
    ParseNumberError(ParseFloatError),
    UnclosedString,
}
impl Lexable for Token {
    type Error = LexError;
    fn lex_next<'a>(lexer: &mut Lexer<'a>) -> Result<Option<Located<Self>>, Located<Self::Error>> {
        while let Some(c) = lexer.peek() {
            if !c.is_ascii_whitespace() {
                break;
            }
            lexer.next();
        }
        let pos = lexer.pos();
        let Some(c) = lexer.next() else {
            return Ok(None);
        };
        match c {
            '=' => {
                if lexer.peek() == Some(&'=') {
                    lexer.next();
                    Ok(Some(Located::new(Token::EqualEqual, pos)))
                } else {
                    Ok(Some(Located::new(Token::Equal, pos)))
                }
            }
            ',' => Ok(Some(Located::new(Token::Comma, pos))),
            '.' => Ok(Some(Located::new(Token::Dot, pos))),
            '(' => Ok(Some(Located::new(Token::ParanLeft, pos))),
            ')' => Ok(Some(Located::new(Token::ParanRight, pos))),
            '[' => Ok(Some(Located::new(Token::BracketLeft, pos))),
            ']' => Ok(Some(Located::new(Token::BracketRight, pos))),
            '{' => Ok(Some(Located::new(Token::BraceLeft, pos))),
            '}' => Ok(Some(Located::new(Token::BraceRight, pos))),
            '+' => Ok(Some(Located::new(Token::Plus, pos))),
            '-' => Ok(Some(Located::new(Token::Minus, pos))),
            '*' => Ok(Some(Located::new(Token::Star, pos))),
            '/' => Ok(Some(Located::new(Token::Slash, pos))),
            '%' => Ok(Some(Located::new(Token::Percent, pos))),
            '^' => Ok(Some(Located::new(Token::Exponent, pos))),
            '<' => {
                if lexer.peek() == Some(&'=') {
                    lexer.next();
                    Ok(Some(Located::new(Token::LessEqual, pos)))
                } else {
                    Ok(Some(Located::new(Token::Less, pos)))
                }
            }
            '>' => {
                if lexer.peek() == Some(&'=') {
                    lexer.next();
                    Ok(Some(Located::new(Token::GreaterEqual, pos)))
                } else {
                    Ok(Some(Located::new(Token::Greater, pos)))
                }
            }
            '&' => Ok(Some(Located::new(Token::Ampersand, pos))),
            '|' => Ok(Some(Located::new(Token::Pipe, pos))),
            '!' => {
                if lexer.peek() == Some(&'=') {
                    lexer.next();
                    Ok(Some(Located::new(Token::ExclamationEqual, pos)))
                } else {
                    Ok(Some(Located::new(Token::Exclamation, pos)))
                }
            }
            '"' => {
                let mut string = String::new();
                while let Some(c) = lexer.peek() {
                    if *c == '"' {
                        break;
                    }
                    string.push(lexer.next().unwrap())
                }
                if lexer.next() != Some('"') {
                    return Err(Located::new(LexError::UnclosedString, pos))
                }
                Ok(Some(Located::new(Token::String(string), pos)))
            }
            c if c.is_ascii_digit() => {
                let mut number = String::from(c);
                while let Some(c) = lexer.peek() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    number.push(lexer.next().unwrap())
                }
                if lexer.peek() == Some(&'.') {
                    number.push(lexer.next().unwrap());
                    while let Some(c) = lexer.peek() {
                        if !c.is_ascii_digit() {
                            break;
                        }
                        number.push(lexer.next().unwrap())
                    }
                }
                Ok(Some(Located::new(
                    Token::Number(
                        number
                            .parse()
                            .map_err(LexError::ParseNumberError)
                            .map_err(|err| Located::new(err, pos.clone()))?,
                    ),
                    pos,
                )))
            }
            c if c.is_alphanumeric() || c == '_' => {
                let mut ident = String::from(c);
                while let Some(c) = lexer.peek() {
                    if !c.is_alphanumeric() && *c != '_' {
                        break;
                    }
                    ident.push(lexer.next().unwrap())
                }
                Ok(Some(Located::new(Token::ident(ident), pos)))
            }
            c => Err(Located::new(LexError::BadCharacter(c), pos)),
        }
    }
}
impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::BadCharacter(c) => write!(f, "bad character {c:?}"),
            LexError::ParseNumberError(err) => write!(f, "error while parsing number: {err}"),
            LexError::UnclosedString => write!(f, "unclosed string"),
        }
    }
}
