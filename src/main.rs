use std::env::args;
use std::process::exit;

use nakati::interpreter::Interpreter;
use nakati::lexer::Lexer;
use nakati::module::ModuleLabel;
use nakati::parser::Parser;
use nakati::source::Source;

fn main() {
    // let filename = args().nth(1).unwrap_or("tests/1.nak".to_string());

    // let source_id = Source::from_path(filename).register();
    // let lexer = Lexer::new(source_id);
    // let mut parser = Parser::new(lexer);

    // let program = parser.parse();

    // if program.is_err() {
    //     println!("Parse error: {:?}", program.err().unwrap());
    //     exit(1);
    // }

    // Interpreter::interpret(program.unwrap(), ModuleLabel::from_special("main")).unwrap();

    let mut interpreter =
        nakati::bytecode::vm::Interpreter::new(std::sync::Arc::new(nakati::bytecode::Module {
            hash: [0; 16],
            name: "idk".to_string(),
            source_stats: (0,),
            signature: None,
            declaration_table: vec![],
            constant_table: vec![nakati::bytecode::Constant::UnsignedInteger(73)],
            code_table: vec![nakati::bytecode::Code {
                instructions: vec![
                    nakati::bytecode::Instruction::LoadConstant(
                        nakati::bytecode::Register::new(0),
                        nakati::bytecode::ConstRef::new(0),
                    ),
                    nakati::bytecode::Instruction::Jump(nakati::bytecode::Offset::new(-1)), // Instruction::Return(Some(Register::new(0))),
                ],
            }],
        }));

    let return_code = interpreter.run().unwrap();
}
