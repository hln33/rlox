use core::fmt;

#[derive(Debug)]
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
    IF,
    Print,
    OR,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

// #[derive(Debug)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: String,
    line: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} {}", self.token_type, self.lexeme, self.literal)
    }
}

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn add_token(&mut self, token_type: TokenType, literal: String) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token {
            token_type,
            lexeme: text.to_string(),
            literal,
            line: self.line,
        })
    }

    fn scan_token(&mut self, token: char) {
        let literal = String::new();
        match token {
            '(' => self.add_token(TokenType::LeftParen, literal),
            ')' => self.add_token(TokenType::RightParen, literal),
            '{' => self.add_token(TokenType::LeftBrace, literal),
            '}' => self.add_token(TokenType::RightBrace, literal),
            ',' => self.add_token(TokenType::Comma, literal),
            '.' => self.add_token(TokenType::Dot, literal),
            '-' => self.add_token(TokenType::Minus, literal),
            '+' => self.add_token(TokenType::RightParen, literal),
            ';' => self.add_token(TokenType::Semicolon, literal),
            '*' => self.add_token(TokenType::Star, literal),
            // todo!
            _ => eprintln!("{}: Unexpected character", self.line),
        }
    }

    fn scan_tokens(&mut self) {
        let cloned_source = self.source.clone();
        for ch in cloned_source.chars() {
            self.start = self.current;
            self.scan_token(ch);
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            literal: String::new(),
            line: self.line,
        });
    }
}
