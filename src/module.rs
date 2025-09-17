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

    special: Option<String>,
}

impl ModuleLabel {
    pub fn parse(label: Spanned<String>) -> Result<Self, InterpreterError> {
        let value = &label.value.clone();

        if let Some(value) = value.strip_prefix("#") {
            return Ok(Self {
                developer: Some("nakati".to_string()),
                package: Some("std".to_string()),
                path: Some(value.to_string()),
                special: None,
            });
        }

        if value.starts_with("<") && value.ends_with(">") {
            let value = &value[1..value.len() - 1];

            return Ok(Self {
                developer: None,
                package: None,
                path: None,
                special: Some(value.to_string()),
            });
        }

        let (developer, value) = Self::parse_developer(value);
        let (package, value) = Self::parse_package(value);
        let (module, value) = Self::parse_module(value);

        let mut module = Self::empty_to_none(module);

        if !developer.is_empty() && package.is_empty() {
            return Err(InterpreterError::InvalidImport(
                "Package must be specified if developer is specified.".to_string(),
                label.span,
            ));
        }

        if module.is_none() && !package.is_empty() {
            module = Some(package.split("/").last().unwrap().to_string());
        }

        if !value.is_empty() {
            // TODO: deal with syntax error in label
            panic!("{}", value);
        }

        Ok(Self {
            developer: Self::empty_to_none(developer),
            package: Self::empty_to_none(package),
            path: module,

            special: None,
        })
    }

    fn empty_to_none(part: String) -> Option<String> {
        if part.is_empty() { None } else { Some(part) }
    }

    fn parse_developer(mut label: &str) -> (String, &str) {
        let mut buf = String::new();

        if !label.starts_with("@") {
            return (buf, label);
        }

        label = &label[1..];

        while !label.starts_with("//") && !label.is_empty() {
            let ch = label.chars().next().unwrap();

            buf += &ch.to_string();

            label = &label[ch.len_utf8()..];
        }

        (buf, label)
    }

    fn parse_package(mut label: &str) -> (String, &str) {
        let mut buf = String::new();

        if !label.starts_with("//") {
            return (buf, label);
        }

        label = &label[2..];

        while !label.starts_with(":") && !label.is_empty() {
            let ch = label.chars().next().unwrap();

            buf += &ch.to_string();

            label = &label[ch.len_utf8()..];
        }

        (buf, label)
    }

    fn parse_module(mut label: &str) -> (String, &str) {
        let mut buf = String::new();

        if label.starts_with(":") {
            label = &label[1..];
        }

        while !label.is_empty() {
            let ch = label.chars().next().unwrap();

            buf += &ch.to_string();

            label = &label[ch.len_utf8()..];
        }

        (buf, label)
    }

    pub fn fill_from(&self, context: ModuleLabel) -> Self {
        Self {
            developer: Some(
                self.developer
                    .clone()
                    .unwrap_or_else(|| context.developer.expect("context's developer to exist")),
            ),
            package: Some(
                self.package
                    .clone()
                    .unwrap_or_else(|| context.package.expect("context's package to exist")),
            ),
            path: Some(self.path.clone().unwrap_or("/".to_string())),

            special: self.special.clone(),
        }
    }

    pub fn from_special(special: impl ToString) -> Self {
        Self {
            developer: None,
            package: None,
            path: None,
            special: Some(special.to_string()),
        }
    }

    pub fn absolute(&self, context: ModuleLabel) -> String {
        // TODO: handle relative paths

        if let Some(special) = &self.special {
            return format!("<{}>", special);
        }

        let filled = self.fill_from(context);

        format!(
            "@{}//{}:{}",
            filled.developer.unwrap(),
            filled.package.unwrap(),
            filled.path.unwrap(),
        )
    }

    fn to_path(&self, context: ModuleLabel) -> PathBuf {
        let filled = self.fill_from(context);

        // TODO: to_path

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
    use crate::{ast::Spanned, common::Span, module::ModuleLabel};

    fn make_spanned(value: &str) -> Spanned<String> {
        Spanned::new(value.to_string(), Span::new_string(value, "<test>"))
    }

    #[test]
    fn special() {
        assert_eq!(
            ModuleLabel::parse(make_spanned("<builtins>")).unwrap(),
            ModuleLabel {
                developer: None,
                package: None,
                path: None,
                special: Some("builtins".to_string())
            }
        );
    }

    #[test]
    fn stdlib() {
        assert_eq!(
            ModuleLabel::parse(make_spanned("#io")).unwrap(),
            ModuleLabel {
                developer: Some("nakati".to_string()),
                package: Some("std".to_string()),
                path: Some("io".to_string()),
                special: None
            }
        );
    }

    #[test]
    fn developer_package_module() {
        assert_eq!(
            ModuleLabel::parse(make_spanned("@developer//package:module")).unwrap(),
            ModuleLabel {
                developer: Some("developer".to_string()),
                package: Some("package".to_string()),
                path: Some("module".to_string()),
                special: None
            }
        );
    }

    #[test]
    fn package_module() {
        assert_eq!(
            ModuleLabel::parse(make_spanned("//package:module")).unwrap(),
            ModuleLabel {
                developer: None,
                package: Some("package".to_string()),
                path: Some("module".to_string()),
                special: None
            }
        );
    }

    #[test]
    fn package() {
        assert_eq!(
            ModuleLabel::parse(make_spanned("//package")).unwrap(),
            ModuleLabel {
                developer: None,
                package: Some("package".to_string()),
                path: Some("package".to_string()),
                special: None
            }
        );
    }

    #[test]
    fn module() {
        assert_eq!(
            ModuleLabel::parse(make_spanned("module")).unwrap(),
            ModuleLabel {
                developer: None,
                package: None,
                path: Some("module".to_string()),
                special: None
            }
        );

        assert_eq!(
            ModuleLabel::parse(make_spanned(":module")).unwrap(),
            ModuleLabel {
                developer: None,
                package: None,
                path: Some("module".to_string()),
                special: None
            }
        );
    }
}
