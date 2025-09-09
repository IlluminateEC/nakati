use std::fmt::Debug;

use crate::{
    ast::{Ast, AstNode},
    common::OptionalResult,
    lexer::{LexError, Lexer},
    token::{Token, TokenKind},
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

    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        self.program()
    }
}

impl Parser {
    fn nom(&mut self) {
        std::mem::swap(&mut self.current, &mut self.next);

        self.next = self.lexer.next_token();
    }

    fn is(&self, kind: TokenKind, content: Option<&str>) -> Result<bool, ParseError> {
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

    fn next_is(&self, kind: TokenKind, content: Option<&str>) -> bool {
        match &self.next {
            OptionalResult::Ok(v) => v.is_and_has_content(kind, content),
            _ => false,
        }
    }

    fn accept(&mut self, kind: TokenKind, content: Option<&str>) -> Result<bool, ParseError> {
        if self.current.is_none() {
            return Ok(false);
        }

        if self.current.is_error() {
            return Err(self.current.clone().as_result().err().unwrap().into());
        }

        let current = self.current.clone().as_option().unwrap();

        if current.kind == kind {
            if let Some(content) = content {
                if current.span.content() == content {
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
        content: Option<&str>,
    ) -> Result<Option<Token>, ParseError> {
        if self.is(kind, content)? {
            let return_ = Ok(Some(self.current.clone().unwrap()));
            self.nom();
            return_
        } else {
            Ok(None)
        }
    }

    fn expect(&mut self, kind: TokenKind, content: Option<&str>) -> Result<Token, ParseError> {
        let current = self.current.clone();

        if self.accept(kind, content)? {
            return Ok(current.unwrap());
        }

        Err(ParseError::ExpectedTokenGot(
            vec![(kind, content.map(|s| s.to_string()))],
            current.clone().as_option(),
        ))
    }

    // TODO: make it return a closure that takes an argument
    #[inline]
    fn branch(&mut self, branches: &[FinishedBranch]) -> ParseResult<AstNode> {
        for FinishedBranch {
            kind,
            content,
            predicate,
            handler,
            consume,
        } in branches
        {
            if self.is(*kind, *content)? && predicate.map(|f| f(self)).unwrap_or(true) {
                if *consume {
                    self.nom();
                }

                return handler(self);
            }
        }

        Err(ParseError::ExpectedTokenGot(
            branches
                .iter()
                .map(|branch| {
                    (
                        branch.kind.to_owned(),
                        branch.content.map(|s| s.to_string()),
                    )
                })
                .collect(),
            self.current.clone().as_option(),
        ))
    }
}

type ParseResult<T> = Result<T, ParseError>;
// type TokenContent = &'static dyn AsRef<str>;
type Predicate<'a> = &'a dyn Fn(&mut Parser) -> bool;
type Handler<'a> = &'a dyn Fn(&mut Parser) -> ParseResult<AstNode>;

struct Branch<'a> {
    kind: TokenKind,
    content: Option<&'static str>,
    predicate: Option<Predicate<'a>>,
    consume: bool,
}

impl<'a> Branch<'a> {
    #[inline]
    pub fn of_kind(kind: TokenKind) -> Self {
        Self {
            kind,
            content: None,
            predicate: None,
            consume: false,
        }
    }

    #[inline]
    pub fn with_content(mut self, content: &'static str) -> Self {
        self.content = Some(content);
        self
    }

    #[inline]
    pub fn when(mut self, predicate: Predicate<'a>) -> Self {
        self.predicate = Some(predicate);
        self
    }

    #[inline]
    pub fn consume(mut self) -> Self {
        self.consume = !self.consume;
        self
    }

    #[inline]
    pub fn then(self, handler: Handler<'a>) -> FinishedBranch<'a> {
        FinishedBranch {
            kind: self.kind,
            content: self.content,
            predicate: self.predicate,
            consume: self.consume,
            handler,
        }
    }
}

struct FinishedBranch<'a> {
    kind: TokenKind,
    content: Option<&'static str>,
    predicate: Option<Predicate<'a>>,
    consume: bool,
    handler: Handler<'a>,
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

    fn import_statement(&mut self) -> Result<AstNode, ParseError> {
        self.expect(TokenKind::Identifier, Some("import"))?;

        let mut names = vec![];

        names.push(self.import_name()?);

        while self.accept(TokenKind::Comma, None)? {
            names.push(self.import_name()?);
        }

        self.expect(TokenKind::Identifier, Some(&"from"))?;

        let module = self.expect(TokenKind::String, None)?;

        self.expect(TokenKind::Semicolon, None)?;

        Ok(AstNode::new(Ast::Import {
            names,
            module: module.span.content().to_string(),
        }))
    }

    fn type_(&mut self) -> Result<AstNode, ParseError> {
        // TODO: actual parsing of types
        // generics

        let name = self.expect(TokenKind::Identifier, None)?;

        Ok(AstNode::new(Ast::TypeName(name.span.content().to_string())))
    }

    fn expression(&mut self) -> Result<AstNode, ParseError> {
        // TODO: actual expression parsing
        // probably going to be quite complicated

        // assume integer literals for now

        // TODO: method calls, bools, strings, floats
        // TODO: math expressions

        self.branch(&[
            Branch::of_kind(TokenKind::Integer).then(&|this| {
                let value = this
                    .current
                    .as_ref()
                    .unwrap()
                    .span
                    .content()
                    .parse()
                    .unwrap();

                this.nom();

                // TODO: handle invalid ints
                Ok(AstNode::new(Ast::Integer(value)))
            }),
            Branch::of_kind(TokenKind::Open)
                .with_content(&"{")
                .then(&Self::block),
            Branch::of_kind(TokenKind::Identifier).then(&|this| {
                // TODO: handle method calls

                let name = this.current.as_ref().unwrap().content();
                this.nom();

                Ok(AstNode::new(Ast::VariableAccess(name)))
            }),
        ])
    }

    fn statement(&mut self) -> Result<AstNode, ParseError> {
        // TODO: patterns for assignment :(

        self.branch(&[
            Branch::of_kind(TokenKind::Identifier)
                .when(&|this| this.next_is(TokenKind::Equals, None))
                .then(&|this| {
                    let name = this.expect(TokenKind::Identifier, None)?.content();
                    this.expect(TokenKind::Equals, None)?;
                    let value = this.expression()?;

                    Ok(AstNode::new(Ast::Assignment { name, body: value }))
                }),
            Branch::of_kind(TokenKind::Identifier)
                .with_content(&"let")
                .consume()
                .then(&|this| {
                    let name = this.expect(TokenKind::Identifier, None)?.content();
                    let mut type_ = None;

                    if this.accept(TokenKind::Colon, None)? {
                        type_ = Some(this.type_()?);
                    }

                    this.expect(TokenKind::Equals, None)?;
                    let value = this.expression()?;

                    Ok(AstNode::new(Ast::Let {
                        name,
                        type_,
                        body: value,
                    }))
                }),
        ])
    }

    fn static_definition(&mut self, is_public: bool) -> Result<AstNode, ParseError> {
        self.expect(TokenKind::Identifier, Some("static"))?;

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

        Ok(Box::new(Ast::Static {
            public: is_public,
            name,
            type_,
            body,
        }))
    }

    fn class_definition(&mut self, is_public: bool) -> Result<AstNode, ParseError> {
        self.expect(TokenKind::Identifier, Some("class"))?;

        let name = self
            .expect(TokenKind::Identifier, None)?
            .span
            .content()
            .to_string();

        // TODO: inheritance, implements

        self.expect(TokenKind::Open, Some(&"{"))?;

        // TODO: handle the body

        self.expect(TokenKind::Close, Some(&"}"))?;

        Ok(Box::new(Ast::Class {
            public: is_public,
            name,
            body: vec![],
        }))
    }

    fn function_argument(&mut self, name: Token) -> Result<(String, AstNode), ParseError> {
        self.expect(TokenKind::Colon, None)?;
        let type_ = self.type_()?;
        Ok((name.content(), type_))
    }

    fn function_arguments(&mut self) -> Result<Vec<(String, AstNode)>, ParseError> {
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

    fn block(&mut self) -> Result<AstNode, ParseError> {
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

        Ok(AstNode::new(Ast::Block {
            statements: bloc,
            return_value: return_,
        }))
    }

    fn function_definition(&mut self, is_public: bool) -> Result<AstNode, ParseError> {
        self.expect(TokenKind::Identifier, Some("fn"))?;

        let name = self.expect(TokenKind::Identifier, None)?.content();
        let args = self.function_arguments()?;

        let mut return_type = None;

        if self.accept(TokenKind::Colon, None)? {
            return_type = Some(self.type_()?);
        }

        let body = self.branch(&[
            Branch::of_kind(TokenKind::Equals).consume().then(&|this| {
                let a = this.expression()?;
                this.expect(TokenKind::Semicolon, None)?;
                Ok(a)
            }),
            Branch::of_kind(TokenKind::Open)
                .with_content(&"{")
                .then(&Self::block),
        ])?;

        Ok(AstNode::new(Ast::Function {
            public: is_public,
            name,
            args,
            return_type,
            body,
        }))
    }

    fn global_declaration(&mut self, is_public: bool) -> Result<AstNode, ParseError> {
        self.branch(&[
            Branch::of_kind(TokenKind::Identifier)
                .with_content("import")
                .then(&Self::import_statement),
            Branch::of_kind(TokenKind::Identifier)
                .with_content("pub")
                .consume()
                .then(&|this| this.global_declaration(true)),
            Branch::of_kind(TokenKind::Identifier)
                .with_content("static")
                .then(&move |this| this.static_definition(is_public)),
            Branch::of_kind(TokenKind::Identifier)
                .with_content("class")
                .then(&move |this| this.class_definition(is_public)),
            Branch::of_kind(TokenKind::Identifier)
                .with_content("fn")
                .then(&move |this| this.function_definition(is_public)),
        ])
    }

    fn program(&mut self) -> Result<AstNode, ParseError> {
        let mut nodes = vec![];

        if self.current.is_ok() {
            nodes.push(self.global_declaration(false)?);

            while !self.current.is_none() {
                nodes.push(self.global_declaration(false)?);
            }

            if self.current.is_ok() {
                return Err(ParseError::UnexpectedToken(
                    self.current.clone().as_option(),
                ));
            }
        }

        Ok(AstNode::new(Ast::Program(nodes)))
    }
}
