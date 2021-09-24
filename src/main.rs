use llvm_kaleidoscope_rs::{
    codegen::Codegen,
    lexer::{Lexer, Token},
    llvm,
    parser::{Parser, PrototypeAST},
    Either,
};

use std::collections::HashMap;
use std::io::Read;

fn main_loop<I>(mut parser: Parser<I>)
where
    I: Iterator<Item = char>,
{
    // Initialize LLVM module with its own context.
    // We will emit LLVM IR into this module.
    let mut module = llvm::Module::new();

    // Create a new JIT, based on the LLVM LLJIT.
    let jit = llvm::LLJit::new();

    // Enable lookup of dynamic symbols in the current process from the JIT.
    jit.enable_process_symbols();

    // Keep track of prototype names to their respective ASTs.
    //
    // This is useful since we jit every function definition into its own LLVM module.
    // To allow calling functions defined in previous LLVM modules we keep track of their
    // prototypes and generate IR for their declarations when they are called from another module.
    let mut fn_protos: HashMap<String, PrototypeAST> = HashMap::new();

    // When adding an IR module to the JIT, it will hand out a ResourceTracker. When the
    // ResourceTracker is dropped, the code generated from the corresponding module will be removed
    // from the JIT.
    //
    // For each function we want to keep the code generated for the last definition, hence we need
    // to keep their ResourceTracker alive.
    let mut fn_jit_rt: HashMap<String, llvm::ResourceTracker> = HashMap::new();

    loop {
        match parser.cur_tok() {
            Token::Eof => break,
            Token::Char(';') => {
                // Ignore top-level semicolon.
                parser.get_next_token();
            }
            Token::Def => match parser.parse_definition() {
                Ok(func) => {
                    println!("Parse 'def'");
                    let func_name = &func.0 .0;

                    // If we already jitted that function, remove the last definition from the JIT
                    // by dropping the corresponding ResourceTracker.
                    fn_jit_rt.remove(func_name);

                    if let Ok(func_ir) = Codegen::compile(&module, &mut fn_protos, Either::B(&func))
                    {
                        func_ir.dump();

                        // Add module to the JIT.
                        let rt = jit.add_module(module);

                        // Keep track of the ResourceTracker to keep the module code in the JIT.
                        fn_jit_rt.insert(func_name.to_string(), rt);

                        // Initialize a new module.
                        module = llvm::Module::new();
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    parser.get_next_token();
                }
            },
            Token::Extern => match parser.parse_extern() {
                Ok(proto) => {
                    println!("Parse 'extern'");
                    if let Ok(proto_ir) =
                        Codegen::compile(&module, &mut fn_protos, Either::A(&proto))
                    {
                        proto_ir.dump();

                        // Keep track of external function declaration.
                        fn_protos.insert(proto.0.clone(), proto);
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    parser.get_next_token();
                }
            },
            _ => match parser.parse_top_level_expr() {
                Ok(func) => {
                    println!("Parse top-level expression");
                    if let Ok(func) = Codegen::compile(&module, &mut fn_protos, Either::B(&func)) {
                        func.dump();

                        // Add module to the JIT. Code will be removed when `_rt` is dropped.
                        let _rt = jit.add_module(module);

                        // Initialize a new module.
                        module = llvm::Module::new();

                        // Call the top level expression.
                        let fp = jit.find_symbol::<unsafe extern "C" fn() -> f64>("__anon_expr");
                        unsafe {
                            println!("Evaluated to {}", fp());
                        }
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

    // Initialize native target for jitting.
    llvm::initialize_native_taget();

    main_loop(parser);

    // De-allocate managed static LLVM data.
    llvm::shutdown();
}
