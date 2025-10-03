pub mod compiler;
pub mod loader;
pub mod vm;

pub const REGISTERS: usize = 64;
pub type Register = ux::u6;
pub type ConstRef = ux::u24;
pub type Offset = ux::i24;

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Size {
    Byte,
    Double,
    Quad,
    Octal,
    // Hex?
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    // Function calls
    Call {
        function: ConstRef,
        start_register: Register,
        args: ux::u6,
    },
    CallThis {
        function: ConstRef,
        start_register: Register,
        args: ux::u6,
        this: Register,
    },
    CallClosure {
        function: Register,
        args_start: Register,
        args: ux::u6,
    },
    StoreCallResult(Register),

    // Register operations
    /// Loads a constant value into a register.
    LoadConstant(Register, ConstRef),
    /// Copies a value from one register to another.
    CopyValueRegReg(Register, Register),
    /// Copies the value from a pointer to a register.
    CopyValuePtrReg(Register, Register, Size),
    /// Copies a value from a register to a pointer.
    CopyValueRegPtr(Register, Register, Size),
    /// Copies a value from a pointer to a poiner.
    CopyValuePtrPtr(Register, Register, Size),
    /// Copies an area of memory from a pointer to a pointer with a length.
    CopyMemory(Register, Register, Register),

    // Control flow
    /// Unconditionally jumps a certain amount of instructions forward or backwards in the current function.
    Jump(Offset),
    /// Jumps a certain amount of instructions forward or backwards in the current function if the register contains true.
    JumpIf(Register, Offset),
    /// Returns from the current function with the value from the register, or void.
    Return(Option<Register>),

    // Memory
    // TODO: figure out how to find the size of a type (pointer sizes can vary, interpreter data types might be different from compiler data types.)
    // It may be better for Allocate to take a Type, and to have a separate AllocateArray(Type, Length)
    // TODO: add output registers
    /// Allocate stack memory of a constant size
    AllocateConstant(usize),
    /// Allocate heap memory of a constant size
    AllocateGCConstant(usize),
    /// Allocate stack memory
    Allocate(Register),
    /// Allocate heap memory
    AllocateGC(Register),
    // Split into Dealloc and DeallocGC?
    /// Deallocate some stack memory or force deallocation of GC'd memory.
    Dealloc(Register),

    // Effects
    // TODO: add output registers
    /// Perform an effect. Creates a resumable copy of the current VM state.
    /// TODO: figure out how to implement in compiler
    PerformEffect(ConstRef),
    /// Makes a copy of the current VM state.
    /// TODO: figure out how to implement in compiler
    /// Handle both closures and functions.
    HandleEffect(ConstRef, Register),

    // Threads
    /// Spawns a new thread. Returns a thread handle.
    Spawn(),
    /// Sends a message to another thread.
    Send(),
    /// Waits for a message from another thread. Optionally can skip waiting.
    Receive(),
    /// Exits the current thread. Optionally takes a return value. When the main thread is exited, this is the program's exit code.
    Exit(Register),
    Kill(),
    Pause(),
    Resume(),

    // Arithmetic
    // Not finalized. Will probably be replaced.
    AddSI(Register, Register, Register),
    AddUI(Register, Register, Register),
}

pub struct Code {
    pub instructions: Vec<Instruction>,
}

pub enum Constant {
    UnsignedInteger(u64),
}

pub enum DeclarationKind {
    Class,
    Function,
    Static,
    TypeAlias,
    Test,
}

/// An entry for a declaration in the module.
pub struct Declaration {
    pub kind: DeclarationKind,
    pub name: String,
}

/// Modules can be signed by the organization or developer of the module.
///
/// Does this have a tangible security benefit? I have no idea.
pub struct Signature {
    /// A signer string
    pub signer: String,
    pub signature: [u8; 64],
}

pub struct Module {
    /// A hash of the source file the module was compiled from. Used by build systems.
    pub hash: [u8; 16],
    /// The canonical path for the module
    pub name: String,
    /// The size and modification timestamp of the source file  
    pub source_stats: (u64,),
    pub signature: Option<Signature>,

    pub declaration_table: Vec<Declaration>,
    pub constant_table: Vec<Constant>,
    pub code_table: Vec<Code>,
}
