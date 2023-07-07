use crate::token::Token;

#[derive(Clone)]
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

#[derive(Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Clone)]
pub struct Literal {
    pub value: Token,
}

#[derive(Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

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
    }
}
