use crate::common::Span;
use std::fmt::Debug;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TokenKind {
    Identifier,
    Integer,
    Float,
    String,
    Boolean,

    Semicolon,
    Equals,
    Operator,
    Dot,
    Comma,

    Open,
    Close,

    Colon,
    ThickArrow,
}

#[derive(Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    pub fn new(span: &Span, kind: TokenKind) -> Self {
        Self {
            span: span.clone(),
            kind,
        }
    }

    pub fn new_content(content: impl ToString, kind: TokenKind) -> Self {
        Self {
            span: Span::new_string(content),
            kind,
        }
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    pub fn has(&self, content: impl AsRef<str>) -> bool {
        self.span.content() == content.as_ref()
    }

    pub fn content(&self) -> String {
        self.span.content().to_string()
    }

    pub fn is_and_has_content(&self, kind: TokenKind, content: Option<&str>) -> bool {
        if self.kind == kind {
            if let Some(content) = content {
                if self.span.content() == content {
                    return true;
                }

                return false;
            } else {
                return true;
            }
        }

        false
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.span.content() == other.span.content()
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<{:?} {:?}>", self.kind, self.span.content()))
    }
}
