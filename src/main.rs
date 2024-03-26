use crate::{lexer::lex, parser::parse};
use lexer::position::Located;
use std::{env, fs, process::exit};

pub mod compiler;
pub mod interpreter;
pub mod lexer;
pub mod parser;

fn main() {
    let mut args = env::args().skip(1);
    if let Some(path) = args.next() {
        let text = fs::read_to_string(&path)
            .map_err(|err| {
                eprintln!("ERROR {path}: {err}");
                exit(1);
            })
            .unwrap();
        let tokens = lex(&text)
            .map_err(|Located { value: err, pos }| {
                eprintln!("ERROR {path}:{}:{}: {err}", pos.ln + 1, pos.col + 1);
                exit(1);
            })
            .unwrap();
        let chunk = parse(tokens)
            .map_err(|Located { value: err, pos }| {
                eprintln!("ERROR {path}:{}:{}: {err}", pos.ln + 1, pos.col + 1);
                exit(1);
            })
            .unwrap();
        dbg!(chunk);
    } else {
        eprintln!("ERROR: no input file path provided");
        exit(1);
    }
}
