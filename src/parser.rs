use crate::expr::{Binary, Expr, Grouping, Literal, Ternary, Unary, Variable, Assignment};
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

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
        if self.match_token(vec![TokenType::VAR]) {
            return self.var_declaration();
        }

        self.statement()
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

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(vec![TokenType::PRINT]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        match value {
            Expr::Assign(assignment) => Ok(Stmt::Assign(assignment.name.clone(), *assignment.value)),
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

        let mut expr = match self.assignment() {
            Ok(expr) => Ok(expr),
            Err(err) => {
                crate::error(err.token.line, err.message.as_str());
                return Err(err);
            }
        };

        // C style comma operator, e.g. (1, 2, 3). The value of the expression is the last value.
        // Not sure if this is working correctly.
        // Not working correctly. The comma operator should be left associative, but this is right associative.
        while self.peek().token_type == TokenType::COMMA {
            self.advance();
            expr = match self.expression() {
                Ok(expr) => Ok(expr),
                Err(err) => {
                    crate::error(err.token.line, err.message.as_str());
                    return Err(err);
                }
            };
        }

        expr
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
                        value: Box::new(value)
                    }));
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
        let mut expr = self.equality()?;

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
        self.primary()
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
