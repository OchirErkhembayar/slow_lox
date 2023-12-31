use crate::expr::{
    Assignment, Binary, Call, Expr, GetExpr, Grouping, Literal, Logical, SetExpr, Ternary, Unary,
    Variable,
};
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn match_token(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(ParseError {
            token: self.peek(),
            message: message.to_string(),
        })
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SEMICOLON {
                return;
            }

            match self.peek().token_type {
                TokenType::CLASS => return,
                TokenType::FUN => return,
                TokenType::VAR => return,
                TokenType::FOR => return,
                TokenType::IF => return,
                TokenType::WHILE => return,
                TokenType::PRINT => return,
                TokenType::RETURN => return,
                _ => self.current += 1,
            }
        }
    }
}

impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            let statement = self.declaration();
            match statement {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    crate::error(error.token.line, error.message.as_str());
                    self.synchronize();
                    continue;
                }
            }
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(vec![TokenType::FUN]) {
            return self.func_declaration("function".to_string());
        }
        if self.match_token(vec![TokenType::VAR]) {
            return self.var_declaration();
        }
        if self.match_token(vec![TokenType::FOR]) {
            return self.for_statement();
        }
        if self.match_token(vec![TokenType::IF]) {
            return self.if_statement();
        }
        if self.match_token(vec![TokenType::WHILE]) {
            return self.while_statement();
        }
        if self.match_token(vec![TokenType::CLASS]) {
            return self.class_declaration();
        }

        self.statement()
    }

    fn class_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect class name.")?;
        self.consume(TokenType::LEFT_BRACE, "Expect '{' before class body.")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            methods.push(self.func_declaration("method".to_string())?);
        }
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after class body.")?;
        Ok(Stmt::Class(name, methods))
    }

    fn func_declaration(&mut self, kind: String) -> Result<Stmt, ParseError> {
        let name = self.consume(
            TokenType::IDENTIFIER,
            format!("Expect {} name.", kind).as_str(),
        )?;
        self.consume(
            TokenType::LEFT_PAREN,
            format!("Expect '(' after {} name.", kind).as_str(),
        )?;
        let mut parameters = Vec::new();
        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                if parameters.len() >= 255 {
                    return Err(ParseError {
                        token: self.peek(),
                        message: "Can't have more than 255 parameters.".to_string(),
                    });
                }
                parameters.push(self.consume(TokenType::IDENTIFIER, "Expect parameter name.")?);
                if !self.match_token(vec![TokenType::COMMA]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after parameters.")?;
        self.consume(
            TokenType::LEFT_BRACE,
            format!("Expect '{{' before {} body.", kind).as_str(),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function(name, parameters, body))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name.")?;

        let initializer = if self.match_token(vec![TokenType::EQUAL]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::SEMICOLON, "Expect ';' after value")?;

        Ok(Stmt::Var(name, initializer))
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'.")?;

        let initializer = if self.match_token(vec![TokenType::SEMICOLON]) {
            None
        } else if self.match_token(vec![TokenType::VAR]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = if !self.check(TokenType::SEMICOLON) {
            Some(self.expression()?)
        } else {
            // Expr::Literal(Literal {
            //    value: Token { token_type: TokenType::TRUE, lexeme: "true".to_string(), line: 0 }
            // })
            None
        };

        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenType::RIGHT_PAREN) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![
                body,
                match increment {
                    Expr::Assign(assignment) => {
                        Stmt::Assign(assignment.name.clone(), Expr::Assign(assignment))
                    }
                    _ => Stmt::Expr(increment),
                },
            ]);
        }

        if condition.is_none() {
            condition = Some(Expr::Literal(Literal {
                value: Token {
                    token_type: TokenType::TRUE,
                    lexeme: "true".to_string(),
                    line: 0,
                },
            }));
        }

        body = Stmt::While(condition.unwrap(), Box::new(body));

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after if condition.")?;
        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.match_token(vec![TokenType::ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If(condition, Box::new(then_branch), else_branch))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(vec![TokenType::PRINT]) {
            return self.print_statement();
        }
        if self.match_token(vec![TokenType::RETURN]) {
            return self.return_statement();
        }
        if self.match_token(vec![TokenType::LEFT_BRACE]) {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.match_token(vec![TokenType::BREAK]) {
            return Ok(Stmt::Break);
        }

        self.expression_statement()
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.")?;
        Ok(stmts)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(TokenType::SEMICOLON) {
            value = Some(self.expression()?);
        }

        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.")?;
        Ok(Stmt::Return(keyword, value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        match value {
            Expr::Assign(assignment) => {
                Ok(Stmt::Assign(assignment.name.clone(), *assignment.value))
            }
            _ => Ok(Stmt::Expr(value)),
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        // Check if the expression starts with a binary operator
        let binary_operators = vec![
            TokenType::BANG_EQUAL,
            TokenType::EQUAL_EQUAL,
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
            TokenType::MINUS,
            TokenType::PLUS,
            TokenType::SLASH,
            TokenType::STAR,
            TokenType::QUESTION,
            TokenType::COLON,
            TokenType::COMMA,
        ];
        if self.match_token(binary_operators.clone()) {
            let token = self.previous();
            crate::error(
                token.line,
                &format!("Expression cannot start with {}", token.lexeme),
            );
            while !self.is_at_end() && !self.match_token(binary_operators.clone()) {
                self.advance();
            }
        }

        match self.assignment() {
            Ok(expr) => Ok(expr),
            Err(err) => {
                crate::error(err.token.line, err.message.as_str());
                Err(err)
            }
        }

        // C style comma operator, e.g. (1, 2, 3). The value of the expression is the last value.
        // Not sure if this is working correctly.
        // Not working correctly. The comma operator should be left associative, but this is right associative.
        // Now it's straight up causing bugs so I'm disabling it.
        // while self.peek().token_type == TokenType::COMMA {
        //    self.advance();
        //    expr = match self.expression() {
        //        Ok(expr) => Ok(expr),
        //        Err(err) => {
        //            crate::error(err.token.line, err.message.as_str());
        //            return Err(err);
        //        }
        //    };
        // }

        // expr
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.ternary()?;

        if self.match_token(vec![TokenType::EQUAL]) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => {
                    return Ok(Expr::Assign(Assignment {
                        name: name.name,
                        value: Box::new(value),
                    }));
                }
                Expr::Get(get) => {
                    let set = Ok(Expr::Set(SetExpr {
                        expr: get.expr,
                        name: get.name,
                        value: Box::new(value),
                    }));
                    return set;
                }
                _ => {
                    crate::error(equals.line, "Invalid assignment target.");
                    return Err(ParseError {
                        token: equals,
                        message: "Invalid assignment target.".to_string(),
                    });
                }
            }
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.or()?;

        if self.peek().token_type == TokenType::QUESTION {
            self.advance();
            let then_branch = self.expression()?;
            self.consume(TokenType::COLON, "Expect ':' after then branch of ternary")?;
            let else_branch = self.expression()?;
            expr = Expr::Ternary(Ternary {
                condition: Box::new(expr),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.peek().token_type == TokenType::OR {
            let operator = self.advance();
            let right = self.and()?;
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.peek().token_type == TokenType::AND {
            let operator = self.advance();
            let right = self.equality()?;
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.peek().token_type == TokenType::BANG_EQUAL
            || self.peek().token_type == TokenType::EQUAL_EQUAL
        {
            let operator = self.advance();
            let right = self.comparison()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.peek().token_type == TokenType::GREATER
            || self.peek().token_type == TokenType::GREATER_EQUAL
            || self.peek().token_type == TokenType::LESS
            || self.peek().token_type == TokenType::LESS_EQUAL
        {
            let operator = self.advance();
            let right = self.term()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.peek().token_type == TokenType::PLUS
            || self.peek().token_type == TokenType::MINUS
        {
            let operator = self.advance();
            let right = self.factor()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.peek().token_type == TokenType::SLASH
            || self.peek().token_type == TokenType::STAR
        {
            let operator = self.advance();
            let right = self.unary()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek().token_type == TokenType::BANG || self.peek().token_type == TokenType::MINUS {
            let operator = self.advance();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            }));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(vec![TokenType::LEFT_PAREN]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(vec![TokenType::DOT]) {
                let name = self.consume(TokenType::IDENTIFIER, "Expect property name after .")?;
                expr = Expr::Get(GetExpr {
                    expr: Box::new(expr),
                    name,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    crate::error(self.peek().line, "Can't have more than 255 arguments.");
                    return Err(ParseError {
                        token: self.peek(),
                        message: "Can't have more than 255 arguments.".to_string(),
                    });
                }
                arguments.push(self.expression()?);
                if !self.match_token(vec![TokenType::COMMA]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.")?;

        Ok(Expr::Call(Call {
            callee: Box::new(callee),
            paren,
            arguments,
        }))
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(vec![TokenType::FALSE]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous(),
            }));
        }
        if self.match_token(vec![TokenType::TRUE]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous(),
            }));
        }
        if self.match_token(vec![TokenType::NIL]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous(),
            }));
        }

        if self.match_token(vec![TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expr::Literal(Literal {
                value: self.previous(),
            }));
        }

        if self.match_token(vec![TokenType::IDENTIFIER]) {
            return Ok(Expr::Variable(Variable {
                name: self.previous(),
            }));
        }

        if self.match_token(vec![TokenType::LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Grouping {
                expression: Box::new(expr),
            }));
        }

        Err(ParseError {
            token: self.peek(),
            message: "Expect expression.".to_string(),
        })
    }
}
