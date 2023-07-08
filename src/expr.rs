use crate::token::Token;

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Ternary(Ternary),
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

#[derive(Clone, Debug)]
pub struct Ternary {
    pub condition: Box<Expr>,
    pub then_branch: Box<Expr>,
    pub else_branch: Box<Expr>,
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
        Expr::Ternary(ternary) => {
            format!(
                "({} ? {} : {})",
                print(*ternary.condition),
                print(*ternary.then_branch),
                print(*ternary.else_branch)
            )
        }
    }
}
