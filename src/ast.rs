use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use crate::common::Span;

// perhaps the public should be a spanned bool?

#[derive(Clone, PartialEq, Debug)]
pub enum Ast {
    Program(Vec<AstNode>),
    Import {
        names: Vec<(SString, Option<SString>)>,
        module: SString,
        module_alias: Option<SString>,
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
        variadic: bool,
    },

    TypeName(String),

    // todo: better type choices
    Integer(u128),
    String(String),
    Boolean(bool),
    Float(u128, u128),

    VariableAccess(String),
    MemberAccess(AstNode, SString),
    Call(AstNode, Vec<AstNode>),

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

    pub fn span(&self) -> Span {
        self.0.span.clone()
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

pub fn print_tree(node: &AstNode) {
    print_tree_internal(node, 0);
}

#[inline]
fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

#[inline]
fn print_node(name: &str, depth: usize) {
    println!("{}<{}>", indent(depth), name);
}

#[inline]
fn print_node_with_value(name: &str, value: impl Display, depth: usize) {
    println!("{}<{} {}>", indent(depth), name, value);
}

#[inline]
fn print_category(name: &str, depth: usize) {
    println!("{}{}:", indent(depth), name);
}

#[inline]
fn print_indented(content: impl Display, depth: usize) {
    println!("{}{}", indent(depth), content);
}

#[inline]
fn print_labeled(label: &str, content: impl Debug, depth: usize) {
    print_indented(format!("{}: {:?}", label, content), depth);
}

#[inline]
fn print_tree_internal(node: &AstNode, depth: usize) {
    match *node.0.value.clone() {
        Ast::Program(nodes) => {
            println!("<Program>");

            for node in nodes {
                print_tree_internal(&node, depth + 1);
            }
        }
        Ast::Import {
            names,
            module,
            module_alias,
        } => {
            print_node("Import", depth);

            print_category("name", depth + 1);

            for name in names {
                print_indented(&*name.0, depth + 2);
                print_labeled("alias", name.1.map(|v| v.value), depth + 2);
            }

            print_labeled("module alias", module_alias.map(|v| v.value), depth + 1);
            print_labeled("from", module.value, depth + 1);
        }
        Ast::Static {
            public,
            name,
            type_,
            body,
        } => {
            print_node("Static", depth);
            print_labeled("public", public, depth + 1);
            print_labeled("name", name.value, depth + 1);
            print_category("type", depth + 1);
            print_tree_internal(&type_, depth + 2);
            print_category("body", depth + 1);
            print_tree_internal(&body, depth + 2);
        }
        Ast::Let {
            pattern,
            type_,
            body,
        } => {
            print_node("Let", depth);
            print_category("pattern", depth + 1);
            print_tree_internal(&pattern, depth + 2);

            print_category("type", depth + 1);
            if let Some(type_) = type_ {
                print_tree_internal(&type_, depth + 2);
            } else {
                print_indented("None", depth + 2);
            }

            print_tree_internal(&body, depth + 1);
        }
        Ast::Assignment { pattern, body } => {
            print_node("Assignment", depth);
            print_category("pattern", depth + 1);
            print_tree_internal(&pattern, depth + 2);

            print_tree_internal(&body, depth + 1);
        }
        Ast::Class { public, name, body } => {
            print_node_with_value("Class", name.value, depth);
            print_labeled("public", public, depth + 1);

            for definition in body {
                print_tree_internal(&definition, depth + 1);
            }
        }
        Ast::Function {
            public,
            name,
            args,
            return_type,
            body,
            variadic,
        } => {
            print_node_with_value("Function", name.value, depth);
            print_labeled("public", public, depth + 1);
            print_category("args", depth + 1);

            for arg in args {
                print_category("arg", depth + 2);
                print_labeled("name", arg.0, depth + 3);
                print_category("type", depth + 3);
                print_tree_internal(&arg.1, depth + 4);
            }

            print_category("type", depth + 1);
            if let Some(return_type) = return_type {
                print_tree_internal(&return_type, depth + 2);
            } else {
                print_indented("None", depth + 2);
            }

            print_labeled("variadic", variadic, depth + 1);

            print_tree_internal(&body, depth + 1);
        }
        Ast::TypeName(name) => {
            print_node_with_value("TypeName", name, depth);
        }
        Ast::Integer(value) => print_node_with_value("Integer", value, depth),
        Ast::String(value) => print_node_with_value("String", value, depth),
        Ast::Boolean(value) => print_node_with_value("Boolean", value, depth),
        Ast::Float(integral, fractional) => {
            print_node_with_value("Float", format!("{}.{}", integral, fractional), depth)
        }
        Ast::VariableAccess(name) => print_node_with_value("VariableAccess", name, depth),
        Ast::MemberAccess(object, name) => {
            print_node_with_value("MemberAccess", name.value, depth);
            print_tree_internal(&object, depth + 1);
        }
        Ast::Call(function, args) => {
            print_node("Call", depth);
            print_tree_internal(&function, depth + 1);
            print_category("args", depth + 1);
            for arg in args {
                print_tree_internal(&arg, depth + 1);
            }
        }
        Ast::Block {
            statements,
            return_value,
        } => {
            print_node("Block", depth);

            for statement in statements {
                print_tree_internal(&statement, depth + 1);
            }

            print_category("return", depth + 1);
            if let Some(value) = return_value {
                print_tree_internal(&value, depth + 2);
            } else {
                print_indented("None", depth + 2);
            }
        }
    }
}
