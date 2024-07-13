use std::collections::HashMap;

use crate::syntax::token::{Literal, Token, TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    has_error: bool,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            has_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            literal: Literal::None,
            line: self.line,
        });
        &self.tokens
    }

    fn scan_token(&mut self) {
        let token = self.source.as_bytes()[self.current] as char;
        self.current += 1;

        match token {
            '(' => self.add_token(TokenType::LeftParen, Literal::None),
            ')' => self.add_token(TokenType::RightParen, Literal::None),
            '{' => self.add_token(TokenType::LeftBrace, Literal::None),
            '}' => self.add_token(TokenType::RightBrace, Literal::None),
            ',' => self.add_token(TokenType::Comma, Literal::None),
            '.' => self.add_token(TokenType::Dot, Literal::None),
            '-' => self.add_token(TokenType::Minus, Literal::None),
            '+' => self.add_token(TokenType::Plus, Literal::None),
            ';' => self.add_token(TokenType::Semicolon, Literal::None),
            '*' => self.add_token(TokenType::Star, Literal::None),

            // single or double length operators
            '!' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::BangEqual, Literal::None);
                } else {
                    self.add_token(TokenType::Bang, Literal::None);
                }
            }
            '=' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::EqualEqual, Literal::None);
                } else {
                    self.add_token(TokenType::Equal, Literal::None);
                }
            }
            '<' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::LessEqual, Literal::None);
                } else {
                    self.add_token(TokenType::Less, Literal::None);
                }
            }
            '>' => {
                if self.match_next_token('=') {
                    self.add_token(TokenType::GreaterEqual, Literal::None);
                } else {
                    self.add_token(TokenType::Greater, Literal::None);
                }
            }
            '/' => {
                if self.match_next_token('/') {
                    // comment goes until the end of the line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.current += 1;
                    }
                } else {
                    self.add_token(TokenType::Slash, Literal::None);
                }
            }

            // newlines and whitespace
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => self.line += 1,

            // string literals
            '"' => self.add_string(),

            _ => {
                if token.is_ascii_digit() {
                    self.add_number();
                } else if token.is_alphabetic() || token == '_' {
                    self.add_identifier();
                } else {
                    eprintln!("{}: Unexpected character.", self.line);
                    self.has_error = true;
                }
            }
        }
    }

    fn match_next_token(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        };
        if self.source.as_bytes()[self.current] as char != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType, literal: Literal) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token {
            token_type,
            lexeme: text.to_string(),
            literal,
            line: self.line,
        })
    }

    fn add_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.current += 1;
        }

        if self.is_at_end() {
            eprintln!("{}: Unterminated string.", self.line);
            self.has_error = true;
        }

        // the closing "
        self.current += 1;

        // Trim surrounding quotes
        let value = self
            .source
            .get((self.start + 1)..(self.current - 1))
            .unwrap()
            .to_string();
        self.add_token(TokenType::String, Literal::String(value));
    }

    fn add_number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.current += 1;
        }

        // look for fractional part of number
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // consume the '.'
            self.current += 1;
        }

        while self.peek().is_ascii_digit() {
            self.current += 1;
        }

        let value: f64 = self
            .source
            .get(self.start..self.current)
            .unwrap()
            .parse()
            .unwrap();
        self.add_token(TokenType::Number, Literal::Number(value))
    }

    fn add_identifier(&mut self) {
        let mut keywords: HashMap<String, TokenType> = HashMap::new();
        keywords.insert(String::from("and"), TokenType::And);
        keywords.insert(String::from("class"), TokenType::Class);
        keywords.insert(String::from("else"), TokenType::Else);
        keywords.insert(String::from("false"), TokenType::False);
        keywords.insert(String::from("for"), TokenType::For);
        keywords.insert(String::from("fun"), TokenType::Fun);
        keywords.insert(String::from("if"), TokenType::If);
        keywords.insert(String::from("nil"), TokenType::Nil);
        keywords.insert(String::from("or"), TokenType::Or);
        keywords.insert(String::from("print"), TokenType::Print);
        keywords.insert(String::from("return"), TokenType::Return);
        keywords.insert(String::from("super"), TokenType::Super);
        keywords.insert(String::from("this"), TokenType::This);
        keywords.insert(String::from("true"), TokenType::True);
        keywords.insert(String::from("var"), TokenType::Var);
        keywords.insert(String::from("while"), TokenType::While);

        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.current += 1;
        }

        let text = self.source.get(self.start..self.current).unwrap();
        match keywords.get(text) {
            Some(token_type) => self.add_token(token_type.clone(), Literal::None),
            None => self.add_token(TokenType::Identifier, Literal::None),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current] as char
    }

    fn peek_next(&self) -> char {
        if (self.current + 1) >= self.source.len() {
            return '\0';
        }
        self.source.as_bytes()[self.current + 1] as char
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers() {
        let mut scanner = Scanner::new(String::from("andy formless fo _ _123 _abc ab123 \n abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_"));
        let tokens = scanner.scan_tokens();

        let expected_tokens = [
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("andy"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("formless"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("fo"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("_"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("_123"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("_abc"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("ab123"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from(
                    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_",
                ),
                literal: Literal::None,
                line: 2,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: String::new(),
                literal: Literal::None,
                line: 2,
            },
        ];

        assert_eq!(tokens.len(), expected_tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[i]);
        }
    }

    #[test]
    fn keywords() {
        let mut scanner = Scanner::new(String::from(
            "and class else false for fun if nil or return super this true var while",
        ));
        let tokens = scanner.scan_tokens();

        let expected_tokens = [
            Token {
                token_type: TokenType::And,
                lexeme: String::from("and"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Class,
                lexeme: String::from("class"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Else,
                lexeme: String::from("else"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::False,
                lexeme: String::from("false"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::For,
                lexeme: String::from("for"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Fun,
                lexeme: String::from("fun"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::If,
                lexeme: String::from("if"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Nil,
                lexeme: String::from("nil"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Or,
                lexeme: String::from("or"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Return,
                lexeme: String::from("return"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Super,
                lexeme: String::from("super"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::This,
                lexeme: String::from("this"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::True,
                lexeme: String::from("true"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Var,
                lexeme: String::from("var"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::While,
                lexeme: String::from("while"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: String::new(),
                literal: Literal::None,
                line: 1,
            },
        ];

        assert_eq!(tokens.len(), expected_tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[i]);
        }
    }

    #[test]
    fn numbers() {
        let mut scanner = Scanner::new(String::from("123\n123.456\n.456\n123."));
        let tokens = scanner.scan_tokens();

        let expected_tokens = [
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("123"),
                literal: Literal::Number(123.0),
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("123.456"),
                literal: Literal::Number(123.456),
                line: 2,
            },
            Token {
                token_type: TokenType::Dot,
                lexeme: String::from("."),
                literal: Literal::None,
                line: 3,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("456"),
                literal: Literal::Number(456.0),
                line: 3,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("123"),
                literal: Literal::Number(123.0),
                line: 4,
            },
            Token {
                token_type: TokenType::Dot,
                lexeme: String::from("."),
                literal: Literal::None,
                line: 4,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: String::new(),
                literal: Literal::None,
                line: 4,
            },
        ];

        assert_eq!(tokens.len(), expected_tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[i]);
        }
    }

    #[test]
    fn punctuators() {
        let mut scanner = Scanner::new(String::from("(){};,+-*!===<=>=!=<>/."));
        let tokens = scanner.scan_tokens();

        let expected_tokens = [
            Token {
                token_type: TokenType::LeftParen,
                lexeme: String::from("("),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::RightParen,
                lexeme: String::from(")"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::LeftBrace,
                lexeme: String::from("{"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::RightBrace,
                lexeme: String::from("}"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Semicolon,
                lexeme: String::from(";"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Comma,
                lexeme: String::from(","),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Plus,
                lexeme: String::from("+"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Minus,
                lexeme: String::from("-"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Star,
                lexeme: String::from("*"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::BangEqual,
                lexeme: String::from("!="),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::EqualEqual,
                lexeme: String::from("=="),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::LessEqual,
                lexeme: String::from("<="),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::GreaterEqual,
                lexeme: String::from(">="),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::BangEqual,
                lexeme: String::from("!="),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Less,
                lexeme: String::from("<"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Greater,
                lexeme: String::from(">"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Slash,
                lexeme: String::from("/"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Dot,
                lexeme: String::from("."),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: String::from(""),
                literal: Literal::None,
                line: 1,
            },
        ];

        assert_eq!(tokens.len(), expected_tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[i]);
        }
    }

    #[test]
    fn strings() {
        let mut scanner = Scanner::new(String::from("\"\" \n \"string\""));
        let tokens = scanner.scan_tokens();

        let expected_tokens = [
            Token {
                token_type: TokenType::String,
                lexeme: String::from("\"\""),
                literal: Literal::String(String::from("")),
                line: 1,
            },
            Token {
                token_type: TokenType::String,
                lexeme: String::from("\"string\""),
                literal: Literal::String(String::from("string")),
                line: 2,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: String::from(""),
                literal: Literal::None,
                line: 2,
            },
        ];

        assert_eq!(tokens.len(), expected_tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[i]);
        }
    }

    #[test]
    fn whitespace() {
        let mut scanner = Scanner::new(String::from(
            "space    tabs				newlines




        end",
        ));
        let tokens = scanner.scan_tokens();

        let expected_tokens = [
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("space"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("tabs"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("newlines"),
                literal: Literal::None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("end"),
                literal: Literal::None,
                line: 6,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: String::from(""),
                literal: Literal::None,
                line: 6,
            },
        ];

        assert_eq!(tokens.len(), expected_tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[i]);
        }
    }
}
