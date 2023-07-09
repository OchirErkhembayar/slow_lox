use crate::expr::Expr;
use crate::token::TokenType;
use core::fmt::Display;

pub fn interpret(expr: Expr) -> Result<Primitive, InterpretError> {
    match expr {
        Expr::Binary(binary) => {
            let left = interpret(*binary.left)?;
            let right = interpret(*binary.right)?;
            match binary.operator.lexeme.as_str() {
                "-" => Ok(Primitive::Number(to_number(left) - to_number(right))),
                "*" => Ok(Primitive::Number(to_number(left) * to_number(right))),
                "/" => Ok(Primitive::Number(to_number(left) / to_number(right))),
                "+" => match (&left, &right) {
                    (Primitive::Number(left), Primitive::Number(right)) => {
                        Ok(Primitive::Number(left + right))
                    }
                    (Primitive::String(left), Primitive::String(right)) => {
                        Ok(Primitive::String(format!("{}{}", left, right)))
                    }
                    _ => {
                        Err(InterpretError {
                            message: format!(
                                "Operands must be two numbers or two strings: {} + {}",
                                left, right
                            ),
                        })
                    }
                },
                ">" => Ok(Primitive::Boolean(to_number(left) > to_number(right))),
                ">=" => Ok(Primitive::Boolean(to_number(left) >= to_number(right))),
                "<" => Ok(Primitive::Boolean(to_number(left) < to_number(right))),
                "<=" => Ok(Primitive::Boolean(to_number(left) <= to_number(right))),
                "!=" => Ok(Primitive::Boolean(to_number(left) != to_number(right))),
                "==" => Ok(Primitive::Boolean(is_equal(left, right))),
                _ => {
                    Err(InterpretError {
                        message: format!("Unknown operator: {}", binary.operator.lexeme),
                    })
                }
            }
        }
        Expr::Grouping(grouping) => Ok(interpret(*grouping.expression)?),
        Expr::Literal(literal) => match literal.value.token_type {
            TokenType::FALSE => Ok(Primitive::Boolean(false)),
            TokenType::TRUE => Ok(Primitive::Boolean(true)),
            TokenType::NIL => Ok(Primitive::Nil),
            TokenType::NUMBER => Ok(Primitive::Number(
                literal.value.lexeme.parse::<f64>().unwrap(),
            )),
            TokenType::STRING => Ok(Primitive::String(literal.value.lexeme)),
            _ => Err(InterpretError {
                message: format!("Unknown literal: {}", literal.value.lexeme),
            }),
        },
        Expr::Unary(unary) => {
            let right = interpret(*unary.right)?;
            match unary.operator.lexeme.as_str() {
                "!" => Ok(Primitive::Boolean(!is_truthy(right))),
                "-" => Ok(Primitive::Number(-to_number(right))),
                _ => {
                    return Err(InterpretError {
                        message: format!("Unknown operator: {}", unary.operator.lexeme),
                    });
                }
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
        Primitive::Nil => false,
        Primitive::Boolean(boolean) => boolean,
        _ => true,
    }
}

fn to_number(primitive: Primitive) -> f64 {
    match primitive {
        Primitive::Number(number) => number,
        Primitive::Boolean(boolean) => {
            if boolean {
                1.0
            } else {
                0.0
            }
        }
        Primitive::Nil => 0.0,
        Primitive::String(string) => string.parse::<f64>().unwrap(),
    }
}

fn is_equal(left: Primitive, right: Primitive) -> bool {
    match (left, right) {
        (Primitive::Nil, Primitive::Nil) => true,
        (Primitive::Boolean(left), Primitive::Boolean(right)) => left == right,
        (Primitive::Number(left), Primitive::Number(right)) => left == right,
        (Primitive::String(left), Primitive::String(right)) => left == right,
        _ => false,
    }
}

#[derive(Debug)]
pub struct InterpretError {
    pub message: String,
}

#[derive(PartialEq)]
pub enum Primitive {
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Number(number) => write!(f, "{}", number),
            Primitive::Boolean(boolean) => write!(f, "{}", boolean),
            Primitive::Nil => write!(f, "nil"),
            Primitive::String(string) => write!(f, "{}", string),
        }
    }
}
