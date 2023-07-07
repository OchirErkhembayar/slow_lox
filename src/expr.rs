use crate::token::Token;

pub enum Expr<'a> {
    Binary(Binary<'a>),
    Grouping(Grouping<'a>),
    Literal(Literal<'a>),
    Unary(Unary<'a>),
}

pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: &'a Token,
    pub right: Box<Expr<'a>>,
}

pub struct Grouping<'a> {
    pub expression: Box<Expr<'a>>,
}

pub struct Literal<'a> {
    pub value: &'a Token,
}

pub struct Unary<'a> {
    pub operator: &'a Token,
    pub right: Box<Expr<'a>>,
}

pub fn print(expr: Expr) -> String {
    match expr {
        Expr::Binary(binary) => {
            format!("({} {} {})", binary.operator.lexeme, print(*binary.left), print(*binary.right))
        }
        Expr::Grouping(grouping) => {
            format!("(group {})", print(*grouping.expression))
        }
        Expr::Literal(literal) => {
            format!("{}", literal.value.lexeme)
        }
        Expr::Unary(unary) => {
            format!("({} {})", unary.operator.lexeme, print(*unary.right))
        }
    }
}
