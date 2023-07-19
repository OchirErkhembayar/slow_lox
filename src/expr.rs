use crate::primitive::Primitive;
use crate::token::Token;
use std::fmt::Debug;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Logical(Logical),
    Ternary(Ternary),
    Variable(Variable),
    Assign(Assignment),
    Call(Call),
    Get(GetExpr),
    Set(SetExpr),
}

// 1 + 2, 3 * 4, etc.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

// (expression)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

// true, false, nil, 1, 2, 3, etc.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Literal {
    pub value: Token,
}

// -1, !true, etc.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

// condition ? then_branch : else_branch
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ternary {
    pub condition: Box<Expr>,
    pub then_branch: Box<Expr>,
    pub else_branch: Box<Expr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Variable {
    pub name: Token,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub primitive: Primitive,
    pub token: Token,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GetExpr {
    pub expr: Box<Expr>,
    pub name: Token,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SetExpr {
    pub expr: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}

#[allow(dead_code)]
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
        Expr::Variable(variable) => variable.name.lexeme,
        Expr::Assign(assignment) => {
            format!(
                "(= {} {})",
                assignment.name.lexeme,
                print(*assignment.value)
            )
        }
        Expr::Logical(logical) => {
            format!(
                "({} {} {})",
                logical.operator.lexeme,
                print(*logical.left),
                print(*logical.right)
            )
        }
        Expr::Call(call) => {
            let mut args = String::new();
            for arg in call.arguments {
                args.push_str(&print(arg));
                args.push_str(", ");
            }
            format!(
                "(call {} {} {})",
                print(*call.callee),
                call.paren.lexeme,
                args
            )
        }
        Expr::Get(get_expr) => {
            format!("(get {})", print(*get_expr.expr))
        }
        Expr::Set(set_expr) => {
            format!(
                "(set {} {} {})",
                set_expr.name.lexeme,
                print(*set_expr.expr),
                print(*set_expr.value)
            )
        }
    }
}
