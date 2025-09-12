use std::fmt::Debug;

use crate::{
    ast::{Ast, AstNode, SString},
    common::{OptionalResult, Span},
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
    span_stack: Vec<Span>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current = lexer.next_token();
        let next = lexer.next_token();

        Self {
            lexer,
            current,
            next,
            span_stack: vec![],
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        self.program()
    }
}

impl Parser {
    fn push_span(&mut self, span: Span) {
        self.span_stack.push(span);
    }

    fn current_scope(&self) -> Span {
        self.span_stack
            .last()
            .cloned()
            .expect("span stack should not be empty")
    }

    fn current_span(&self) -> Option<Span> {
        let a = self.current.as_ref().map(|token| &token.span);

        if a.is_ok() {
            Some(a.unwrap().clone())
        } else {
            None
        }
    }

    fn begin_scope(&mut self) {
        if let Some(current_span) = self.current_span() {
            self.push_span(current_span);
        }
    }

    fn end_scope(&mut self) -> Span {
        self.span_stack
            .pop()
            .expect("span stack should not be empty")
    }
}

impl Parser {
    fn nom(&mut self) {
        for span in &mut self.span_stack {
            if let OptionalResult::Ok(current) = &self.current {
                span.fit(&current.span);
            }
        }

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
        for branch in branches {
            let FinishedBranch {
                kind,
                content,
                predicate,
                handler,
                consume,
            } = branch;

            if self.is(*kind, *content)? && predicate.map(|f| f(self)).unwrap_or(true) {
                self.begin_scope();

                if *consume {
                    self.nom();
                }

                let ast_node = handler(self);

                self.end_scope();

                return ast_node;
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

    /// Returns a spanned string created from the current identifier token.
    fn current_identifier(&mut self) -> ParseResult<SString> {
        let token = self.expect(TokenKind::Identifier, None)?;

        Ok(SString::new(token.content(), token.span))
    }

    /// Returns a spanned string created from the current string token.
    fn current_string(&mut self) -> ParseResult<SString> {
        let token = self.expect(TokenKind::String, None)?;

        // todo: unescaping

        Ok(SString::new(token.content(), token.span))
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
    /// Begins the process of creating a new branch.
    /// Specifies the token kind to expect.
    pub fn of_kind(kind: TokenKind) -> Self {
        Self {
            kind,
            content: None,
            predicate: None,
            consume: false,
        }
    }

    #[inline]
    /// Specifies that this branch's token will have specific content
    pub fn with_content(mut self, content: &'static str) -> Self {
        self.content = Some(content);
        self
    }

    #[inline]
    /// Provides a condition for when this should match, beyond the basic kind and content
    pub fn when(mut self, predicate: Predicate<'a>) -> Self {
        self.predicate = Some(predicate);
        self
    }

    #[inline]
    /// Sets that the token should be consumed rather than just checked for
    pub fn consume(mut self) -> Self {
        self.consume = !self.consume;
        self
    }

    #[inline]
    /// Finishes creating the branch by specifying a closure to be run once this branch is matched.
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
    fn import_symbol_and_alias(&mut self) -> Result<(SString, Option<SString>), ParseError> {
        let name = self.current_identifier()?;
        let mut alias = None;

        if self.accept(TokenKind::Identifier, Some(&"as"))? {
            alias = Some(self.current_identifier()?);
        }

        Ok((name, alias))
    }

    fn import_statement(&mut self) -> Result<AstNode, ParseError> {
        self.expect(TokenKind::Identifier, Some("import"))?;

        let mut names = vec![];

        names.push(self.import_symbol_and_alias()?);

        while self.accept(TokenKind::Comma, None)? {
            names.push(self.import_symbol_and_alias()?);
        }

        self.expect(TokenKind::Identifier, Some(&"from"))?;

        let module = self.current_string()?;

        self.expect(TokenKind::Semicolon, None)?;

        Ok(AstNode::new(
            Ast::Import { names, module },
            self.current_scope(),
        ))
    }

    fn type_(&mut self) -> Result<AstNode, ParseError> {
        // TODO: actual parsing of types
        // generics

        self.begin_scope();

        let name = self.expect(TokenKind::Identifier, None)?;

        Ok(AstNode::new(
            Ast::TypeName(name.span.content().to_string()),
            self.end_scope(),
        ))
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
                    .expect("integer should parse correctly");

                this.nom();

                // TODO: handle invalid ints
                Ok(AstNode::new(Ast::Integer(value), this.current_scope()))
            }),
            Branch::of_kind(TokenKind::Open)
                .with_content(&"{")
                .then(&Self::block),
            Branch::of_kind(TokenKind::Identifier).then(&|this| {
                // TODO: handle method calls

                let name = this.current.as_ref().unwrap().content();
                this.nom();

                Ok(AstNode::new(
                    Ast::VariableAccess(name),
                    this.current_scope(),
                ))
            }),
        ])
    }

    fn statement(&mut self) -> Result<AstNode, ParseError> {
        // TODO: patterns for assignment :(

        self.branch(&[
            Branch::of_kind(TokenKind::Identifier)
                .when(&|this| this.next_is(TokenKind::Equals, None))
                .then(&|this| {
                    let name = this.current_identifier()?;
                    this.expect(TokenKind::Equals, None)?;
                    let value = this.expression()?;

                    Ok(AstNode::new(
                        Ast::Assignment {
                            pattern: AstNode::new(Ast::VariableAccess(name.value), name.span),
                            body: value,
                        },
                        this.current_scope(),
                    ))
                }),
            Branch::of_kind(TokenKind::Identifier)
                .with_content(&"let")
                .consume()
                .then(&|this| {
                    let name = this.current_identifier()?;
                    let mut type_ = None;

                    if this.accept(TokenKind::Colon, None)? {
                        type_ = Some(this.type_()?);
                    }

                    this.expect(TokenKind::Equals, None)?;
                    let value = this.expression()?;

                    Ok(AstNode::new(
                        Ast::Let {
                            pattern: AstNode::new(Ast::VariableAccess(name.value), name.span),
                            type_,
                            body: value,
                        },
                        this.current_scope(),
                    ))
                }),
        ])
    }

    fn static_definition(&mut self, is_public: bool) -> Result<AstNode, ParseError> {
        self.expect(TokenKind::Identifier, Some("static"))?;

        let name = self.current_identifier()?;

        self.expect(TokenKind::Colon, None)?;
        let type_ = self.type_()?;
        self.expect(TokenKind::Equals, None)?;
        let body = self.expression()?;
        self.expect(TokenKind::Semicolon, None)?;

        Ok(AstNode::new(
            Ast::Static {
                public: is_public,
                name,
                type_,
                body,
            },
            self.current_scope(),
        ))
    }

    fn class_definition(&mut self, is_public: bool) -> Result<AstNode, ParseError> {
        self.expect(TokenKind::Identifier, Some("class"))?;

        let name = self.current_identifier()?;

        // TODO: inheritance, implements

        self.expect(TokenKind::Open, Some(&"{"))?;

        // TODO: handle the body

        self.expect(TokenKind::Close, Some(&"}"))?;

        Ok(AstNode::new(
            Ast::Class {
                public: is_public,
                name,
                body: vec![],
            },
            self.current_scope(),
        ))
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

    /// does not consume anything unless the entire sequence matches
    /// TODO: this will probably consume a while loop's first line?
    fn beginning_statement_or_expression(&mut self) -> Result<Option<AstNode>, ParseError> {
        self.begin_scope();

        if self.is(TokenKind::Identifier, None)? && self.next_is(TokenKind::Identifier, None) {
            return Ok(None);
        }

        if self.is(TokenKind::Identifier, None)? {
            // todo: return None instead of Err
            let name = self.current_identifier()?;
            let mut access = AstNode::new(Ast::VariableAccess(name.value), name.span);

            while self.is(TokenKind::Dot, None)? || self.is(TokenKind::Open, Some("("))? {
                if self.accept(TokenKind::Dot, None)? {
                    let name = self.current_identifier()?;
                    access = AstNode::new(Ast::MemberAccess(access, name), self.current_scope());
                } else {
                    let args = self.args()?;
                    access = AstNode::new(Ast::Call(access, args), self.current_scope());
                }
            }

            self.end_scope();

            Ok(Some(access))
        } else {
            self.end_scope();
            Ok(None)
        }
    }

    fn args(&mut self) -> ParseResult<Vec<AstNode>> {
        let args = vec![];

        self.expect(TokenKind::Open, Some("("))?;

        self.expect(TokenKind::Close, Some(")"))?;

        Ok(args)
    }

    // fn try_parse_statement()

    // fn try_parse_expression()

    fn block(&mut self) -> ParseResult<AstNode> {
        self.begin_scope();
        let mut bloc = vec![];
        let mut return_ = None;

        self.expect(TokenKind::Open, Some(&"{"))?;

        loop {
            self.begin_scope();

            if let Some(access_expr) = self.beginning_statement_or_expression()? {
                if self.accept(TokenKind::Equals, None)? {
                    let expr = self.expression()?;

                    bloc.push(AstNode::new(
                        Ast::Assignment {
                            pattern: access_expr,
                            body: expr,
                        },
                        self.end_scope(),
                    ));

                    self.expect(TokenKind::Semicolon, None)?;
                } else if self.accept(TokenKind::Semicolon, None)? {
                    bloc.push(access_expr);
                    self.end_scope();
                    continue;
                } else if self.is(TokenKind::Close, Some("}"))? {
                    return_ = Some(access_expr);
                    self.end_scope();
                    break;
                }
            } else if self.starts_statement()? {
                let statement = self.statement()?;
                bloc.push(statement);
                self.expect(TokenKind::Semicolon, None)?;
                self.end_scope();
                continue;
            } else if self.starts_expression()? {
                let expression = self.expression()?;

                if self.accept(TokenKind::Semicolon, None)? {
                    bloc.push(expression);
                    self.end_scope();
                    continue;
                } else {
                    return_ = Some(expression);
                    self.end_scope();
                    break;
                }
            } else if self.is(TokenKind::Close, Some("}"))? {
                self.end_scope();
                break;
            } else {
                return Err(ParseError::UnexpectedToken(
                    self.current.clone().as_option(),
                ));
            }
        }

        self.expect(TokenKind::Close, Some(&"}"))?;

        Ok(AstNode::new(
            Ast::Block {
                statements: bloc,
                return_value: return_,
            },
            self.end_scope(),
        ))
    }

    fn function_definition(&mut self, is_public: bool) -> Result<AstNode, ParseError> {
        self.begin_scope();
        self.expect(TokenKind::Identifier, Some("fn"))?;

        let name = self.current_identifier()?;
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

        Ok(AstNode::new(
            Ast::Function {
                public: is_public,
                name,
                args,
                return_type,
                body,
            },
            self.end_scope(),
        ))
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
        self.begin_scope();
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

        Ok(AstNode::new(Ast::Program(nodes), self.end_scope()))
    }
}
