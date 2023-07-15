use std::fmt::{Debug, Display};

use crate::{token::{Token, TokenType}, interpreter::{InterpretError, Interpreter}, stmt::Stmt};

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
}

impl Callable {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Self {
        Self {
            arity: params.len(),
            name,
            params,
            body,
        }
    }

    pub fn call(&self, args: Vec<Value>) -> Result<Value, InterpretError> {
        let mut interpreter = Interpreter::new();
        for (i, arg) in args.iter().enumerate() {
            interpreter
                .environment
                .define(self.params[i].lexeme.clone(), arg.clone());
        }
        match interpreter.interpret_block(self.body.clone()) {
            Ok(_) => Ok(Value {
                primitive: Primitive::Nil,
                token: Token::new(TokenType::NIL, String::from("nil"), 0),
            }),
            Err(e) => Err(e),
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
            Primitive::Callable(_) => write!(f, "<fn>"),
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
