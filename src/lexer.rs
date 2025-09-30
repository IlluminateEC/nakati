use crate::common::{OpRes, OptionalResult, Span, Window};
use crate::source::SourceId;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum LexError {
    UnexpectedCharacter(&'static String),
}

pub struct Lexer {
    window: Window,
}

impl Lexer {
    pub fn new(source: SourceId) -> Self {
        Self {
            window: Window::new(source),
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
        self.window.has(target)
    }

    fn at_any(&self, targets: &[&str]) -> bool {
        if let Some(cur) = self.window.cur() {
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
        let mut old_span = self.window.make();
        let mut changed = true;

        while changed {
            self.window.skip_while(Self::is_whitespace);

            if self.window.has("//") {
                self.window.bump("//");
                self.window.skip_while(|g| !g.contains('\n'));
            }

            changed = self.window.make() != old_span;
            old_span = self.window.make();
        }
    }

    pub fn next_token(&mut self) -> OptionalResult<Token, LexError> {
        self.discard_ignorables();

        let g = self.window.cur();

        if g.is_none() {
            return OpRes::None;
        }

        let g = g.unwrap();

        OpRes::Ok(if Self::is_alphabetic(g) {
            self.window.bump_while(Self::is_alphanumeric);

            let content = &self.window.content();

            if content == &"true" || content == &"false" {
                Token::new(&self.window.make(), TokenKind::Boolean)
            } else {
                Token::new(&self.window.make(), TokenKind::Identifier)
            }
        } else if Self::is_digit(g) {
            self.window.bump_while(Self::is_digit);

            if self.at(".") {
                self.window.bump(self.window.cur().unwrap());
                self.window.bump_while(Self::is_digit);

                Token::new(&self.window.make(), TokenKind::Float)
            } else {
                Token::new(&self.window.make(), TokenKind::Integer)
            }
        } else if self.at("\"") {
            // TODO: handle escaping
            self.window.bump_cur();
            self.window.release(); // skip beginning quote
            self.window.bump_while(|g| g != "\"");

            let span = self.window.make();
            self.window.bump_cur(); // skip end quote

            Token::new(&span, TokenKind::String)
        } else if self.at("=>") {
            self.window.bump("=>");
            Token::new(&self.window.make(), TokenKind::ThickArrow)
        } else if self.at("=") {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Equals)
        } else if self.at(";") {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Semicolon)
        } else if self.at("...") {
            self.window.bump("...");
            Token::new(&self.window.make(), TokenKind::Ellipsis)
        } else if self.at(".") {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Dot)
        } else if self.at(",") {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Comma)
        } else if self.at(":") {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Colon)
        } else if self.at("@") {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Strudel)
        } else if self.at_any(&["+", "-", "/", "*"]) {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Operator)
        } else if self.at_any(&["(", "{", "["]) {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Open)
        } else if self.at_any(&[")", "}", "]"]) {
            self.window.bump_cur();
            Token::new(&self.window.make(), TokenKind::Close)
        } else {
            OpRes::Err(LexError::UnexpectedCharacter(self.window.cur().unwrap()))?
        })
    }

    pub fn get_span(&self) -> Span {
        self.window.make()
    }
}
