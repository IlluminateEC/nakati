use std::{ptr::NonNull, sync::Arc, usize};

use crate::bytecode::{Code, Instruction, Module, REGISTERS};

#[derive(Debug, Clone, PartialEq)]
pub enum VMError {
    NotImplemented,
}

pub type VMResult<T> = Result<T, VMError>;

// TODO: threads

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegisterValue {
    UnsignedInteger(u64),
    SignedInteger(i64),
    Empty,
}

#[derive(Clone, Debug)]
struct Registers([RegisterValue; REGISTERS]);

impl Registers {
    pub fn new() -> Self {
        Self([RegisterValue::Empty; 64])
    }
}

/// An individual frame in the call stack.
struct StackFrame<'a> {
    parent: Option<NonNull<Self>>,
    function: &'a Code,
    // Should be u24 eventually.
    ip: u32,
    registers: Registers,
    return_value: Option<RegisterValue>,
}

impl<'a> StackFrame<'a> {
    pub fn new(parent: Option<NonNull<Self>>, function: &Code) -> NonNull<Self> {
        use std::alloc::{Layout, alloc};

        unsafe {
            // TODO: replace with Allocator trait and Global.
            let func = std::mem::transmute::<&Code, &'static Code>(function);
            let ptr = alloc(Layout::new::<Self>()) as *mut StackFrame<'a>;

            *ptr = StackFrame {
                parent,
                function: func,
                ip: 0,
                registers: Registers::new(),
                return_value: None,
            };

            NonNull::new_unchecked(ptr)
        }
    }
}

// TODO: switch from Arc to static vector
pub struct Interpreter {
    module: Arc<Module>,
    frame: NonNull<StackFrame<'static>>,
}

impl Interpreter {
    pub fn new(module: Arc<Module>) -> Self {
        // assume it's the first block for now
        // TODO: eliminate this clone
        let main = &module.clone().code_table[0];

        Self {
            module,
            frame: StackFrame::new(None, main),
        }
    }

    fn run_instruction(&mut self, instruction: &Instruction) -> VMResult<()> {
        let frame = unsafe { self.frame.as_mut() };

        match *instruction {
            Instruction::Return(return_value) => {
                let return_value = return_value.map(|register| {
                    frame.registers.0[unsafe { usize::try_from(register).unwrap_unchecked() }]
                });

                // TODO: switch to VMError
                let parent = frame.parent.expect("function has caller when returning");

                let current_frame = self.frame;
                self.frame = parent;
                let frame = unsafe { self.frame.as_mut() };
                frame.return_value = return_value;

                unsafe {
                    std::alloc::dealloc(
                        current_frame.as_ptr() as *mut u8,
                        std::alloc::Layout::new::<StackFrame<'static>>(),
                    );
                }

                Ok(())
            }
            _ => todo!("{:?}", instruction),
        }
    }

    pub fn run(&mut self) -> VMResult<()> {
        while let Some(instruction) = unsafe { self.frame.as_ref() }
            .function
            .instructions
            .get(unsafe { self.frame.as_ref() }.ip as usize)
        {
            unsafe { self.frame.as_mut() }.ip += 1;

            self.run_instruction(instruction)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bytecode::{Code, ConstRef, Constant, Instruction, Module, Register};

    use super::Interpreter;

    #[test]
    fn test() {
        let mut interpreter = Interpreter::new(Arc::new(Module {
            hash: [0; 16],
            name: "idk".to_string(),
            source_stats: (0,),
            signature: None,
            declaration_table: vec![],
            constant_table: vec![Constant::UnsignedInteger(73)],
            code_table: vec![Code {
                instructions: vec![Instruction::LoadConstant(
                    Register::new(0),
                    ConstRef::new(0),
                )],
            }],
        }));

        interpreter.run().unwrap();

        println!("{:?}", unsafe { &interpreter.frame.as_ref().registers });
        panic!();
    }
}
