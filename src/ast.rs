use std::ops::Deref;

use crate::common::Span;

// perhaps the public should be a spanned bool?

#[derive(Clone, PartialEq, Debug)]
pub enum Ast {
    Program(Vec<AstNode>),
    Import {
        names: Vec<(SString, Option<SString>)>,
        module: SString,
    },
    Static {
        public: bool,
        name: SString,
        type_: AstNode,
        body: AstNode,
    },

    // TODO
    Let {
        name: SString,
        type_: Option<AstNode>,
        body: AstNode,
    },
    // TODO
    Assignment {
        name: SString,
        body: AstNode,
    },

    Class {
        public: bool,
        name: SString,
        // TODO: inheritance, implementations
        body: Vec<AstNode>,
    },
    Function {
        public: bool,
        name: SString,
        args: Vec<(String, AstNode)>,
        return_type: Option<AstNode>,
        body: AstNode,
    },

    TypeName(String),

    // todo: better type choices
    Integer(u128),
    String(String),
    Boolean(bool),
    Float(u128, u128),

    VariableAccess(String),

    Block {
        statements: Vec<AstNode>,
        return_value: Option<AstNode>,
    },
}

// pub type SString = Spanned<String>;
// pub type AstNode = BoxSpanned<Ast>;
pub type SString = String;
pub type AstNode = Box<Ast>;

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoxSpanned<T>(Spanned<Box<T>>);

impl<T> BoxSpanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self(Spanned::new(Box::new(value), span))
    }
}

impl<T> Deref for BoxSpanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}
