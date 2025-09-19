pub mod compiler;
pub mod interpreter;
pub mod loader;

// TODO: figure out what instructions I need
pub enum Instruction {
    Return,
}

pub struct Code {
    pub instructions: Vec<Instruction>,
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
    pub code_table: Vec<Declaration>,
}
