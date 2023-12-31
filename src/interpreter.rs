use crate::expr::{Expr, Value};
use crate::primitive::{Callable, Class, Instance, LoxCallable, Primitive};
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use core::fmt::Display;
use environment::Environment;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

pub mod environment;

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
    pub locals: HashMap<Expr, usize>,
}

#[derive(Debug)]
pub struct InterpretError {
    pub message: String,
    pub token: Token,
    pub value: Option<Value>,
}

impl InterpretError {
    pub fn new(message: String, token: Token) -> Self {
        Self {
            message,
            token,
            value: None,
        }
    }

    fn with_value(message: String, token: Token, value: Value) -> Self {
        Self {
            message,
            token,
            value: Some(value),
        }
    }
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Interpreter {
    pub fn new(environment: Rc<RefCell<Environment>>) -> Self {
        Self {
            environment,
            locals: HashMap::new(),
        }
    }

    pub fn new_with_locals(
        environment: Rc<RefCell<Environment>>,
        locals: HashMap<Expr, usize>,
    ) -> Self {
        Self {
            environment,
            locals,
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.environment.borrow_mut().define(name, value);
    }

    pub fn get_local(&mut self, expr: &Expr) -> Option<usize> {
        let distance = self.locals.get(expr);
        if let Some(distance) = distance {
            return Some(*distance);
        } else {
            None
        }
    }

    fn assign(&mut self, token: Token, value: Value) -> Result<(), InterpretError> {
        self.environment.borrow_mut().assign(token.lexeme, value)
    }

    pub fn new_environment(&mut self) {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(Environment::new(previous)));
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr, depth);
    }

    fn look_up_var(&self, name: &Token, expr: &Expr) -> Result<Value, InterpretError> {
        let distance = self.locals.get(expr);
        if let Some(distance) = distance {
            self.environment
                .borrow()
                .get(*distance, name.lexeme.as_str())
        } else {
            self.environment.borrow().get_global(name.lexeme.as_str())
        }
        .ok_or_else(|| {
            InterpretError::new(
                format!("Undefined variable '{}'.", name.lexeme),
                name.clone(),
            )
        })
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
            Primitive::Callable(_)
            | Primitive::String(_)
            | Primitive::Nil
            | Primitive::Boolean(_)
            | Primitive::Instance(_)
            | Primitive::Class(_) => Err(InterpretError::new(
                format!("Expected number, got {}", value.primitive),
                value.token,
            )),
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

impl Interpreter {
    pub fn interpret(&mut self, stmt: Stmt) -> Result<(), InterpretError> {
        match stmt {
            Stmt::Return(token, expr) => {
                if let Some(expr) = expr {
                    let value = self.interpret_expr(expr)?;
                    return Err(InterpretError::with_value(
                        "Successful return".to_string(),
                        token,
                        value,
                    ));
                }
                Err(InterpretError::with_value(
                    "Successful return".to_string(),
                    token.clone(),
                    Value {
                        primitive: Primitive::Nil,
                        token: Token {
                            token_type: TokenType::NIL,
                            lexeme: "nil".to_string(),
                            line: token.line,
                        },
                    },
                ))
            }
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
                self.define(token.lexeme, value);
                Ok(())
            }
            Stmt::Assign(token, expr) => {
                let value = self.interpret_expr(expr)?;
                self.assign(token, value)
            }
            Stmt::Block(stmts) => {
                let previous = self.environment.clone();
                self.new_environment();
                match self.interpret_block(stmts) {
                    Ok(_) => {}
                    Err(err) => {
                        self.environment = previous;
                        return Err(err);
                    }
                };
                self.environment = previous;
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
            Stmt::Function(token, parameters, body) => {
                let callable =
                    Callable::new(token.clone(), parameters, body, self.environment.clone());
                let value = Value {
                    primitive: Primitive::Callable(callable),
                    token: token.clone(),
                };
                self.define(token.lexeme, value);
                Ok(())
            }
            Stmt::Class(name, methods) => {
                let class = Class::new(name.clone(), methods);
                let value = Value {
                    primitive: Primitive::Class(class.clone()),
                    token: name,
                };
                self.define(class.name.lexeme, value);
                Ok(())
            }
            Stmt::Break => todo!(),
        }
    }

    pub fn interpret_block(&mut self, stmts: Vec<Stmt>) -> Result<(), InterpretError> {
        for stmt in stmts {
            match self.interpret(stmt) {
                Ok(_) => {}
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub fn interpret_expr(&mut self, expr: Expr) -> Result<Value, InterpretError> {
        match expr.clone() {
            Expr::Get(get_expr) => {
                let object = self.interpret_expr(*get_expr.expr)?;
                println!("Object we're getting: {:?}", object);
                match object.primitive {
                    Primitive::Instance(instance) => instance.get(get_expr.name.clone()),
                    _ => Err(InterpretError::new(
                        "Only instances have properties.".to_string(),
                        get_expr.name,
                    )),
                }
            }
            Expr::Set(set_expr) => {
                let object = self.interpret_expr(*set_expr.expr)?;
                match object.primitive {
                    Primitive::Instance(mut instance) => {
                        let value = self.interpret_expr(*set_expr.value)?;
                        println!("Instace fields before: {:?}", instance.fields);
                        instance.set(set_expr.name.clone(), value.clone());
                        println!("Instance fields after: {:?}", instance.fields);
                        return Ok(value);
                    }
                    _ => Err(InterpretError::new(
                        "Only instances have fields.".to_string(),
                        set_expr.name,
                    )),
                }
            }
            Expr::Call(call) => {
                let callee = self.interpret_expr(*call.callee)?;
                let mut arguments = Vec::new();
                for argument in call.arguments {
                    arguments.push(self.interpret_expr(argument)?);
                }
                match callee.primitive {
                    Primitive::Callable(callable) => {
                        if arguments.len() != callable.arity {
                            return Err(InterpretError::new(
                                format!(
                                    "Expected {} arguments but got {}.",
                                    callable.arity,
                                    arguments.len()
                                ),
                                call.paren,
                            ));
                        }
                        callable.call(arguments, self.locals.clone())
                    }
                    Primitive::Class(class) => {
                        if arguments.len() != 0 {
                            return Err(InterpretError::new(
                                format!("Expected 0 arguments but got {}.", arguments.len()),
                                call.paren,
                            ));
                        }
                        Ok(Value {
                            primitive: Primitive::Instance(Instance::new(class)),
                            token: call.paren,
                        })
                    }
                    _ => Err(InterpretError::new(
                        "Can only call functions and classes.".to_string(),
                        call.paren,
                    )),
                }
            }
            Expr::Binary(binary) => {
                let left = self.interpret_expr(*binary.left)?;
                let right = self.interpret_expr(*binary.right)?;
                match binary.operator.lexeme.as_str() {
                    "-" => {
                        if let (Primitive::Number(left), Primitive::Number(right)) =
                            (&left.primitive, &right.primitive)
                        {
                            Ok(Value {
                                primitive: Primitive::Number(left - right),
                                token: binary.operator,
                            })
                        } else {
                            Err(InterpretError::new(
                                format!(
                                    "Operands must be two numbers: {} - {}",
                                    left.token.lexeme, right.token.lexeme
                                ),
                                binary.operator,
                            ))
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
                            Err(InterpretError::new(
                                format!(
                                    "Operands must be two numbers: {} * {}",
                                    left.token.lexeme, right.token.lexeme
                                ),
                                binary.operator,
                            ))
                        }
                    }
                    "/" => {
                        if let (Primitive::Number(left), Primitive::Number(right)) =
                            (&left.primitive, &right.primitive)
                        {
                            if right == &0.0 {
                                Err(InterpretError::new(
                                    "Division by zero.".to_string(),
                                    binary.operator,
                                ))
                            } else {
                                Ok(Value {
                                    primitive: Primitive::Number(left / right),
                                    token: binary.operator,
                                })
                            }
                        } else {
                            Err(InterpretError::new(
                                format!(
                                    "Operands must be two numbers: {} / {}",
                                    left.token.lexeme, right.token.lexeme
                                ),
                                binary.operator,
                            ))
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
                        _ => Err(InterpretError::new(
                            format!(
                                "Operands must be two numbers or two strings: {} + {}",
                                left.token.lexeme, right.token.lexeme
                            ),
                            binary.operator,
                        )),
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
                    _ => Err(InterpretError::new(
                        format!(
                            "Operands must be two numbers or two strings: {} + {}",
                            left.token.lexeme, right.token.lexeme
                        ),
                        binary.operator,
                    )),
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
                _ => Err(InterpretError::new(
                    format!("Unknown literal: {}", literal.value.lexeme),
                    literal.value,
                )),
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
                    _ => Err(InterpretError::new(
                        format!("Unknown unary operator: {}", unary.operator.lexeme),
                        unary.operator,
                    )),
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
            Expr::Variable(variable) => Ok(self.look_up_var(&variable.name, &expr)?),
            Expr::Assign(assign) => {
                let distance = self.get_local(&expr);
                if let Some(distance) = distance.clone() {
                    let expr = self.interpret_expr(*assign.value.clone())?;
                    self.environment.borrow_mut().assign_at(
                        distance,
                        assign.name.lexeme.clone(),
                        expr,
                    );
                } else {
                    let expr = self.interpret_expr(*assign.value.clone())?;
                    self.environment
                        .borrow_mut()
                        .assign_global(assign.name.lexeme.clone(), expr);
                }
                Ok(self.interpret_expr(*assign.value)?)
            }
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
            }
        }
    }
}
