use std::{fmt::{Debug, Display}, rc::Rc, cell::RefCell};

use crate::{token::{Token, TokenType}, interpreter::{InterpretError, Interpreter, environment::Environment}, stmt::Stmt};

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Logical(Logical),
    Ternary(Ternary),
    Variable(Variable),
    Assign(Assignment),
    Call(Call),
}

// 1 + 2, 3 * 4, etc.
#[derive(Clone, Debug)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

// (expression)
#[derive(Clone, Debug)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

// true, false, nil, 1, 2, 3, etc.
#[derive(Clone, Debug)]
pub struct Literal {
    pub value: Token,
}

// -1, !true, etc.
#[derive(Clone, Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

// condition ? then_branch : else_branch
#[derive(Clone, Debug)]
pub struct Ternary {
    pub condition: Box<Expr>,
    pub then_branch: Box<Expr>,
    pub else_branch: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: Token,
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}


#[derive(Clone, Debug)]
pub struct Value {
    pub primitive: Primitive,
    pub token: Token,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Primitive {
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
    Callable(Callable),
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
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>, closure: Rc<RefCell<Environment>>) -> Self {
        let callable = Self {
            arity: params.len(),
            name: name.clone(),
            params,
            body,
            closure: Rc::new((*closure).clone()),
        };
        callable.closure.borrow_mut().define(name.lexeme.clone(), Value {
            primitive: Primitive::Callable(callable.clone()),
            token: name.clone(),
        });
        callable
    }

    pub fn call(&mut self, args: Vec<Value>) -> Result<Value, InterpretError> {
        let mut new_interpreter = Interpreter::new(self.closure.clone());
        new_interpreter.new_environment();
        for (i, arg) in args.iter().enumerate() {
            new_interpreter.define(self.params[i].lexeme.clone(), arg.clone());
        }
        let value = match new_interpreter.interpret_block(self.body.clone()) {
            Ok(_) => {
                Ok(Value {
                primitive: Primitive::Nil,
                token: Token::new(TokenType::NIL, String::from("nil"), 0)
            })},
            Err(e) => {
                if let Some(value) = e.value {
                    Ok(value)
                } else {
                    Err(e)
                }
            },
        };
        value
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Number(number) => write!(f, "{}", number),
            Primitive::Boolean(boolean) => write!(f, "{}", boolean),
            Primitive::Nil => write!(f, "nil"),
            Primitive::String(string) => write!(f, "\"{}\"", string),
            Primitive::Callable(callable) => write!(f, "<fn> {}({})", callable.name.lexeme, callable.params.iter().map(|param| param.lexeme.clone()).collect::<Vec<String>>().join(", "))
        }
    }
}

#[allow(dead_code)]
pub fn print(expr: Expr) -> String {
    match expr {
        Expr::Binary(binary) => {
            format!(
                "({} {} {})",
                binary.operator.lexeme,
                print(*binary.left),
                print(*binary.right)
            )
        }
        Expr::Grouping(grouping) => {
            format!("(group {})", print(*grouping.expression))
        }
        Expr::Literal(literal) => literal.value.lexeme,
        Expr::Unary(unary) => {
            format!("({} {})", unary.operator.lexeme, print(*unary.right))
        }
        Expr::Ternary(ternary) => {
            format!(
                "({} ? {} : {})",
                print(*ternary.condition),
                print(*ternary.then_branch),
                print(*ternary.else_branch)
            )
        }
        Expr::Variable(variable) => variable.name.lexeme,
        Expr::Assign(assignment) => {
            format!(
                "(= {} {})",
                assignment.name.lexeme,
                print(*assignment.value)
            )
        }
        Expr::Logical(logical) => {
            format!(
                "({} {} {})",
                logical.operator.lexeme,
                print(*logical.left),
                print(*logical.right)
            )
        }
        Expr::Call(call) => {
            let mut args = String::new();
            for arg in call.arguments {
                args.push_str(&print(arg));
                args.push_str(", ");
            }
            format!(
                "(call {} {} {})",
                print(*call.callee),
                call.paren.lexeme,
                args
            )
        }
    }
}
