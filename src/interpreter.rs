use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use core::fmt::Display;
use std::fmt::Debug;
use environment::Environment;

mod environment;

pub struct Interpreter {
    environment: Environment,
    globals: Environment,
}

#[derive(Debug)]
pub struct InterpretError {
    pub message: String,
    pub token: Token,
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
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
struct Callable {
    arity: usize,
    call: fn(&mut Interpreter, Vec<Value>) -> Result<Value, InterpretError>,
}

impl Callable {
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, InterpretError> {
        (self.call)(interpreter, args)
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

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::global();
        Self {
            environment: Environment::new(Box::new(globals.clone())),
            globals,
        }
    }

    pub fn interpret(&mut self, stmt: Stmt) -> Result<(), InterpretError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.interpret_expr(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.interpret_expr(expr)?;
                println!("{}", value.primitive);
                Ok(())
            }
            Stmt::Var(token, initializer) => {
                let value = match initializer {
                    Some(expr) => self.interpret_expr(expr)?,
                    None => Value {
                        primitive: Primitive::Nil,
                        token: Token {
                            token_type: TokenType::NIL,
                            lexeme: "nil".to_string(),
                            line: token.line,
                        },
                    },
                };
                self.environment.define(token.lexeme, value);
                Ok(())
            }
            Stmt::Assign(token, expr) => {
                if !self.environment.contains_key(token.lexeme.as_str()) {
                    return Err(InterpretError {
                        message: format!("Undefined variable '{}'.", token.lexeme),
                        token,
                    });
                }
                let value = self.interpret_expr(expr)?;
                self.environment.assign(token.lexeme, value)?;
                Ok(())
            }
            Stmt::Block(stmts) => {
                let environment = Environment::new(Box::new(self.environment.clone()));
                self.environment = environment;
                for stmt in stmts {
                    match self.interpret(stmt) {
                        Ok(_) => {}
                        Err(error) => {
                            self.environment.clear_child();
                            return Err(error);
                        }
                    }
                }
                self.environment.clear_child();
                Ok(())
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let condition = self.interpret_expr(condition)?;
                if condition.primitive == Primitive::Boolean(true) {
                    self.interpret(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.interpret(*else_branch)?;
                }
                Ok(())
            }
            Stmt::While(condition, body) => {
                while self.interpret_expr(condition.clone())?.primitive == Primitive::Boolean(true)
                {
                    self.interpret(*body.clone())?;
                }
                Ok(())
            }
            Stmt::Break => Err(InterpretError {
                message: "Break outside of loop".to_string(),
                token: Token {
                    token_type: TokenType::BREAK,
                    lexeme: "break".to_string(),
                    line: 0,
                },
            }),
        }
    }

    fn interpret_expr(&mut self, expr: Expr) -> Result<Value, InterpretError> {
        match expr {
            Expr::Call(call) => {
                let callee = self.interpret_expr(*call.callee)?;
                let mut arguments = Vec::new();
                for argument in call.arguments {
                    arguments.push(self.interpret_expr(argument)?);
                }
                match callee.primitive {
                    Primitive::Callable(callable) => {
                        if arguments.len() != callable.arity {
                            return Err(InterpretError {
                                message: format!(
                                    "Expected {} arguments but got {}.",
                                    callable.arity,
                                    arguments.len()
                                ),
                                token: call.paren,
                            });
                        }
                        callable.call(self, arguments)
                    }
                    _ => Err(InterpretError {
                        message: "Can only call functions and classes.".to_string(),
                        token: call.paren,
                    }),
                }
            }
            Expr::Binary(binary) => {
                let left = self.interpret_expr(*binary.left)?;
                let right = self.interpret_expr(*binary.right)?;
                match binary.operator.lexeme.as_str() {
                    "-" => {
                        if let (Primitive::Number(right), Primitive::Number(left)) =
                            (&left.primitive, &right.primitive)
                        {
                            Ok(Value {
                                primitive: Primitive::Number(left - right),
                                token: binary.operator,
                            })
                        } else {
                            Err(InterpretError {
                                message: format!(
                                    "Operands must be two numbers: {} - {}",
                                    left.token.lexeme, right.token.lexeme
                                ),
                                token: binary.operator,
                            })
                        }
                    }
                    "*" => {
                        if let (Primitive::Number(left), Primitive::Number(right)) =
                            (&left.primitive, &right.primitive)
                        {
                            Ok(Value {
                                primitive: Primitive::Number(left * right),
                                token: binary.operator,
                            })
                        } else {
                            Err(InterpretError {
                                message: format!(
                                    "Operands must be two numbers: {} * {}",
                                    left.token.lexeme, right.token.lexeme
                                ),
                                token: binary.operator,
                            })
                        }
                    }
                    "/" => {
                        if let (Primitive::Number(left), Primitive::Number(right)) =
                            (&left.primitive, &right.primitive)
                        {
                            if right == &0.0 {
                                Err(InterpretError {
                                    message: "Division by zero".to_string(),
                                    token: binary.operator,
                                })
                            } else {
                                Ok(Value {
                                    primitive: Primitive::Number(left / right),
                                    token: binary.operator,
                                })
                            }
                        } else {
                            Err(InterpretError {
                                message: format!(
                                    "Operands must be two numbers: {} / {}",
                                    left.token.lexeme, right.token.lexeme
                                ),
                                token: binary.operator,
                            })
                        }
                    }
                    "+" => match (&left.primitive, &right.primitive) {
                        (Primitive::Number(left), Primitive::Number(right)) => Ok(Value {
                            primitive: Primitive::Number(left + right),
                            token: binary.operator,
                        }),
                        (Primitive::String(left), Primitive::String(right)) => Ok(Value {
                            primitive: Primitive::String(format!("{}{}", left, right)),
                            token: binary.operator,
                        }),
                        (Primitive::String(left), Primitive::Number(right)) => Ok(Value {
                            primitive: Primitive::String(format!("{}{}", left, right)),
                            token: binary.operator,
                        }),
                        (Primitive::Number(left), Primitive::String(right)) => Ok(Value {
                            primitive: Primitive::String(format!("{}{}", left, right)),
                            token: binary.operator,
                        }),
                        _ => Err(InterpretError {
                            message: format!(
                                "Operands must be two numbers or two strings: {} + {}",
                                left.token.lexeme, right.token.lexeme
                            ),
                            token: binary.operator,
                        }),
                    },
                    ">" => Ok(Value {
                        primitive: Primitive::Boolean(
                            self.to_number(left)? > self.to_number(right)?,
                        ),
                        token: binary.operator,
                    }),
                    ">=" => Ok(Value {
                        primitive: Primitive::Boolean(
                            self.to_number(left)? >= self.to_number(right)?,
                        ),
                        token: binary.operator,
                    }),
                    "<" => Ok(Value {
                        primitive: Primitive::Boolean(
                            self.to_number(left)? < self.to_number(right)?,
                        ),
                        token: binary.operator,
                    }),
                    "<=" => Ok(Value {
                        primitive: Primitive::Boolean(
                            self.to_number(left)? <= self.to_number(right)?,
                        ),
                        token: binary.operator,
                    }),
                    "!=" => Ok(Value {
                        primitive: Primitive::Boolean(!self.is_equal(left, right)),
                        token: binary.operator,
                    }),
                    "==" => Ok(Value {
                        primitive: Primitive::Boolean(self.is_equal(left, right)),
                        token: binary.operator,
                    }),
                    _ => Err(InterpretError {
                        message: format!("Unknown operator: {}", binary.operator.lexeme),
                        token: binary.operator,
                    }),
                }
            }
            Expr::Grouping(grouping) => Ok(self.interpret_expr(*grouping.expression)?),
            Expr::Literal(literal) => match literal.value.token_type {
                TokenType::FALSE => Ok(Value {
                    primitive: Primitive::Boolean(false),
                    token: literal.value,
                }),
                TokenType::TRUE => Ok(Value {
                    primitive: Primitive::Boolean(true),
                    token: literal.value,
                }),
                TokenType::NIL => Ok(Value {
                    primitive: Primitive::Nil,
                    token: literal.value,
                }),
                TokenType::NUMBER => Ok(Value {
                    primitive: Primitive::Number(literal.value.lexeme.parse().unwrap()),
                    token: literal.value,
                }),
                TokenType::STRING => Ok(Value {
                    primitive: Primitive::String(literal.value.lexeme.clone()),
                    token: literal.value,
                }),
                _ => Err(InterpretError {
                    message: format!("Unknown literal: {}", literal.value.lexeme),
                    token: literal.value,
                }),
            },
            Expr::Unary(unary) => {
                let right = self.interpret_expr(*unary.right)?;
                match unary.operator.lexeme.as_str() {
                    "!" => Ok(Value {
                        primitive: Primitive::Boolean(!self.is_truthy(&right)),
                        token: unary.operator,
                    }),
                    "-" => Ok(Value {
                        primitive: Primitive::Number(-self.to_number(right)?),
                        token: unary.operator,
                    }),
                    _ => Err(InterpretError {
                        message: format!("Unknown operator: {}", unary.operator.lexeme),
                        token: unary.operator,
                    }),
                }
            }
            Expr::Ternary(ternary) => {
                let condition = self.interpret_expr(*ternary.condition)?;
                if self.is_truthy(&condition) {
                    Ok(self.interpret_expr(*ternary.then_branch)?)
                } else {
                    Ok(self.interpret_expr(*ternary.else_branch)?)
                }
            }
            Expr::Variable(variable) => {
                let value = self.environment.get(&variable.name.lexeme);
                match value {
                    Some(value) => Ok(value.clone()),
                    None => Err(InterpretError {
                        message: format!("Undefined variable: {}", variable.name.lexeme),
                        token: variable.name,
                    }),
                }
            }
            Expr::Assign(assign) => Ok(self.interpret_expr(*assign.value)?),
            Expr::Logical(logical) => {
                let left = self.interpret_expr(*logical.left)?;
                if logical.operator.token_type == TokenType::OR {
                    if self.is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !self.is_truthy(&left) {
                        return Ok(left);
                    }
                }
                self.interpret_expr(*logical.right)
            },
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value.primitive {
            Primitive::Nil => false,
            Primitive::Boolean(bool) => bool,
            _ => true,
        }
    }

    fn to_number(&self, value: Value) -> Result<f64, InterpretError> {
        match value.primitive {
            Primitive::Number(number) => Ok(number),
            Primitive::String(_) | Primitive::Nil | Primitive::Boolean(_) => Err(InterpretError {
                message: format!("Operand must be a number: {}", value.token.lexeme),
                token: value.token,
            }),
        }
    }

    fn is_equal(&self, left: Value, right: Value) -> bool {
        match (left.primitive, right.primitive) {
            (Primitive::Nil, Primitive::Nil) => true,
            (Primitive::Boolean(left), Primitive::Boolean(right)) => left == right,
            (Primitive::Number(left), Primitive::Number(right)) => left == right,
            (Primitive::String(left), Primitive::String(right)) => left == right,
            _ => false,
        }
    }
}
