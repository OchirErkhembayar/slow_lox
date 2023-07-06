use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::token::{Token, TokenType};
use crate::error;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut map = HashMap::new();
        map.insert("and", TokenType::AND);
        map.insert("class", TokenType::CLASS);
        map.insert("else", TokenType::ELSE);
        map.insert("false", TokenType::FALSE);
        map.insert("for", TokenType::FOR);
        map.insert("fun", TokenType::FUN);
        map.insert("if", TokenType::IF);
        map.insert("nil", TokenType::NIL);
        map.insert("or", TokenType::OR);
        map.insert("print", TokenType::PRINT);
        map.insert("return", TokenType::RETURN);
        map.insert("super", TokenType::SUPER);
        map.insert("this", TokenType::THIS);
        map.insert("true", TokenType::TRUE);
        map.insert("var", TokenType::VAR);
        map.insert("while", TokenType::WHILE);
        map
    };
}

fn match_keyword(identifier: &str) -> TokenType {
    match KEYWORDS.get(identifier) {
        Some(token_type) => token_type.clone(),
        None => TokenType::IDENTIFIER,
    }
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(TokenType::EOF, String::new(), self.line));
        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        match self.advance() {
            '(' => self.make_token(TokenType::LEFT_PAREN, String::from("(")),
            ')' => self.make_token(TokenType::RIGHT_PAREN, String::from(")")),
            '{' => self.make_token(TokenType::LEFT_BRACE, String::from("{")),
            '}' => self.make_token(TokenType::RIGHT_BRACE, String::from("}")),
            ',' => self.make_token(TokenType::COMMA, String::from(",")),
            '.' => self.make_token(TokenType::DOT, String::from(".")),
            '-' => self.make_token(TokenType::MINUS, String::from("-")),
            '+' => self.make_token(TokenType::PLUS, String::from("+")),
            ';' => self.make_token(TokenType::SEMICOLON, String::from(";")),
            '*' => self.make_token(TokenType::STAR, String::from("*")),
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenType::BANG_EQUAL, String::from("!="));
                } else {
                    self.make_token(TokenType::BANG, String::from("!"));
                };
            },
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenType::EQUAL_EQUAL, String::from("=="));
                } else {
                    self.make_token(TokenType::EQUAL, String::from("="));
                }
            },
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenType::LESS_EQUAL, String::from("<="));
                } else {
                    self.make_token(TokenType::LESS, String::from("<"));
                }
            },
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenType::GREATER_EQUAL, String::from(">="));
                } else {
                    self.make_token(TokenType::GREATER, String::from(">"));
                }
            },
            '/' => {
                if self.match_char('*') {
                    while !(self.peek() == '*' && self.peak_next() == '/') && !self.is_at_end() {
                        if self.peek() == '\n' {
                            self.line += 1;
                        }
                        self.advance();
                    }
                    if self.is_at_end() {
                        error(self.line, "Unterminated block comment");
                        return
                    } else {
                        self.advance();
                    }
                    if self.is_at_end() {
                        error(self.line, "Unterminated block comment");
                        return
                    } else {
                        self.advance();
                    }
                } else if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.make_token(TokenType::SLASH, String::new());
                }
            },
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            '0'..='9' => self.number(),
            '_' | 'a'..='z' | 'A'..='Z' => self.identifier(),
            _ => error(self.line, "Unexpected character."),
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminated string");
            return
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_string();

        self.make_token(TokenType::STRING, value);
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peak_next().is_digit(10) {
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current].parse::<f64>().unwrap();

        self.make_token(TokenType::NUMBER, value.to_string());
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let str = &self.source[self.start..self.current];

        self.make_token(match_keyword(str), String::from(str));
    }

    fn peak_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn make_token(&mut self, token_type: TokenType, literal: String) {
        self.tokens.push(
            Token::new(token_type, literal, self.line)
        );
    }

    fn match_char(&mut self, char: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current) != Some(char) {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_comments() {
        let mut scanner = Scanner::new("/* This is a block comment */".to_string());
        let tokens = scanner.scan_tokens();
        assert_eq!(Token::new(TokenType::EOF, String::new(), 1), tokens[0]);
    }

    #[test]
    fn test_block_comment_with_slashes_in_it() {
        let mut scanner = Scanner::new("/* This is a block comment with // slashes in it */".to_string());
        let tokens = scanner.scan_tokens();
        assert_eq!(Token::new(TokenType::EOF, String::new(), 1), tokens[0]);
    }
}
