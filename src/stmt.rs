use crate::{expr::Expr, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Assign(Token, Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Break,
    Function(Token, Vec<Token>, Vec<Stmt>),
    Class(Token, Vec<Stmt>),
    Return(Token, Option<Expr>),
}
