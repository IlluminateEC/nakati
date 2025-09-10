use std::{
    fmt::Debug,
    ops::{FromResidual, Try},
    rc::Rc,
};

use unicode_segmentation::UnicodeSegmentation;

pub struct Source {
    pub name: String,
    pub body: String,
    pub graphemes: Vec<String>,
}

impl Source {
    pub fn new(name: impl ToString, body: impl ToString) -> Rc<Self> {
        let new_body = body.to_string();
        let graphemes = new_body.graphemes(true).map(|g| g.to_string()).collect();

        Rc::new(Self {
            name: name.to_string(),
            body: new_body,
            graphemes,
        })
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Rc<Self> {
        // TODO: error handling

        let content = std::fs::read_to_string(path.as_ref());

        Self::new(path.as_ref().to_str().unwrap(), content.unwrap())
    }
}

impl Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<Source name={:?}>", self.name))
    }
}

#[derive(Clone)]
pub struct Span {
    source: Rc<Source>,

    start: usize,
    start_line: usize,
    start_column: usize,
    start_grapheme: usize,

    end: usize,
    end_line: usize,
    end_column: usize,
    end_grapheme: usize,
}

impl Span {
    pub fn new(source: Rc<Source>) -> Self {
        Self {
            source,

            start: 0,
            start_column: 0,
            start_line: 0,
            start_grapheme: 0,

            end: 0,
            end_column: 0,
            end_line: 0,
            end_grapheme: 0,
        }
    }

    pub fn new_string(string: impl ToString) -> Self {
        let source = Source::new("<internal value>", string);

        let end = source.body.len();
        let end_column = source.body.len();
        let end_grapheme = source.graphemes.len();

        Self {
            source,

            start: 0,
            start_column: 0,
            start_line: 0,
            start_grapheme: 0,

            end,
            end_column,
            end_line: 0,
            end_grapheme,
        }
    }

    pub fn bump(&mut self, value: impl AsRef<str>) {
        for char in value.as_ref().chars() {
            self.end += char.len_utf8();
            self.end_column += 1;
            self.end_grapheme += 1;

            if char == '\n' {
                self.end_line += 1;
                self.end_column = 0;
            }
        }
    }

    pub fn bump_cur(&mut self) {
        if let Some(cur) = self.cur() {
            self.bump(cur);
        }
    }

    pub fn release(&mut self) {
        self.start = self.end;
        self.start_column = self.end_column;
        self.start_line = self.end_line;
        self.start_grapheme = self.end_grapheme;
    }

    pub fn content(&self) -> &str {
        &self.source.body[self.start..self.end]
    }

    pub fn cur(&self) -> Option<&'static String> {
        // Actually has a performance improvement over just returning a cloned String somehow.
        unsafe { std::mem::transmute(self.source.graphemes.get(self.end_grapheme)) }
    }

    pub fn bump_while(&mut self, predicate: impl Fn(&str) -> bool) {
        while let Some(grapheme) = self.cur() {
            if !predicate(grapheme) {
                break;
            }

            self.bump(grapheme);
        }
    }

    pub fn skip_while(&mut self, predicate: impl Fn(&str) -> bool) {
        while let Some(grapheme) = self.cur() {
            if !predicate(grapheme) {
                break;
            }

            self.bump(grapheme);
        }

        self.release();
    }

    pub fn has(&self, value: impl AsRef<str>) -> bool {
        let graphemes = value
            .as_ref()
            .graphemes(true)
            .enumerate()
            .collect::<Vec<_>>();

        for (idx, grapheme) in graphemes {
            let src_grapheme = self.source.graphemes.get(self.end_grapheme + idx);

            if src_grapheme.is_none() {
                return false;
            }

            if src_grapheme.unwrap() != grapheme {
                return false;
            }
        }

        true
    }

    pub fn fit(&mut self, other: &Self) {
        if other.start < self.start {
            self.start = other.start;
            self.start_line = other.start_line;
            self.start_column = other.start_column;
            self.start_grapheme = other.start_grapheme;
        }

        if other.end > self.end {
            self.end = other.end;
            self.end_line = other.end_line;
            self.end_column = other.end_column;
            self.end_grapheme = other.end_grapheme;
        }
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.start_line == other.start_line
            && self.start_column == other.start_column
            && self.start_grapheme == other.start_grapheme
            && self.end == other.end
            && self.end_line == other.end_line
            && self.end_column == other.end_column
            && self.end_grapheme == other.end_grapheme
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<{}:{} length {}>",
            self.start_line + 1,
            self.start_column + 1,
            self.end - self.start
        ))
    }
}

#[must_use]
#[derive(Copy, Eq, Debug, Hash, Clone, PartialEq)]
pub enum OptionalResult<T, E> {
    Ok(T),
    Err(E),
    None,
}

pub type OpRes<T, E> = OptionalResult<T, E>;

impl<T, E> OptionalResult<T, E> {
    #[inline]
    pub fn unwrap(self) -> T
    where
        E: Debug,
    {
        match self {
            Self::Ok(v) => v,
            Self::Err(e) => panic!("Could not unwrap Err value: {:?}", e),
            Self::None => panic!("Could not unwrap None value."),
        }
    }

    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    #[inline]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Err(_))
    }

    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn as_result(self) -> Result<T, E> {
        match self {
            Self::Ok(v) => Ok(v),
            Self::Err(e) => Err(e),
            Self::None => panic!("Cannot convert None into a Result"),
        }
    }

    #[inline]
    pub fn as_option(self) -> Option<T>
    where
        E: Debug,
    {
        match self {
            Self::Ok(v) => Some(v),
            Self::Err(e) => panic!("Could not convert Err value into Option: {:?}", e),
            Self::None => None,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> OptionalResult<&T, &E> {
        match self {
            Self::Ok(v) => OptionalResult::Ok(v),
            Self::Err(e) => OptionalResult::Err(e),
            Self::None => OptionalResult::None,
        }
    }

    #[inline]
    pub fn map<U>(self, op: impl Fn(T) -> U) -> OptionalResult<U, E> {
        match self {
            Self::Ok(v) => OptionalResult::Ok(op(v)),
            Self::Err(e) => OptionalResult::Err(e),
            Self::None => OptionalResult::None,
        }
    }

    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Ok(v) => v,
            Self::Err(_) => default,
            Self::None => default,
        }
    }
}

impl<T, E> From<Option<T>> for OptionalResult<T, E> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => Self::Ok(v),
            None => Self::None,
        }
    }
}

impl<T, E> From<Result<T, E>> for OptionalResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(t) => Self::Ok(t),
            Err(e) => Self::Err(e),
        }
    }
}

impl<T, E> From<T> for OptionalResult<T, E> {
    fn from(value: T) -> Self {
        Self::Ok(value)
    }
}

impl<T, E> FromResidual for OptionalResult<T, E> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        residual
    }
}

impl<T, E> Try for OptionalResult<T, E> {
    type Output = T;

    type Residual = Self;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::None => std::ops::ControlFlow::Break(self),
            Self::Err(_) => std::ops::ControlFlow::Break(self),
            Self::Ok(v) => std::ops::ControlFlow::Continue(v),
        }
    }
}
