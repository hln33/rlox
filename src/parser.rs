use crate::{
    expr::Expr,
    print_error,
    scanner::{Literal, Token, TokenType},
    stmt::Stmt,
};

#[derive(Debug)]
struct ParseError;

type Result<T> = std::result::Result<T, ParseError>;

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

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.match_token(&[TokenType::Fun]) {
            self.function("function")
        } else if self.match_token(&[TokenType::Var]) {
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

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()));
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.match_token(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = match self.check(&TokenType::Semicolon) {
            true => None,
            false => Some(self.expression()?),
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = match self.check(&TokenType::RightParen) {
            true => None,
            false => Some(self.expression()?),
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)])
        }

        if condition.is_none() {
            condition.replace(Expr::Literal {
                value: Literal::Bool(true),
            });
        }
        body = Stmt::While {
            condition: Box::new(condition.unwrap()),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = match self.match_token(&[TokenType::Else]) {
            true => Some(Box::new(self.statement()?)),
            false => None,
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let keyword = self.previous();

        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return {
            name: keyword,
            value: value.map(Box::new),
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let mut initializer = None;
        if self.match_token(&[TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after ' while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(value))
    }

    fn function(&mut self, kind: &str) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;

        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut params = vec![];
        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(
                        self.error(self.peek().clone(), "Can't have more than 255 parameters.")
                    );
                }
                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block();

        Ok(Stmt::Function { name, params, body })
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let Some(decl) = self.declaration() {
                statements.push(decl);
            }
        }

        let _ = self.consume(TokenType::RightBrace, "Expect '}' after block.");
        statements
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

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

    fn or(&mut self) -> Result<Expr> {
        self.parse_logical_op(&[TokenType::Or], |parser| parser.and())
    }

    fn and(&mut self) -> Result<Expr> {
        self.parse_logical_op(&[TokenType::And], |parser| parser.equality())
    }

    fn equality(&mut self) -> Result<Expr> {
        self.parse_binary_op(&[TokenType::BangEqual, TokenType::EqualEqual], |parser| {
            parser.comparison()
        })
    }

    fn comparison(&mut self) -> Result<Expr> {
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

    fn term(&mut self) -> Result<Expr> {
        self.parse_binary_op(&[TokenType::Minus, TokenType::Plus], |parser| {
            parser.factor()
        })
    }

    fn factor(&mut self) -> Result<Expr> {
        self.parse_binary_op(&[TokenType::Slash, TokenType::Star], |parser| {
            parser.unary()
        })
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut args = vec![];

        if !self.check(&TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    self.error(self.peek().clone(), "Can't have more than 255 arguments.");
                }
                args.push(self.expression()?);

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                };
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            args,
        })
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
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
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;
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
    ) -> Result<Expr>
    where
        F: FnMut(&mut Self) -> Result<Expr>,
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

    fn parse_logical_op<F>(
        &mut self,
        operators: &[TokenType],
        mut parse_next_level: F,
    ) -> Result<Expr>
    where
        F: FnMut(&mut Self) -> Result<Expr>,
    {
        let mut expr = parse_next_level(self)?;

        while self.match_token(operators) {
            let operator = self.previous();
            let right = parse_next_level(self)?;
            expr = Expr::Logical {
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

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek().clone(), message))
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
