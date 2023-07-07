use crate::token::{Token, TokenType};
use crate::expr::{Expr, Binary, Literal, Grouping, Unary};

struct Input<'a> {
    tokens: &'a Vec<Token>,
    index: usize,
}

impl<'a> Input<'a> {
    fn new(tokens: &'a Vec<Token>) -> Input<'a> {
        Input {
            tokens,
            index: 0,
        }
    }

    fn from_input(&self, index: usize) -> Input<'a> {
        Input {
            tokens: self.tokens,
            index,
        }
    }
}

fn parse<'a>(tokens: &mut Vec<Token>) -> Result<Expr<'a>, ParserError<'a>> {
    expression(tokens, 0)
}

fn expression<'a>(tokens: &mut Vec<Token>, index: usize) -> Result<Expr<'a>, ParserError<'a>> {
    equality(tokens, index)
}

fn equality<'a>(tokens: &mut Vec<Token>, index: usize) -> Result<Expr<'a>, ParserError<'a>> {
    let mut expr = comparison(tokens, index)?;

    while match_token(tokens, index, &[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
        let operator = &tokens[index];
        let right = comparison(tokens, index + 1)?;
        expr = Expr::Binary(
            Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        );
    }

    Ok(expr)
}

fn match_token(tokens: &Vec<Token>, index: usize, types: &[TokenType]) -> bool {
    if index >= tokens.len() {
        return false;
    }

    for token_type in types {
        if tokens[index].token_type == *token_type {
            return true;
        }
    }

    false
}

pub struct ParserError<'a> {
    pub token: &'a Token,
    pub message: String,
}
