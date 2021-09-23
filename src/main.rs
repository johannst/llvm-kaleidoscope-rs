use llvm_kaleidoscope_rs::{
    codegen::Codegen,
    lexer::{Lexer, Token},
    llvm,
    parser::Parser,
    Either,
};

use std::io::Read;

fn main_loop<I>(mut parser: Parser<I>)
where
    I: Iterator<Item = char>,
{
    // Initialize LLVM module with its own context.
    // We will emit LLVM IR into this module.
    let module = llvm::Module::new();

    loop {
        match parser.cur_tok() {
            Token::Eof => break,
            Token::Char(';') => {
                // Ignore top-level semicolon.
                parser.get_next_token();
            }
            Token::Def => match parser.parse_definition() {
                Ok(func) => {
                    println!("Parse 'def'\n{:?}", func);
                    if let Ok(func) = Codegen::compile(&module, Either::B(&func)) {
                        func.dump();
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    parser.get_next_token();
                }
            },
            Token::Extern => match parser.parse_extern() {
                Ok(proto) => {
                    println!("Parse 'extern'\n{:?}", proto);
                    if let Ok(proto) = Codegen::compile(&module, Either::A(&proto)) {
                        proto.dump();
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    parser.get_next_token();
                }
            },
            _ => match parser.parse_top_level_expr() {
                Ok(func) => {
                    println!("Parse top-level expression\n{:?}", func);
                    if let Ok(func) = Codegen::compile(&module, Either::B(&func)) {
                        func.dump();
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    parser.get_next_token();
                }
            },
        };
    }

    // Dump all the emitted LLVM IR to stdout.
    module.dump();
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

    main_loop(parser);

    // De-allocate managed static LLVM data.
    llvm::shutdown();
}
