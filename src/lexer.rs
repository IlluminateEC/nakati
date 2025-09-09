use std::rc::Rc;

use crate::common::{OpRes, OptionalResult, Source, Span};
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum LexError {
    UnexpectedCharacter(&'static String),
}

pub struct Lexer {
    span: Span,
}

impl Lexer {
    pub fn new(source: Rc<Source>) -> Self {
        Self {
            span: Span::new(source),
        }
    }

    fn is_emoji(g: &str) -> bool {
        emojis::get(g).is_some()
    }

    fn is_digit(g: &str) -> bool {
        g.chars().nth(0).unwrap().is_ascii_digit()
    }

    fn is_whitespace(g: &str) -> bool {
        g.chars().nth(0).unwrap().is_whitespace()
    }

    fn is_alphabetic(g: &str) -> bool {
        g.chars().nth(0).unwrap().is_alphabetic() || Self::is_emoji(g) || g == "_"
    }

    fn is_alphanumeric(g: &str) -> bool {
        g.chars().nth(0).unwrap().is_alphanumeric() || Self::is_emoji(g) || g == "_"
    }

    fn at(&self, target: &str) -> bool {
        self.span.has(target)
    }

    fn at_any(&self, targets: &[&str]) -> bool {
        if let Some(cur) = self.span.cur() {
            for target in targets {
                if target == cur {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    fn discard_ignorables(&mut self) {
        let mut old_span = self.span.clone();
        let mut changed = true;

        while changed {
            self.span.skip_while(Self::is_whitespace);

            if self.span.has("//") {
                self.span.bump("//");
                self.span.skip_while(|g| !g.contains('\n'));
            }

            changed = self.span != old_span;
            old_span = self.span.clone();
        }
    }

    pub fn next_token(&mut self) -> OptionalResult<Token, LexError> {
        self.discard_ignorables();

        let g = self.span.cur();

        if g.is_none() {
            return OpRes::None;
        }

        let g = g.unwrap();

        OpRes::Ok(if Self::is_alphabetic(g) {
            self.span.bump_while(Self::is_alphanumeric);

            let content = &self.span.content();

            if content == &"true" || content == &"false" {
                Token::new(&self.span, TokenKind::Boolean)
            } else {
                Token::new(&self.span, TokenKind::Identifier)
            }
        } else if Self::is_digit(g) {
            self.span.bump_while(Self::is_digit);

            if self.at(".") {
                self.span.bump(self.span.cur().unwrap());
                self.span.bump_while(Self::is_digit);

                Token::new(&self.span, TokenKind::Float)
            } else {
                Token::new(&self.span, TokenKind::Integer)
            }
        } else if self.at("\"") {
            // TODO: handle escaping
            self.span.bump_cur();
            self.span.release(); // skip beginning quote
            self.span.bump_while(|g| g != "\"");

            let span = self.span.clone();
            self.span.bump_cur(); // skip end quote

            Token::new(&span, TokenKind::String)
        } else if self.at("=>") {
            self.span.bump("=>");
            Token::new(&self.span, TokenKind::ThickArrow)
        } else if self.at("=") {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Equals)
        } else if self.at(";") {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Semicolon)
        } else if self.at(".") {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Dot)
        } else if self.at(",") {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Comma)
        } else if self.at(":") {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Colon)
        } else if self.at_any(&["+", "-", "/", "*"]) {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Operator)
        } else if self.at_any(&["(", "{", "["]) {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Open)
        } else if self.at_any(&[")", "}", "]"]) {
            self.span.bump_cur();
            Token::new(&self.span, TokenKind::Close)
        } else {
            OpRes::Err(LexError::UnexpectedCharacter(self.span.cur().unwrap()))?
        })
    }
}
