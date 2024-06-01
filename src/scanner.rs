use core::fmt;
use std::collections::HashMap;

#[derive(Debug)]
enum Literal {
    String(String),
    Number(f64),
    None,
}

#[derive(Debug, Clone)]
enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two chracter tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    Nil,
    If,
    Print,
    Or,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Literal,
    line: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

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
                if token.is_digit(10) {
                    self.add_number();
                } else if token.is_alphabetic() {
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
        return true;
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
        while self.peek().is_digit(10) {
            self.current += 1;
        }

        // look for fractional part of number
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // consume the '.'
            self.current += 1;
        }

        while self.peek().is_digit(10) {
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

        while self.peek().is_alphanumeric() {
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
