#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // word kinds
    Ident(String),
    Number(f64),
    String(String),

    // symbols
    Equal, // =
    Comma, // ,
    Dot, // .
    ParanLeft, // (
    ParanRight, // )
    BracketLeft, // [
    BracketRight, // ]
    BraceLeft, // {
    BraceRight, // }
    Plus, // +
    Minus, // -
    Star, // *
    Slash, // /
    Percent, // %
    Exponent, // ^
    EqualEqual, // ==
    ExclamationEqual, // !=
    Less, // <
    Greater, // >
    LessEqual, // <=
    GreaterEqual, // >=
    Ampersand, // &
    Pipe, // |
    Exclamation, // !
    
    // key words
    Let,
    Def,
    If,
    Else,
    While,
    For,
}