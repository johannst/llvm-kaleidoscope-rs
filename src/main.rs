use llvm_kaleidoscope_rs::{
    codegen::Codegen,
    lexer::{Lexer, Token},
    llvm,
    parser::Parser,
    Either,
};

use std::io::Read;

fn handle_definition<I>(p: &mut Parser<I>, module: &llvm::Module)
where
    I: Iterator<Item = char>,
{
    match p.parse_definition() {
        Ok(func) => {
            println!("Parse 'def'\n{:?}", func);
            if let Ok(func) = Codegen::compile(module, Either::B(&func)) {
                func.dump();
            }
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn handle_extern<I>(p: &mut Parser<I>, module: &llvm::Module)
where
    I: Iterator<Item = char>,
{
    match p.parse_extern() {
        Ok(proto) => {
            println!("Parse 'extern'\n{:?}", proto);
            if let Ok(proto) = Codegen::compile(module, Either::A(&proto)) {
                proto.dump();
            }
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
            p.get_next_token();
        }
    }
}

fn handle_top_level_expression<I>(p: &mut Parser<I>, module: &llvm::Module)
where
    I: Iterator<Item = char>,
{
    match p.parse_top_level_expr() {
        Ok(func) => {
            println!("Parse top-level expression\n{:?}", func);
            if let Ok(func) = Codegen::compile(module, Either::B(&func)) {
                func.dump();
            }
        }
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

    // Create lexer over stdin.
    let lexer = Lexer::new(std::io::stdin().bytes().filter_map(|v| {
        let v = v.ok()?;
        Some(v.into())
    }));

    // Create parser for kaleidoscope.
    let mut parser = Parser::new(lexer);

    // Throw first coin and initialize cur_tok.
    parser.get_next_token();

    // Initialize LLVM module with its own context.
    // We will emit LLVM IR into this module.
    let module = llvm::Module::new();

    loop {
        match *parser.cur_tok() {
            Token::Eof => break,
            Token::Char(';') => {
                // Ignore top-level semicolon.
                parser.get_next_token()
            }
            Token::Def => handle_definition(&mut parser, &module),
            Token::Extern => handle_extern(&mut parser, &module),
            _ => handle_top_level_expression(&mut parser, &module),
        }
    }

    // Dump all the emitted LLVM IR to stdout.
    module.dump();
}
