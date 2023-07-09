use crate::expr::Expr;
use crate::token::{Token, TokenType};
use core::fmt::Display;

pub fn interpret(expr: Expr) -> Result<Primitive, InterpretError> {
    match expr {
        Expr::Binary(binary) => {
            let left = interpret(*binary.left)?;
            let right = interpret(*binary.right)?;
            match binary.operator.lexeme.as_str() {
                "-" => {
                    if let (
                        Primitive::Number {
                            value: left,
                            token: _,
                        },
                        Primitive::Number {
                            value: right,
                            token: _,
                        },
                    ) = (&left, &right)
                    {
                        Ok(Primitive::Number {
                            value: left - right,
                            token: binary.operator,
                        })
                    } else {
                        Err(InterpretError {
                            message: format!("Operands must be two numbers: {} - {}", left, right),
                            token: binary.operator,
                        })
                    }
                }
                "*" => {
                    if let (
                        Primitive::Number {
                            value: left,
                            token: _,
                        },
                        Primitive::Number {
                            value: right,
                            token: _,
                        },
                    ) = (&left, &right)
                    {
                        Ok(Primitive::Number {
                            value: left * right,
                            token: binary.operator,
                        })
                    } else {
                        Err(InterpretError {
                            message: format!("Operands must be two numbers: {} * {}", left, right),
                            token: binary.operator,
                        })
                    }
                }
                "/" => {
                    if let (
                        Primitive::Number {
                            value: left,
                            token: _,
                        },
                        Primitive::Number {
                            value: right,
                            token: _,
                        },
                    ) = (&left, &right)
                    {
                        if right == &0.0 {
                            Err(InterpretError {
                                message: "Division by zero".to_string(),
                                token: binary.operator,
                            })
                        } else {
                            Ok(Primitive::Number {
                                value: left / right,
                                token: binary.operator,
                            })
                        }
                    } else {
                        Err(InterpretError {
                            message: format!("Operands must be two numbers: {} / {}", left, right),
                            token: binary.operator,
                        })
                    }
                }
                "+" => match (&left, &right) {
                    (
                        Primitive::Number {
                            value: left,
                            token: _,
                        },
                        Primitive::Number {
                            value: right,
                            token: _,
                        },
                    ) => Ok(Primitive::Number {
                        value: left + right,
                        token: binary.operator,
                    }),
                    (
                        Primitive::String {
                            value: left,
                            token: _,
                        },
                        _,
                    ) => Ok(Primitive::String {
                        value: format!("{}{}", left, right),
                        token: binary.operator,
                    }),
                    (
                        _,
                        Primitive::String {
                            value: right,
                            token: _,
                        },
                    ) => Ok(Primitive::String {
                        value: format!("{}{}", left, right),
                        token: binary.operator,
                    }),
                    _ => Err(InterpretError {
                        message: format!(
                            "Operands must be two numbers or two strings: {} + {}",
                            left, right
                        ),
                        token: binary.operator,
                    }),
                },
                ">" => Ok(Primitive::Boolean {
                    value: (to_number(left)? > to_number(right)?),
                    token: binary.operator,
                }),
                ">=" => Ok(Primitive::Boolean {
                    value: (to_number(left)? >= to_number(right)?),
                    token: binary.operator,
                }),
                "<" => Ok(Primitive::Boolean {
                    value: (to_number(left)? < to_number(right)?),
                    token: binary.operator,
                }),
                "<=" => Ok(Primitive::Boolean {
                    value: (to_number(left)? <= to_number(right)?),
                    token: binary.operator,
                }),
                "!=" => Ok(Primitive::Boolean {
                    value: (to_number(left)? != to_number(right)?),
                    token: binary.operator,
                }),
                "==" => Ok(Primitive::Boolean {
                    value: (to_number(left)? == to_number(right)?),
                    token: binary.operator,
                }),
                _ => Err(InterpretError {
                    message: format!("Unknown operator: {}", binary.operator.lexeme),
                    token: binary.operator,
                }),
            }
        }
        Expr::Grouping(grouping) => Ok(interpret(*grouping.expression)?),
        Expr::Literal(literal) => match literal.value.token_type {
            TokenType::FALSE => Ok(Primitive::Boolean {
                value: false,
                token: literal.value,
            }),
            TokenType::TRUE => Ok(Primitive::Boolean {
                value: true,
                token: literal.value,
            }),
            TokenType::NIL => Ok(Primitive::Nil {
                token: literal.value,
            }),
            TokenType::NUMBER => Ok(Primitive::Number {
                value: literal.value.lexeme.parse::<f64>().unwrap(),
                token: literal.value,
            }),
            TokenType::STRING => Ok(Primitive::String {
                value: literal.value.lexeme.clone(),
                token: literal.value,
            }),
            _ => Err(InterpretError {
                message: format!("Unknown literal: {}", literal.value.lexeme),
                token: literal.value,
            }),
        },
        Expr::Unary(unary) => {
            let right = interpret(*unary.right)?;
            match unary.operator.lexeme.as_str() {
                "!" => Ok(Primitive::Boolean {
                    value: !is_truthy(right),
                    token: unary.operator,
                }),
                "-" => Ok(Primitive::Number {
                    value: -to_number(right)?,
                    token: unary.operator,
                }),
                _ => Err(InterpretError {
                    message: format!("Unknown operator: {}", unary.operator.lexeme),
                    token: unary.operator,
                }),
            }
        }
        Expr::Ternary(ternary) => {
            let condition = interpret(*ternary.condition)?;
            if is_truthy(condition) {
                Ok(interpret(*ternary.then_branch)?)
            } else {
                Ok(interpret(*ternary.else_branch)?)
            }
        }
    }
}

fn is_truthy(primitive: Primitive) -> bool {
    match primitive {
        Primitive::Nil { token: _ } => false,
        Primitive::Boolean {
            value: boolean,
            token: _,
        } => boolean,
        _ => true,
    }
}

fn to_number(primitive: Primitive) -> Result<f64, InterpretError> {
    match primitive {
        Primitive::Number {
            value: number,
            token: _,
        } => Ok(number),
        Primitive::String { value: _, token }
        | Primitive::Nil { token }
        | Primitive::Boolean { value: _, token } => Err(InterpretError {
            message: format!("Operand must be a number: {}", token.lexeme),
            token,
        }),
    }
}

#[allow(dead_code)]
fn is_equal(left: Primitive, right: Primitive) -> bool {
    match (left, right) {
        (Primitive::Nil { token: _ }, Primitive::Nil { token: _ }) => true,
        (
            Primitive::Boolean {
                value: left,
                token: _,
            },
            Primitive::Boolean {
                value: right,
                token: _,
            },
        ) => left == right,
        (
            Primitive::Number {
                value: left,
                token: _,
            },
            Primitive::Number {
                value: right,
                token: _,
            },
        ) => left == right,
        (
            Primitive::String {
                value: left,
                token: _,
            },
            Primitive::String {
                value: right,
                token: _,
            },
        ) => left == right,
        _ => false,
    }
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

#[derive(PartialEq)]
pub enum Primitive {
    Number { value: f64, token: Token },
    Boolean { value: bool, token: Token },
    Nil { token: Token },
    String { value: String, token: Token },
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Number {
                value: number,
                token: _,
            } => write!(f, "{}", number),
            Primitive::Boolean {
                value: boolean,
                token: _,
            } => write!(f, "{}", boolean),
            Primitive::Nil { token: _ } => write!(f, "nil"),
            Primitive::String {
                value: string,
                token: _,
            } => write!(f, "\"{}\"", string),
        }
    }
}
