use std::env::args;
use std::process::exit;

use nakati::interpreter::Interpreter;
use nakati::lexer::Lexer;
use nakati::module::ModuleLabel;
use nakati::parser::Parser;
use nakati::source::Source;

fn main() {
    let filename = args().nth(1).unwrap_or("tests/1.nak".to_string());

    let source_id = Source::from_path(filename).register();
    let lexer = Lexer::new(source_id);
    let mut parser = Parser::new(lexer);

    let program = parser.parse();

    if program.is_err() {
        println!("Parse error: {:?}", program.err().unwrap());
        exit(1);
    }

    Interpreter::interpret(program.unwrap(), ModuleLabel::from_special("main")).unwrap();
}
