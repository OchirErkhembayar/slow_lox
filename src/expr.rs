use crate::token::{Token, TokenType};

enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

struct Binary {
    left: Box<Expr>,
    operator: Token,
    right: Box<Expr>,
}

struct Grouping {
    expression: Box<Expr>,
}

struct Literal {
    value: Token,
}

struct Unary {
    operator: Token,
    right: Box<Expr>,
}

