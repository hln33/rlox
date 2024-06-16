use crate::{
    expr::Expr,
    print_error,
    scanner::{Literal, Token, TokenType},
};

#[derive(Debug)]
struct ParseError {}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl Parser<'_> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Expr {
        match self.expression() {
            Ok(expr) => expr,
            Err(_) => panic!("parsing error!"),
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right_expr = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right_expr = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right_expr = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right_expr = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expr),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right_expr = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right_expr),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(vec![TokenType::False]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }
        if self.match_token(vec![TokenType::True]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }
        if self.match_token(vec![TokenType::Nil]) {
            return Ok(Expr::Literal {
                value: Literal::None,
            });
        }

        if self.match_token(vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                value: self.previous().literal,
            });
        }

        if self.match_token(vec![TokenType::Nil]) {
            let expr = self.expression()?;
            let _ = self.consume(
                TokenType::RightParen,
                String::from("Expect ')' after expression"),
            );
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(self.error(self.peek().clone(), String::from("Expected expression.")))

        // let token_type = &self.peek().token_type;
        // match token_type {
        //     TokenType::False => {
        //         self.advance();
        //         Ok(Expr::Literal {
        //             value: Literal::Bool(false),
        //         })
        //     }
        //     TokenType::True => {
        //         self.advance();
        //         Ok(Expr::Literal {
        //             value: Literal::Bool(true),
        //         })
        //     }
        //     TokenType::Nil => {
        //         self.advance();
        //         Ok(Expr::Literal {
        //             value: Literal::None,
        //         })
        //     }
        //     TokenType::Number | TokenType::String => {
        //         self.advance();
        //         Ok(Expr::Literal {
        //             value: self.previous().literal,
        //         })
        //     }
        //     TokenType::LeftParen => {
        //         self.advance();

        //         let expr = self.expression()?;
        //         let _ = self.consume(
        //             TokenType::RightParen,
        //             String::from("Expect ')' after expression"),
        //         );
        //         Ok(Expr::Grouping {
        //             expression: Box::new(expr),
        //         })
        //     }
        //     _ => Err(self.error(self.peek().clone(), String::from("Expected expression."))),
        // }
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

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek().clone(), message))
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
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> Token {
        println!("{}", self.current);
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn error(&self, token: Token, message: String) -> ParseError {
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
