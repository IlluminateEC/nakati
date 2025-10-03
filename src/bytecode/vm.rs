use std::{
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::Arc,
    usize,
};

use crate::bytecode::{Code, ConstRef, Constant, Instruction, Module, Offset, REGISTERS, Register};

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

impl From<&Constant> for RegisterValue {
    fn from(value: &Constant) -> Self {
        match value {
            Constant::UnsignedInteger(value) => Self::UnsignedInteger(*value),
        }
    }
}

#[derive(Clone, Debug)]
// struct Registers([RegisterValue; REGISTERS]);
struct Registers([u64; REGISTERS]);

impl Registers {
    pub fn new() -> Self {
        // Self([RegisterValue::Empty; 64])
        Self([0; 64])
    }
}

impl Index<Register> for Registers {
    // type Output = RegisterValue;
    type Output = u64;

    fn index(&self, index: Register) -> &Self::Output {
        &self.0[unsafe { usize::try_from(index).unwrap_unchecked() }]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.0[unsafe { usize::try_from(index).unwrap_unchecked() }]
    }
}

impl Into<u64> for &Constant {
    fn into(self) -> u64 {
        match self {
            Constant::UnsignedInteger(number) => *number,
        }
    }
}

/// An individual frame in the call stack.
struct StackFrame<'a> {
    parent: Option<NonNull<Self>>,
    function: &'a Code,
    // Should be u24 eventually.
    ip: u32,
    registers: Registers,
    // return_value: Option<RegisterValue>,
    return_value: Option<u64>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Continue,
    // Exit(Option<RegisterValue>),
    Exit(Option<u64>),
    JumpTo(u32),
}

// TODO: switch from Arc to static vector
pub struct Interpreter {
    module: Arc<Module>,
    frame: Option<NonNull<StackFrame<'static>>>,
}

impl Interpreter {
    #[inline]
    fn register_to_usize(register: Register) -> usize {
        unsafe { usize::try_from(register).unwrap_unchecked() }
    }

    #[inline]
    fn constant_to_usize(constant: ConstRef) -> usize {
        unsafe { usize::try_from(constant).unwrap_unchecked() }
    }

    #[inline]
    fn offset_to_isize(offset: Offset) -> i32 {
        unsafe { i32::try_from(offset).unwrap_unchecked() }
    }

    #[inline]
    fn get_frame(&self) -> Option<&'static mut StackFrame<'static>> {
        unsafe { self.frame.map(|mut ptr| ptr.as_mut()) }
    }
}

impl Interpreter {
    pub fn new(module: Arc<Module>) -> Self {
        // assume it's the first block for now
        // TODO: eliminate this clone
        let main = &module.clone().code_table[0];

        Self {
            module,
            frame: Some(StackFrame::new(None, main)),
        }
    }

    // TODO: invalid offsets should return a VM error
    #[inline]
    fn apply_offset(&self, offset: Offset) -> u32 {
        let ip = self.get_frame().unwrap().ip;
        let offset = Self::offset_to_isize(offset);

        if offset.is_negative() {
            ip - offset.wrapping_abs() as u32
        } else {
            ip + offset as u32
        }
    }

    #[inline]
    fn run_instruction(&mut self, instruction: &Instruction) -> VMResult<State> {
        let frame = self
            .get_frame()
            .expect("Stack frame should exist when running instruction. Please.");

        match *instruction {
            Instruction::Return(return_value) => {
                let return_value = return_value.map(|register| frame.registers[register]);

                unsafe {
                    std::alloc::dealloc(
                        self.frame.unwrap().as_ptr() as *mut u8,
                        std::alloc::Layout::new::<StackFrame<'static>>(),
                    );
                }

                Ok(if let Some(parent) = frame.parent {
                    self.frame = Some(parent);
                    self.get_frame().unwrap().return_value = return_value;

                    State::Continue
                } else {
                    // Exiting the program.
                    State::Exit(return_value)
                })
            }
            Instruction::LoadConstant(register, constant) => {
                let constant_value = self
                    .module
                    .constant_table
                    .get(Self::constant_to_usize(constant))
                    .expect("constant should exist");

                frame.registers.0[Self::register_to_usize(register)] = constant_value.into();

                Ok(State::Continue)
            }

            Instruction::Jump(offset) => Ok(State::JumpTo(self.apply_offset(offset))),

            _ => todo!("{:?}", instruction),
        }
    }

    pub fn run(&mut self) -> VMResult<u8> {
        let start = std::time::Instant::now();
        let mut count: usize = 0;
        // let max = std::time::Duration::from_secs(10);

        while let Some(instruction) = self
            .get_frame()
            .map(|frame| frame.function.instructions.get(frame.ip as usize))
            .flatten()
        {
            count += 1;

            let state = self.run_instruction(instruction)?;

            // if duration > max {
            if count >= 10_000_000_000 {
                println!("Count: {}", count);
                let duration = std::time::Instant::now() - start;
                println!("Average: {}", duration.as_nanos() as f64 / count as f64);
                break;
            }

            match state {
                State::Continue => {
                    unsafe { self.get_frame().unwrap_unchecked() }.ip += 1;
                }
                State::JumpTo(ip) => {
                    unsafe { self.get_frame().unwrap_unchecked() }.ip = ip;
                }
                State::Exit(return_code) => {
                    return Ok(match return_code {
                        // Some(RegisterValue::UnsignedInteger(value)) => {
                        //     u8::try_from(value).unwrap_or(u8::MAX)
                        // }
                        Some(number) => u8::try_from(number).unwrap_or(u8::MAX),
                        // Some(_) => u8::MAX,
                        None => 0,
                    });
                }
            }
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bytecode::{Code, ConstRef, Constant, Instruction, Module, Offset, Register};

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
                instructions: vec![
                    Instruction::LoadConstant(Register::new(0), ConstRef::new(0)),
                    Instruction::Jump(Offset::new(-1)), // Instruction::Return(Some(Register::new(0))),
                ],
            }],
        }));

        let return_code = interpreter.run().unwrap();

        assert_eq!(return_code, 0);
        // assert_eq!(
        //     interpreter.get_frame().registers[Register::new(0)],
        //     RegisterValue::UnsignedInteger(73)
        // );
    }
}
