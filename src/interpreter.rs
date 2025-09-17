use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    ast::{Ast, AstNode},
    common::Span,
    module::{ImportPool, ModuleLabel},
    parser::ParseError,
};

#[derive(Clone)]
pub enum FunctionBody {
    Ast(Arc<AstNode>),
    Native(Arc<dyn Fn(Vec<Value>) -> Result<Value, InterpreterError> + Sync + Send>),
}

impl Debug for FunctionBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ast(arg0) => f.debug_tuple("AstFunction").field(arg0).finish(),
            Self::Native(_) => f.debug_tuple("NativeFunction").finish(),
        }
    }
}

impl PartialEq for FunctionBody {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ast(left), Self::Ast(right)) => left == right,
            (Self::Native(left), Self::Native(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i128),
    Float(f64),
    String(String),
    Boolean(bool),
    Class(Arc<Class>),
    Instance(Arc<Instance>),
    Function(Function),
    Module(Module),
    None,
}

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    scope: Arc<Scope>,
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.scope, &other.scope)
    }
}

// TODO: inheritance, interfaces
#[derive(Debug, Clone)]
pub struct Class {
    name: String,
    fields: HashMap<String, Value>,
    span: Span,
    scope: Arc<Scope>,
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.fields == other.fields
            && self.span == other.span
            && Arc::ptr_eq(&self.scope, &other.scope)
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    class: Arc<Class>,
    fields: HashMap<String, Value>,
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class && self.fields == other.fields
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    // TODO: add typing
    args: Vec<(String, ())>,
    return_type: (),
    body: FunctionBody,
    span: Span,
    scope: Arc<Scope>,
    variadic: bool,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.args == other.args
            && self.return_type == other.return_type
            && self.body == other.body
            && self.span == other.span
            && Arc::ptr_eq(&self.scope, &other.scope)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::Function(Function { name, .. }) => {
                f.write_fmt(format_args!("<function {:?}>", name))
            }

            Self::String(value) => f.write_fmt(format_args!("{}", value)),
            Self::Integer(value) => f.write_fmt(format_args!("{}", value)),

            Self::Class(value) => f.write_fmt(format_args!("<class {:?}>", value.name)),
            Self::Instance(value) => f.write_fmt(format_args!(
                "<{:?} at {:x}>",
                value.class.name,
                value.as_ref() as *const Instance as usize
            )),

            _ => todo!("{:?}", self),
        }
    }
}

// TODO: publics
#[derive(Debug)]
pub struct Scope {
    parent: Option<Arc<Scope>>,
    pairs: RwLock<HashMap<String, Value>>,
    frozen: AtomicBool,
}

impl Scope {
    pub fn new(parent: Option<Arc<Scope>>) -> Arc<Self> {
        Arc::new(Self {
            parent,
            pairs: RwLock::new(HashMap::new()),
            frozen: AtomicBool::new(false),
        })
    }

    pub fn get<'a>(&'a self, key: impl AsRef<str>) -> Option<&'a Value> {
        if let Some(value) = self.pairs.read().unwrap().get(key.as_ref()) {
            return Some(unsafe { std::mem::transmute(value) });
        }

        if let Some(parent) = &self.parent {
            parent.get(key)
        } else {
            None
        }
    }

    /// Find the first definition for a variable and update it if it isn't frozen.
    pub fn update(&self, key: impl ToString, value: Value) -> Result<(), InterpreterError> {
        let did_update = self.update_internal(key.to_string(), value);

        if did_update {
            Ok(())
        } else {
            Err(InterpreterError::MutabilityError(format!(
                "Cannot mutate frozen variable {:?}",
                key.to_string()
            )))
        }
    }

    /// Define a new variable in the current scope.
    pub fn define(&self, key: impl ToString, value: Value) -> Result<(), InterpreterError> {
        let key = key.to_string();

        if self.frozen.load(Ordering::Relaxed) {
            return Err(InterpreterError::MutabilityError(format!(
                "Cannot define variable {:?} in frozen scope",
                &key
            )));
        }

        self.pairs.write().unwrap().insert(key, value);

        Ok(())
    }

    pub fn has(&self, key: &String) -> bool {
        if self.pairs.read().unwrap().contains_key(key) {
            true
        } else if let Some(parent) = &self.parent {
            parent.has(key)
        } else {
            false
        }
    }

    fn update_internal(&self, key: String, value: Value) -> bool {
        // Assume parent scopes are also frozen.
        if self.frozen.load(Ordering::Relaxed) {
            return false;
        }

        if self.pairs.read().unwrap().contains_key(&key) {
            self.pairs.write().unwrap().insert(key, value);

            return true;
        }

        if let Some(parent) = &self.parent {
            parent.update_internal(key, value)
        } else {
            false
        }
    }

    pub fn freeze(&self) {
        self.frozen.store(true, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub enum InterpreterError {
    TypeError(String, Span),
    UndefinedVariable(String, Span),
    ArgumentError(String, Span),
    PatternError(String, Span, Span),
    // TODO: add spans
    MutabilityError(String),
    SyntaxError(ParseError),
    InvalidImport(String, Span),
}

impl From<ParseError> for InterpreterError {
    fn from(value: ParseError) -> Self {
        Self::SyntaxError(value)
    }
}

// TODO: add spans to track all definitions?

pub struct Interpreter {
    module_pool: ImportPool,
    scope_stack: Vec<Arc<Scope>>,
    label: ModuleLabel,
}

impl Interpreter {
    pub fn interpret(ast: AstNode, label: ModuleLabel) -> Result<Value, InterpreterError> {
        let scope = Self::evaluate_module(ast, label, Self::make_default_imports())?;

        // TODO: return return value of main
        Ok(Value::None)
    }

    pub fn evaluate_module(
        ast: AstNode,
        label: ModuleLabel,
        pool: ImportPool,
    ) -> Result<Arc<Scope>, InterpreterError> {
        let builtins = pool.get(&"<builtins>".to_string()).unwrap();

        let mut this = Self {
            scope_stack: vec![builtins],
            module_pool: pool.clone(),
            label,
        };

        this.begin_scope();

        this.interpret_internal(&ast)?;

        let scope = this.current_scope().clone();

        this.end_scope();

        Ok(scope)
    }

    fn make_default_imports() -> ImportPool {
        let pool = ImportPool::new();

        pool.set("<builtins>".to_string(), Self::make_builtins());
        pool.set("@nakati//std:io".to_string(), {
            let io = Scope::new(pool.get(&"<builtins>".to_string()));

            io.define("life", Value::Integer(42)).unwrap();

            // io.define("OStream", Value::Class(Arc::new()))

            // io.define("Out", Value::Instance())?;

            // TODO: Err
            // TODO: In

            io
        });

        pool
    }

    fn make_builtins() -> Arc<Scope> {
        let builtins = Scope::new(None);

        let print_span = Span::new_string("pub fn print() { ... }", "<builtins>");
        builtins
            .define(
                "print",
                Value::Function(Function {
                    name: "print".to_string(),
                    args: vec![("value".to_string(), ())],
                    return_type: (),
                    body: FunctionBody::Native(Arc::new(move |values| {
                        if values.is_empty() {
                            println!();
                        }

                        if values.len() == 1 {
                            let value = values[0].clone();
                            println!("{}", value);
                            Ok(value)
                        } else {
                            let joined = values
                                .iter()
                                .map(|v| format!("{}", v))
                                .intersperse(" ".to_string())
                                .collect::<String>();

                            println!("{}", joined);

                            Ok(Value::None)
                        }
                    })),
                    span: print_span,
                    scope: builtins.clone(),
                    variadic: true,
                }),
            )
            .unwrap();

        builtins.freeze();

        builtins
    }

    fn begin_scope(&mut self) {
        self.scope_stack
            .push(Scope::new(self.scope_stack.last().cloned()))
    }

    fn current_scope(&mut self) -> &Arc<Scope> {
        self.scope_stack.last().unwrap()
    }

    fn end_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn call_function(
        &mut self,
        value: &Value,
        args: Vec<Value>,
        name: Option<String>,
        span: Option<Span>,
    ) -> Result<Value, InterpreterError> {
        match value {
            Value::Function(Function {
                body,
                args: params,
                scope,
                variadic,
                ..
            }) => match body {
                FunctionBody::Ast(ast) => {
                    if args.len() != params.len() && !variadic {
                        return Err(InterpreterError::ArgumentError(
                            format!(
                                "{} takes {} {} but {} {} given",
                                value,
                                params.len(),
                                if params.len() == 1 {
                                    "argument"
                                } else {
                                    "arguments"
                                },
                                args.len(),
                                if args.len() == 1 { "was" } else { "were" },
                            ),
                            span.unwrap(),
                        ));
                    }

                    self.scope_stack.push(scope.clone());
                    self.begin_scope();

                    let scope = self.current_scope();

                    // TODO: if arg is variadic and arg count > param count, collect them into a list and pass as the last argument

                    for (arg, param) in args.into_iter().zip(params.iter().map(|p| p.0.clone())) {
                        scope.define(param, arg)?;
                    }

                    let value = self.interpret_internal(ast.as_ref());
                    self.end_scope();
                    self.end_scope();
                    value
                }
                FunctionBody::Native(closure) => closure(args),
            },
            _ => Err(InterpreterError::TypeError(
                format!(
                    "`{}` is not a function",
                    name.unwrap_or(
                        span.clone()
                            .expect("span cannot be None")
                            .content()
                            .to_owned()
                    )
                ),
                span.expect("span cannot be None"),
            )),
        }
    }

    fn unpack(
        &mut self,
        pattern: &AstNode,
        value: Value,
        value_span: Span,
    ) -> Result<(), InterpreterError> {
        // TODO: more complex patterns

        match &**pattern {
            Ast::VariableAccess(name) => self.current_scope().define(name, value),
            _ => Err(InterpreterError::PatternError(
                "I dunno what to do with that pattern tbh".to_string(),
                pattern.span(),
                value_span.clone(),
            )),
        }
    }

    fn interpret_internal(&mut self, ast: &AstNode) -> Result<Value, InterpreterError> {
        match &**ast {
            Ast::Program(nodes) => {
                for node in nodes {
                    self.interpret_internal(node)?;
                }

                if let Some(main) = self.current_scope().get("main").cloned() {
                    self.call_function(&main, vec![], Some("main".to_string()), None)
                } else {
                    Ok(Value::None)
                }
            }

            Ast::Let {
                pattern,
                type_: _,
                body,
            } => {
                // todo: check types

                let value = self.interpret_internal(body)?;

                self.unpack(pattern, value, body.span())?;

                Ok(Value::None)
            }
            Ast::Static {
                public: _,
                name,
                type_: _,
                body,
            } => {
                // TODO: public
                // TODO: types

                let value = self.interpret_internal(body)?;

                self.current_scope().define(&name.value, value)?;

                Ok(Value::None)
            }

            Ast::Function {
                public: _,
                name,
                args,
                return_type: _,
                body,
                variadic,
            } => {
                let scope = self.current_scope();

                scope.define(
                    &name.value,
                    Value::Function(Function {
                        name: name.value.clone(),
                        args: args
                            .iter()
                            .map(|(name, _type)| (name.clone(), ()))
                            .collect(),
                        return_type: (),
                        body: FunctionBody::Ast(Arc::new(body.clone())),
                        span: ast.span(),
                        scope: scope.clone(),
                        variadic: *variadic,
                    }),
                )?;

                Ok(Value::None)
            }
            Ast::Call(fn_access, args) => {
                let function = self.interpret_internal(fn_access)?;

                let mut evaluated_args = vec![];
                let mut span = fn_access.span();

                for arg in args {
                    span.fit(&arg.span());
                    evaluated_args.push(self.interpret_internal(arg)?);
                }

                self.call_function(&function, evaluated_args, None, Some(span))
            }
            Ast::Block {
                statements,
                return_value,
            } => {
                for statement in statements {
                    self.interpret_internal(statement)?;
                }

                if let Some(return_value) = return_value {
                    self.interpret_internal(return_value)
                } else {
                    Ok(Value::None)
                }
            }
            Ast::Import {
                names,
                module: module_name,
                module_alias,
            } => {
                let resolver = ModuleLabel::parse(module_name.clone())?;
                let module = resolver.resolve(self.module_pool.clone(), self.label.clone())?;

                for (import, alias) in names {
                    let value = module.get(&import.value);

                    if value.is_none() {
                        return Err(InterpreterError::UndefinedVariable(
                            format!(
                                "Cannot import {:?} from module {:?} as it is not defined",
                                &import.value, &module_name.value
                            ),
                            import.span.clone(),
                        ));
                    }

                    self.current_scope().define(
                        alias.as_ref().unwrap_or(import).value.clone(),
                        value.unwrap().to_owned(),
                    )?;
                }

                let label = self.label.clone();

                if let Some(module_alias) = module_alias {
                    self.current_scope().define(
                        &module_alias.value,
                        Value::Module(Module {
                            name: resolver.absolute(label),
                            scope: module,
                        }),
                    )?;
                }

                Ok(Value::None)
            }

            Ast::VariableAccess(name) => {
                if let Some(value) = self.current_scope().get(name) {
                    Ok(value.to_owned())
                } else {
                    Err(InterpreterError::UndefinedVariable(
                        name.clone(),
                        ast.span(),
                    ))
                }
            }
            Ast::MemberAccess(mapping, name) => {
                let mapping_object = self.interpret_internal(mapping)?;

                match mapping_object {
                    Value::Instance(_) => {
                        todo!();
                    }
                    Value::Class(_) => {
                        todo!("accessing class members");
                    }
                    Value::Module(module) => {
                        if let Some(value) = module.scope.get(&name.value) {
                            Ok(value.to_owned())
                        } else {
                            Err(InterpreterError::UndefinedVariable(
                                format!(
                                    "Cannot access member variable {:?} of {:?} as it does not exist",
                                    &name.value,
                                    Value::Module(module)
                                ),
                                name.span.clone(),
                            ))
                        }
                    }

                    value => Err(InterpreterError::TypeError(
                        format!(
                            "Cannot access field {:?} on {:?}",
                            name.value.clone(),
                            value
                        ),
                        name.span.clone(),
                    )),
                }
            }

            // Literals
            Ast::String(value) => Ok(Value::String(value.clone())),
            Ast::Integer(value) => Ok(Value::Integer((*value).try_into().unwrap())),

            node => todo!("{:?}", node),
        }
    }
}
