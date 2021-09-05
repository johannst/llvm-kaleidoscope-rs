mod lexer;
mod parser;

use lexer::{Lexer, Token};
use parser::Parser;
use std::io::Read;

fn handle_definition<I>(p: &mut Parser<I>)
where
    I: Iterator<Item = char>,
{
    match p.parse_definition() {
        Ok(expr) => println!("Parse 'def'\n{:?}", expr),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn handle_extern<I>(p: &mut Parser<I>)
where
    I: Iterator<Item = char>,
{
    match p.parse_extern() {
        Ok(expr) => println!("Parse 'extern'\n{:?}", expr),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn handle_top_level_expression<I>(p: &mut Parser<I>)
where
    I: Iterator<Item = char>,
{
    match p.parse_top_level_expr() {
        Ok(expr) => println!("Parse top-level expression\n{:?}", expr),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn main() {
    println!("Parse stdin.");
    println!("ENTER to parse current input.");
    println!("C-d   to exit.");

    let lexer = Lexer::new(std::io::stdin().bytes().filter_map(|v| {
        let v = v.ok()?;
        Some(v.into())
    }));

    let mut parser = Parser::new(lexer);

    // Throw first coin and initialize cur_tok.
    parser.get_next_token();

    loop {
        match *parser.cur_tok() {
            Token::Eof => break,
            Token::Char(';') => {
                // Ignore top-level semicolon.
                parser.get_next_token()
            }
            Token::Def => handle_definition(&mut parser),
            Token::Extern => handle_extern(&mut parser),
            _ => handle_top_level_expression(&mut parser),
        }
    }
}
