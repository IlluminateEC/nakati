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
        pattern: AstNode,
        type_: Option<AstNode>,
        body: AstNode,
    },
    // TODO
    Assignment {
        pattern: AstNode,
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
    MemberAccess(AstNode, SString),

    Block {
        statements: Vec<AstNode>,
        return_value: Option<AstNode>,
    },
}

pub type SString = Spanned<String>;
pub type AstNode = BoxSpanned<Ast>;

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

#[derive(Clone, PartialEq)]
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

impl<T: std::fmt::Debug> std::fmt::Debug for BoxSpanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
