use std::collections::HashMap;

use crate::{interpreter::{Interpreter, InterpretError}, stmt::Stmt, token::{TokenType, Token}};

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
}

impl Resolver {
    pub fn resolve(&mut self, stmts: Vec<Stmt>) -> Result<(), InterpretError> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: Stmt) -> Result<(), InterpretError> {
        self.interpreter.interpret(stmt)
    }

    fn resolve_expr(&mut self, expr: crate::expr::Expr) -> Result<(), InterpretError> {
        self.interpreter.interpret_expr(expr)?;
        Ok(())
    }

    fn resolve_var(&mut self, name: String) -> Result<(), InterpretError> {
        if let Some(scope) = self.stacks.last_mut() {
            if scope.contains_key(&name) {
                return Err(InterpretError::new(
                    String::from("Variable with this name already declared in this scope."),
                    Token::new(TokenType::IDENTIFIER, name, 0),
                ));
            }
            scope.insert(name, false);
        }
        Ok(())
    }
}
