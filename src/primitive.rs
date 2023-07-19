use crate::{
    expr::{Expr, Value},
    interpreter::{environment::Environment, InterpretError, Interpreter},
    stmt::Stmt,
    token::{Token, TokenType},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
    Callable(Callable),
    Class(Class),
    Instance(Instance),
}

pub trait LoxCallable {
    fn call(&self, args: Vec<Value>, locals: HashMap<Expr, usize>)
        -> Result<Value, InterpretError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub name: Token,
    pub methods: Vec<Stmt>,
}

impl Class {
    pub fn new(name: Token, methods: Vec<Stmt>) -> Self {
        Self { name, methods }
    }
}

impl LoxCallable for Class {
    fn call(
        &self,
        args: Vec<Value>,
        locals: HashMap<Expr, usize>,
    ) -> Result<Value, InterpretError> {
        Ok(Value {
            primitive: Primitive::Instance(Instance::new(self.clone())),
            token: self.name.clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    class: Class,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: Token) -> Result<Value, InterpretError> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone());
        }
        Err(InterpretError::new(
            format!("Undefined property '{}'.", name.lexeme),
            name,
        ))
    }

    pub fn set(&mut self, name: Token, value: Value) {
        self.fields.insert(name.lexeme, value);
    }
}

impl PartialEq for Callable {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn>")
    }
}

#[derive(Clone)]
pub struct Callable {
    pub arity: usize,
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

impl Callable {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        let callable = Self {
            arity: params.len(),
            name: name.clone(),
            params,
            body,
            closure,
        };
        callable
    }
}

impl LoxCallable for Callable {
    fn call(
        &self,
        args: Vec<Value>,
        locals: HashMap<Expr, usize>,
    ) -> Result<Value, InterpretError> {
        let mut new_interpreter = Interpreter::new_with_locals(self.closure.clone(), locals);
        new_interpreter.new_environment();
        for (i, arg) in args.iter().enumerate() {
            new_interpreter.define(self.params[i].lexeme.clone(), arg.clone());
        }
        match new_interpreter.interpret_block(self.body.clone()) {
            Ok(_) => Ok(Value {
                primitive: Primitive::Nil,
                token: Token::new(TokenType::NIL, String::from("nil"), 0),
            }),
            Err(e) => {
                if let Some(value) = e.value {
                    Ok(value)
                } else {
                    Err(e)
                }
            }
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Number(number) => write!(f, "{}", number),
            Primitive::Boolean(boolean) => write!(f, "{}", boolean),
            Primitive::Nil => write!(f, "nil"),
            Primitive::String(string) => write!(f, "\"{}\"", string),
            Primitive::Callable(callable) => write!(
                f,
                "<fn> {}({})",
                callable.name.lexeme,
                callable
                    .params
                    .iter()
                    .map(|param| param.lexeme.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Primitive::Class(class) => write!(f, "{}", class.name.lexeme),
            Primitive::Instance(instance) => write!(f, "{} instance", instance.class.name.lexeme),
        }
    }
}
