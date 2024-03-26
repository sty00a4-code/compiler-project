use std::{env, fs, process::exit};

use lexer::{lexer::Lexable, position::Located, tokens::Token};

pub mod lexer;
pub mod parser;
pub mod compiler;
pub mod interpreter;

fn main() {
    let mut args = env::args().skip(1);
    if let Some(path) = args.next() {
        let text = fs::read_to_string(&path).map_err(|err| {
            eprintln!("ERROR {path}: {err}");
            exit(1);
        }).unwrap();
        let tokens = Token::lex(&text).map_err(|Located { value: err, pos }| {
            eprintln!("ERROR {path}:{}:{}: {err}", pos.ln + 1, pos.col + 1);
            exit(1);
        }).unwrap();
        dbg!(tokens);
    } else {
        eprintln!("ERROR: no input file path provided");
        exit(1);
    }
}