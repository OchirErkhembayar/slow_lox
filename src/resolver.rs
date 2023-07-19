use std::collections::HashMap;

use crate::{interpreter::{Interpreter, InterpretError}, stmt::Stmt, token::Token, expr::Expr};

pub struct Resolver<'a> {
    stacks: Vec<HashMap<String, bool>>,
    interpreter: &'a mut Interpreter,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            stacks: Vec::new(),
            interpreter,
        }
    }

    fn begin_scope(&mut self) {
        self.stacks.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.stacks.pop();
    }

    fn declare(&mut self, name: Token) -> Result<(), InterpretError> {
        if let Some(scope) = self.stacks.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(InterpretError::new(
                    String::from("Variable with this name already declared in this scope."),
                    name,
                ));
            }
            scope.insert(name.lexeme.clone(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: Token) -> Result<(), InterpretError> {
        if let Some(scope) = self.stacks.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
        Ok(())
    }
}

impl<'a> Resolver<'a> {
    pub fn resolve(&mut self, stmts: Vec<Stmt>) -> Result<(), InterpretError> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: Stmt) -> Result<(), InterpretError> {
        match stmt {
            Stmt::Function(token, tokens, stmts) => {
                self.declare(token.clone())?;
                self.define(token.clone())?;
                self.resolve_function(tokens, stmts)?;
            },
            Stmt::Expr(expr) => {
                self.resolve_expr(expr)?;
            },
            Stmt::If(condition, then_branch, else_branch) => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(*then_branch)?;
                if let Some(else_stmt) = else_branch {
                    self.resolve_stmt(*else_stmt)?;
                }
            },
            Stmt::Print(expr) => {
                self.resolve_expr(expr)?;
            },
            Stmt::Return(_, expr) => {
                if let Some(expr) = expr {
                    self.resolve_expr(expr)?;
                }
            },
            Stmt::While(condition, body) => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(*body)?;
            },
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve(stmts)?;
                self.end_scope();
            },
            Stmt::Var(name, expr) => {
                self.declare(name.clone())?;
                if let Some(expr) = expr {
                    self.resolve_expr(expr)?;
                }
                self.define(name)?;
            },
            Stmt::Assign(_, expr) => {
                self.resolve_expr(expr)?;
            },
            Stmt::Break => {},
        }
        Ok(())
    }

    fn resolve_expr(&mut self, expr: Expr) -> Result<(), InterpretError> {
        match expr {
            Expr::Call(call) => {
                self.resolve_expr(*call.callee)?;
                for arg in call.arguments {
                    self.resolve_expr(arg)?;
                }
            },
            Expr::Assign(assign) => {
                self.resolve_expr(*assign.value.clone())?;
                self.resolve_local(*assign.value, assign.name);
            },
            Expr::Binary(binary) => {
                self.resolve_expr(*binary.left)?;
                self.resolve_expr(*binary.right)?;
            },
            Expr::Grouping(grouping) => {
                self.resolve_expr(*grouping.expression)?;
            },
            Expr::Literal(_) => {},
            Expr::Logical(logical) => {
                self.resolve_expr(*logical.left)?;
                self.resolve_expr(*logical.right)?;
            },
            Expr::Unary(unary) => {
                self.resolve_expr(*unary.right)?;
            },
            Expr::Variable(var) => {
                if let Some(scope) = self.stacks.last_mut() {
                    if scope.get(&var.name.lexeme) == Some(&false) {
                        crate::error(var.name.line, "Cannot read local variable in its own initializer.");
                    }
                }
                self.resolve_var_expr(Expr::Variable(var))?;
            },
            Expr::Ternary(ternary) => {
                self.resolve_expr(*ternary.condition)?;
                self.resolve_expr(*ternary.then_branch)?;
                self.resolve_expr(*ternary.else_branch)?;
            },
        }
        Ok(())
    }

    fn resolve_var_expr(&mut self, expr: Expr) -> Result<(), InterpretError> {
        let expr_clone = expr.clone();
        if let Expr::Variable(var) = expr {
            if let Some(scope) = self.stacks.last_mut() {
                if scope.get(&var.name.lexeme) == Some(&false) {
                    crate::error(var.name.line, "Cannot read local variable in its own initializer.");
                }
            }
            self.resolve_local(expr_clone, var.name);
        }
        Ok(())
    }

    fn resolve_function(&mut self, params: Vec<Token>, stmts: Vec<Stmt>) -> Result<(), InterpretError> {
        self.begin_scope();
        for param in params {
            self.declare(param.clone())?;
            self.define(param.clone())?;
        }
        self.resolve(stmts)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_local(&mut self, expr: Expr, name: Token) {
        for (i, scope) in self.stacks.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, i);
                return;
            }
        }
    }
}
