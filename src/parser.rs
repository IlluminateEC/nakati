use std::fmt::Debug;

use crate::{
    common::{Ast, OptionalResult, Token, TokenKind},
    lexer::{LexError, Lexer},
};

#[derive(Clone)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedToken(Option<Token>),
    ExpectedTokenGot(Vec<(TokenKind, Option<String>)>, Option<Token>),
    Todo(String),
}

impl ParseError {
    fn format_list_according_to_english_rules(mut items: Vec<impl ToString>) -> String {
        if items.len() == 0 {
            panic!("cannot format an empty vector");
        }

        if items.len() == 1 {
            return items[0].to_string();
        }

        if items.len() == 2 {
            return items[0].to_string() + " or " + &items[1].to_string();
        }

        let last = items.pop().unwrap();
        let mut buf = "".to_string();

        for name in items {
            buf += &(name.to_string() + ", ");
        }

        format!("{}or {}", buf, last.to_string())
    }

    fn format_token_pair(pair: &(TokenKind, Option<String>)) -> String {
        if pair.1.is_some() {
            format!("{:?}", pair.1.as_ref().unwrap())
        } else {
            format!("{:?}", pair.0)
        }
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LexError(e) => f.debug_tuple("LexError").field(e).finish(),
            Self::UnexpectedToken(tok) => f.debug_tuple("UnexpectedToken").field(tok).finish(),
            Self::ExpectedTokenGot(expected, got) => f.write_fmt(format_args!(
                "expected {} but got {}",
                Self::format_list_according_to_english_rules(
                    expected
                        .iter()
                        .map(|p| Self::format_token_pair(p))
                        .collect()
                ),
                got.as_ref()
                    .map(|v| format!("{:?} {:?}", v.kind, v.span.content()))
                    .unwrap_or("end of file".to_string())
            )),
            Self::Todo(message) => f.write_fmt(format_args!("TODO: {}", message)),
        }
    }
}

impl From<LexError> for ParseError {
    fn from(value: LexError) -> Self {
        Self::LexError(value)
    }
}

pub struct Parser {
    lexer: Lexer,
    current: OptionalResult<Token, LexError>,
    next: OptionalResult<Token, LexError>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current = lexer.next();
        let next = lexer.next();

        Self {
            lexer,
            current,
            next,
        }
    }

    pub fn parse(&mut self) -> Result<Ast, ParseError> {
        self.program()
    }
}

impl Parser {
    fn nom(&mut self) {
        std::mem::swap(&mut self.current, &mut self.next);

        self.next = self.lexer.next();
    }

    fn is(
        &mut self,
        kind: TokenKind,
        content: Option<&dyn AsRef<str>>,
    ) -> Result<bool, ParseError> {
        if self.current.is_none() {
            return Ok(false);
        }

        if self.current.is_error() {
            return Err(self.current.clone().as_result().err().unwrap().into());
        }

        let current = self.current.clone().as_option().unwrap();

        if current.kind == kind {
            if let Some(content) = content {
                if current.span.content() == content.as_ref() {
                    return Ok(true);
                }

                return Ok(false);
            } else {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn accept(
        &mut self,
        kind: TokenKind,
        content: Option<&dyn AsRef<str>>,
    ) -> Result<bool, ParseError> {
        if self.current.is_none() {
            return Ok(false);
        }

        if self.current.is_error() {
            return Err(self.current.clone().as_result().err().unwrap().into());
        }

        let current = self.current.clone().as_option().unwrap();

        if current.kind == kind {
            if let Some(content) = content {
                if current.span.content() == content.as_ref() {
                    self.nom();
                    return Ok(true);
                }

                return Ok(false);
            } else {
                self.nom();
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn expect(
        &mut self,
        kind: TokenKind,
        content: Option<&dyn AsRef<str>>,
    ) -> Result<Token, ParseError> {
        let current = self.current.clone();

        if self.accept(kind.clone(), content)? {
            return Ok(current.unwrap());
        }

        Err(ParseError::ExpectedTokenGot(
            vec![(kind, content.map(|s| s.as_ref().to_string()))],
            current.clone().as_option(),
        ))
    }

    fn import_name(&mut self) -> Result<(String, Option<String>), ParseError> {
        let name = self
            .expect(TokenKind::Identifier, None)?
            .span
            .content()
            .to_string();
        let mut alias = None;

        if self.accept(TokenKind::Identifier, Some(&"as"))? {
            alias = Some(
                self.expect(TokenKind::Identifier, None)?
                    .span
                    .content()
                    .to_string(),
            );
        }

        Ok((name, alias))
    }

    fn import_statement(&mut self) -> Result<Ast, ParseError> {
        let mut names = vec![];

        names.push(self.import_name()?);

        while self.accept(TokenKind::Comma, None)? {
            names.push(self.import_name()?);
        }

        self.expect(TokenKind::Identifier, Some(&"from"))?;

        let module = self.expect(TokenKind::String, None)?;

        self.expect(TokenKind::Semicolon, None)?;

        Ok(Ast::Import {
            names,
            module: module.span.content().to_string(),
        })
    }

    fn type_(&mut self) -> Result<Ast, ParseError> {
        // TODO: actual parsing of types
        // generics

        let name = self.expect(TokenKind::Identifier, None)?;

        Ok(Ast::TypeName(name.span.content().to_string()))
    }

    fn expr(&mut self) -> Result<Ast, ParseError> {
        // TODO: actual expression parsing
        // probably going to be quite complicated

        // assume integer literals for now

        if self.is(TokenKind::Integer, None)? {
            let value = self
                .current
                .as_ref()
                .unwrap()
                .span
                .content()
                .parse()
                .unwrap();

            self.nom();

            // todo: handle invalid ints
            Ok(Ast::Integer(value))
        } else {
            Err(ParseError::ExpectedTokenGot(
                vec![
                    (TokenKind::Integer, None),
                    (TokenKind::String, None),
                    (TokenKind::Float, None),
                    (TokenKind::Boolean, None),
                ],
                self.current.clone().as_option(),
            ))
        }
    }

    fn static_definition(&mut self, is_public: bool) -> Result<Ast, ParseError> {
        let name = self
            .expect(TokenKind::Identifier, None)?
            .span
            .content()
            .to_string();

        self.expect(TokenKind::Colon, None)?;
        let type_ = self.type_()?;
        self.expect(TokenKind::Equals, None)?;
        let body = self.expr()?;
        self.expect(TokenKind::Semicolon, None)?;

        Ok(Ast::Static {
            public: is_public,
            name: name,
            type_: Box::new(type_),
            body: Box::new(body),
        })
    }

    fn class_definition(&mut self, is_public: bool) -> Result<Ast, ParseError> {
        let name = self
            .expect(TokenKind::Identifier, None)?
            .span
            .content()
            .to_string();

        // TODO: inheritance, implements

        self.expect(TokenKind::Open, Some(&"{"))?;

        // TODO: handle the body

        self.expect(TokenKind::Close, Some(&"}"))?;

        Ok(Ast::Class {
            public: is_public,
            name: name,
            body: vec![],
        })
    }

    fn global_declaration(&mut self, is_public: bool) -> Result<Ast, ParseError> {
        if self.accept(TokenKind::Identifier, Some(&"import"))? {
            self.import_statement()
        } else if self.accept(TokenKind::Identifier, Some(&"pub"))? {
            self.global_declaration(true)
        } else if self.accept(TokenKind::Identifier, Some(&"static"))? {
            self.static_definition(is_public)
        } else if self.accept(TokenKind::Identifier, Some(&"class"))? {
            self.class_definition(is_public)
        } else {
            panic!("unknown token: {:?}", self.current);
        }
    }

    fn program(&mut self) -> Result<Ast, ParseError> {
        let mut nodes = vec![];

        if self.current.is_none() {
            return Ok(Ast::Program(nodes));
        }

        nodes.push(self.global_declaration(false)?);

        while !self.current.is_none() {
            nodes.push(self.global_declaration(false)?);
        }

        if self.current.is_ok() {
            return Err(ParseError::UnexpectedToken(
                self.current.clone().as_option(),
            ));
        }

        Ok(Ast::Program(nodes))
    }
}
