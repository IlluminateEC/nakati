use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{
    ast::Spanned,
    common::Source,
    interpreter::{Interpreter, InterpreterError, Scope},
    lexer::Lexer,
    parser::Parser,
};

#[derive(Clone)]
pub struct ImportPool {
    scopes: Arc<RwLock<HashMap<String, Arc<Scope>>>>,
}

impl ImportPool {
    pub fn new() -> Self {
        Self {
            scopes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &String) -> Option<Arc<Scope>> {
        self.scopes.read().unwrap().get(name).cloned()
    }

    pub fn set(&self, name: String, scope: Arc<Scope>) {
        self.scopes.write().unwrap().insert(name, scope);
    }
}

/// Parses and resolves import labels
/// ```text
/// import Out from "@nakati/io";
///                 ^^^^^^^^^^^^ label
///
/// import module as io from "@nakati/io";
/// ```
///
/// Imports should be pooled into a global import table
/// Caching their evaluated Scope
///
/// Valid labels:
///
/// - @dev//package:module
/// - @dev//package
/// - //package:module
/// - //package
/// - :module
/// - module
/// - #module
///
/// - @nakati//std:io
/// - #io
#[derive(Clone, PartialEq, Debug)]
pub struct ModuleLabel {
    developer: Option<String>,
    package: Option<String>,
    path: Option<String>,

    special: Option<&'static str>,
}

impl ModuleLabel {
    pub fn parse(label: Spanned<String>) -> Result<Self, InterpreterError> {
        let value = label.value.clone();

        let (developer, value) = Self::parse_developer(value);
        let (package, value) = Self::parse_developer(value);
        let (module, value) = Self::parse_developer(value);

        if !value.is_empty() {
            // TODO: deal with syntax error in label
            panic!();
        }

        Ok(Self {
            developer: Self::empty_to_none(developer),
            package: Self::empty_to_none(package),
            path: Self::empty_to_none(module),

            special: None,
        })

        // if value.starts_with("@") {
        //     // @dev//package
        //     // @dev//package:module
        // } else if value.starts_with("//") {
        //     // //package:module
        //     // //package
        // } else {
        //     // :module
        //     // ./module
        //     // module
        // }
    }

    fn empty_to_none(part: String) -> Option<String> {
        if part.is_empty() { None } else { Some(part) }
    }

    fn parse_developer(label: String) -> (String, String) {}

    pub fn fill_from(&self, context: ModuleLabel) -> Self {
        Self {
            developer: Some(
                self.developer
                    .clone()
                    .unwrap_or(context.developer.expect("context's developer to exist")),
            ),
            package: Some(
                self.package
                    .clone()
                    .unwrap_or(context.package.expect("context's package to exist")),
            ),
            path: Some(self.path.clone().unwrap_or("/".to_string())),

            special: self.special,
        }
    }

    pub fn from_special(special: &'static str) -> Self {
        Self {
            developer: None,
            package: None,
            path: None,
            special: Some(special),
        }
    }

    pub fn absolute(&self, context: ModuleLabel) -> String {
        // TODO: handle relative paths

        if let Some(special) = self.special {
            return format!("<{}>", special);
        }

        let filled = self.fill_from(context);

        format!(
            "@{}/{}//{}",
            filled.developer.unwrap(),
            filled.package.unwrap(),
            filled.path.unwrap(),
        )
    }

    fn to_path(&self, context: ModuleLabel) -> PathBuf {
        let filled = self.fill_from(context);

        // idk how to do this rn

        // self.

        PathBuf::new()
    }

    pub fn resolve(
        &self,
        pool: ImportPool,
        context: ModuleLabel,
    ) -> Result<Arc<Scope>, InterpreterError> {
        if let Some(scope) = pool.get(&self.absolute(context.clone())) {
            return Ok(scope);
        }

        let path = self.to_path(context);
        let source = Source::from_path(path);
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;

        let scope = Interpreter::evaluate_module(ast, self.clone(), pool);

        scope
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::Spanned, common::Span, resolver::ModuleLabel};

    fn make_spanned(value: &str) -> Spanned<String> {
        Spanned::new(value.to_string(), Span::new_string(value, "<test>"))
    }

    #[test]
    fn parses() {
        assert_eq!(
            ModuleLabel::parse(make_spanned("@nakati/io")),
            ModuleLabel {
                developer: Some("nakati".to_string()),
                package: Some("io".to_string()),
                path: None,
                special: None
            }
        );
    }
}
