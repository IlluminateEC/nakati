use std::process::exit;

fn main() {
    let source = nakati::common::Source::from_path("tests/1.nak");
    let lexer = nakati::lexer::Lexer::new(source.clone());
    let mut parser = nakati::parser::Parser::new(lexer);

    let program = parser.parse();

    if program.is_err() {
        println!("Parse error: {:?}", program.err().unwrap());
        exit(1);
    }

    println!("AST: {:#?}", program.unwrap());
}
