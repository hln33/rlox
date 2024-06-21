use crate::{
    expr::Expr,
    print_error,
    scanner::{Literal, Token, TokenType},
    stmt::Stmt,
};

#[derive(Debug)]
struct ParseError;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl Parser<'_> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];

        while !self.is_at_end() {
            statements.push(self.declaration().unwrap());
        }

        statements
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match res {
            Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statment();
        }

        self.expression_statement()
    }

    fn print_statment(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        let _ = self.consume(
            TokenType::Semicolon,
            String::from("Expect ';' after value."),
        );
        Ok(Stmt::Print(value))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, String::from("Expect variable name."))?;

        let mut initializer = None;
        if self.match_token(&[TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        let _ = self.consume(
            TokenType::Semicolon,
            String::from("Expect ';' after variable declaration"),
        );
        Ok(Stmt::Var { name, initializer })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        let _ = self.consume(
            TokenType::Semicolon,
            String::from("Expect ';' after expression."),
        );
        Ok(Stmt::Expression(value))
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(self.error(equals, "Invalid assignment target."));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_op(&[TokenType::BangEqual, TokenType::EqualEqual], |parser| {
            parser.comparison()
        })
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_op(
            &[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
            |parser| parser.term(),
        )
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_op(&[TokenType::Minus, TokenType::Plus], |parser| {
            parser.factor()
        })
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_op(&[TokenType::Slash, TokenType::Star], |parser| {
            parser.unary()
        })
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }
        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal {
                value: Literal::None,
            });
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                value: self.previous().literal,
            });
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable {
                name: self.previous(),
            });
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            let _ = self.consume(
                TokenType::RightParen,
                String::from("Expect ')' after expression"),
            );
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(self.error(self.peek().clone(), "Expected expression."))
    }

    fn parse_binary_op<F>(
        &mut self,
        operators: &[TokenType],
        mut parse_next_level: F,
    ) -> Result<Expr, ParseError>
    where
        F: FnMut(&mut Self) -> Result<Expr, ParseError>,
    {
        let mut expr = parse_next_level(self)?;

        while self.match_token(operators) {
            let operator = self.previous();
            let right = parse_next_level(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types.iter() {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek().clone(), &message))
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn error(&self, token: Token, message: &str) -> ParseError {
        print_error(token.line.try_into().unwrap(), token.lexeme, message);
        ParseError {}
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
