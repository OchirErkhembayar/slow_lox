use std::collections::HashMap;

use crate::{interpreter::{Interpreter, InterpretError}, stmt::Stmt, token::{TokenType, Token}, expr::Expr};

struct Resolver {
    stacks: Vec<HashMap<String, bool>>,
    interpreter: Interpreter,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
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

impl Resolver {
    pub fn resolve(&mut self, stmts: Vec<Stmt>) -> Result<(), InterpretError> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_assignment_expr(&mut self) -> Result<(), InterpretError> {
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: Stmt) -> Result<(), InterpretError> {
        self.interpreter.interpret(stmt)
    }

    fn resolve_expr(&mut self, expr: crate::expr::Expr) -> Result<(), InterpretError> {
        self.interpreter.interpret_expr(expr)?;
        Ok(())
    }

    fn resolve_var(&mut self, stmt: Stmt) -> Result<(), InterpretError> {
        if let Stmt::Var(name, initializer) = stmt {
            self.declare(name.clone())?;
            if let Some(initializer) = initializer {
                self.resolve_expr(initializer)?;
            }
            self.define(name.clone())?;
            if let Some(scope) = self.stacks.last_mut() {
                scope.insert(name.lexeme, false);
            }
            Ok(())
        } else {
            Err(InterpretError::new(
                String::from("Expected variable declaration."),
                Token::new(TokenType::VAR, String::from(""), 0),
            ))
        }
    }

    fn resolve_var_expr(&mut self,expr: Expr) {
        if let Expr::Variable(var) = expr {
            if let Some(scope) = self.stacks.last_mut() {
                if scope.get(&var.name.lexeme) == Some(&false) {
                    crate::error(var.name.line, "Cannot read local variable in its own initializer.");
                }
            }
            if let Some(scope) = self.stacks.last_mut() {
                scope.insert(var.name.lexeme, true);
            }
        }
    }
}
