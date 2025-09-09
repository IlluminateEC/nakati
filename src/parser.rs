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
        if items.is_empty() {
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
                    expected.iter().map(Self::format_token_pair).collect()
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
        let current = lexer.next_token();
        let next = lexer.next_token();

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

        self.next = self.lexer.next_token();
    }

    fn is(&self, kind: TokenKind, content: Option<&dyn AsRef<str>>) -> Result<bool, ParseError> {
        if self.current.is_none() {
            return Ok(false);
        }

        if self.current.is_error() {
            return Err(self.current.clone().as_result().err().unwrap().into());
        }

        Ok(self
            .current
            .as_ref()
            .unwrap()
            .is_and_has_content(kind, content))
    }

    fn next_is(&self, kind: TokenKind, content: Option<&dyn AsRef<str>>) -> bool {
        match &self.next {
            OptionalResult::Ok(v) => v.is_and_has_content(kind, content),
            _ => false,
        }
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

    fn accept_option(
        &mut self,
        kind: TokenKind,
        content: Option<&dyn AsRef<str>>,
    ) -> Result<Option<Token>, ParseError> {
        if self.is(kind, content)? {
            let return_ = Ok(Some(self.current.clone().unwrap()));
            self.nom();
            return_
        } else {
            Ok(None)
        }
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
}

impl Parser {
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

    fn expression(&mut self) -> Result<Ast, ParseError> {
        // TODO: actual expression parsing
        // probably going to be quite complicated

        // assume integer literals for now

        // TODO: method calls, bools, strings, floats
        // TODO: math expressions

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

            // TODO: handle invalid ints
            Ok(Ast::Integer(value))
        } else if self.is(TokenKind::Open, Some(&"{"))? {
            self.block()
        } else if self.is(TokenKind::Identifier, None)? {
            // TODO: handle method calls

            let name = self.current.as_ref().unwrap().content();
            self.nom();

            Ok(Ast::VariableAccess(name))
        } else {
            Err(ParseError::ExpectedTokenGot(
                vec![
                    (TokenKind::Integer, None),
                    (TokenKind::String, None),
                    (TokenKind::Float, None),
                    (TokenKind::Boolean, None),
                    (TokenKind::Open, Some("{".to_string())),
                ],
                self.current.clone().as_option(),
            ))
        }
    }

    fn statement(&mut self) -> Result<Ast, ParseError> {
        // TODO: patterns for assignment :(

        if self.is(TokenKind::Identifier, None)? && self.next_is(TokenKind::Equals, None) {
            let name = self.expect(TokenKind::Identifier, None)?.content();
            self.expect(TokenKind::Equals, None)?;
            let value = self.expression()?;

            Ok(Ast::Assignment {
                name,
                body: Box::new(value),
            })
        } else if self.accept(TokenKind::Identifier, Some(&"let"))? {
            let name = self.expect(TokenKind::Identifier, None)?.content();
            let mut type_ = None;

            if self.accept(TokenKind::Colon, None)? {
                type_ = Some(self.type_()?);
            }

            self.expect(TokenKind::Equals, None)?;
            let value = self.expression()?;

            Ok(Ast::Let {
                name,
                type_: Box::new(type_),
                body: Box::new(value),
            })
        } else {
            Err(ParseError::ExpectedTokenGot(
                vec![
                    (TokenKind::Identifier, None),
                    (TokenKind::Identifier, Some("let".to_string())),
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
        let body = self.expression()?;
        self.expect(TokenKind::Semicolon, None)?;

        Ok(Ast::Static {
            public: is_public,
            name,
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
            name,
            body: vec![],
        })
    }

    fn function_argument(&mut self, name: Token) -> Result<(String, Ast), ParseError> {
        self.expect(TokenKind::Colon, None)?;
        let type_ = self.type_()?;
        Ok((name.content(), type_))
    }

    fn function_arguments(&mut self) -> Result<Vec<(String, Ast)>, ParseError> {
        self.expect(TokenKind::Open, Some(&"("))?;

        let mut args = vec![];

        if let Some(name) = self.accept_option(TokenKind::Identifier, None)? {
            args.push(self.function_argument(name)?);
        }

        while self.accept(TokenKind::Comma, None)? {
            let name = self.expect(TokenKind::Identifier, None)?;

            args.push(self.function_argument(name)?);
        }

        self.expect(TokenKind::Close, Some(&")"))?;

        Ok(args)
    }

    fn starts_expression(&mut self) -> Result<bool, ParseError> {
        // TODO: this
        Ok(self.is(TokenKind::Identifier, None)?
            || self.is(TokenKind::Integer, None)?
            || self.is(TokenKind::Open, Some(&"{"))?)
    }

    fn starts_statement(&mut self) -> Result<bool, ParseError> {
        // TODO: this
        Ok(self.is(TokenKind::Identifier, Some(&"let"))?
            || (self.is(TokenKind::Identifier, None)? && self.next_is(TokenKind::Equals, None)))
    }

    fn block(&mut self) -> Result<Ast, ParseError> {
        let mut bloc = vec![];
        let mut return_ = None;

        self.expect(TokenKind::Open, Some(&"{"))?;

        loop {
            if self.starts_statement()? {
                let statement = self.statement()?;
                bloc.push(statement);
                self.expect(TokenKind::Semicolon, None)?;
            } else if self.starts_expression()? {
                let expression = self.expression()?;

                if self.accept(TokenKind::Semicolon, None)? {
                    bloc.push(expression);
                    continue;
                } else {
                    return_ = Some(expression);
                    break;
                }
            }

            // i have *no* idea what i am doing
            // - naki
            if self.is(TokenKind::Close, Some(&"}"))? {
                break;
            }
        }

        self.expect(TokenKind::Close, Some(&"}"))?;

        Ok(Ast::Block {
            statements: bloc,
            return_: Box::new(return_),
        })
    }

    fn function_definition(&mut self, is_public: bool) -> Result<Ast, ParseError> {
        let name = self.expect(TokenKind::Identifier, None)?.content();
        let args = self.function_arguments()?;

        let mut return_type: Option<Ast> = None;

        if self.accept(TokenKind::Colon, None)? {
            return_type = Some(self.type_()?);
        }

        let body: Ast;

        if self.accept(TokenKind::Equals, None)? {
            body = self.expression()?;
            self.expect(TokenKind::Semicolon, None)?;
        } else if self.is(TokenKind::Open, Some(&"{"))? {
            body = self.block()?;
        } else {
            return Err(ParseError::ExpectedTokenGot(
                vec![
                    (TokenKind::Equals, Some("=".to_string())),
                    (TokenKind::Open, Some("{".to_string())),
                ],
                self.current.clone().as_option(),
            ));
        }

        Ok(Ast::Function {
            public: is_public,
            name,
            args,
            return_: Box::new(return_type),
            body: Box::new(body),
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
        } else if self.accept(TokenKind::Identifier, Some(&"fn"))? {
            self.function_definition(is_public)
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
