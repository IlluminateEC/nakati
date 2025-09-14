use std::env::args;
use std::process::exit;

use nakati::ast::print_tree;

fn main() {
    let filename = args().nth(1).unwrap_or("tests/1.nak".to_string());

    let source = nakati::common::Source::from_path(filename);
    let lexer = nakati::lexer::Lexer::new(source.clone());
    let mut parser = nakati::parser::Parser::new(lexer);

    let program = parser.parse();

    if program.is_err() {
        println!("Parse error: {:?}", program.err().unwrap());
        exit(1);
    }

    print_tree(&program.unwrap());
}
