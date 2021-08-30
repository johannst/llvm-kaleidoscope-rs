mod lexer;

use lexer::Lexer;
use std::io::Read;

fn main() {
    println!("Lex stdin.");
    println!("ENTER to lex current input.");
    println!("C-c   to exit.");
    let mut lex = Lexer::new(std::io::stdin().bytes().filter_map(|v| {
        let v = v.ok()?;
        Some(v.into())
    }));

    loop {
        println!("{:?}", lex.gettok());
    }
}
