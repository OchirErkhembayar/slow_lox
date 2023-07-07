use crate::expr::{Binary, Expr, Grouping, Literal, Unary};
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn match_token(&self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
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
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        match self.expression() {
            Ok(expr) => Ok(expr),
            Err(err) => {
                crate::error(err.token.line, err.message.as_str());
                Err(err)
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
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
            println!("I'm in unary");
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
            println!("Tokens: {:?}", self.tokens);
            println!("Current : {}", self.current);
            return Ok(Expr::Literal(Literal {
                value: self.previous(),
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

pub struct ParseError {
    pub token: Token,
    pub message: String,
}
