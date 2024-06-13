use crate::{
    ast_printer::AstPrinter,
    scanner::{Literal, Token},
};

pub trait Visitor<T> {
    fn visit_expr(&self, expression: &Expr) -> T;
}

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

pub fn test_ast_print() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token {
                token_type: crate::scanner::TokenType::Minus,
                lexeme: String::from("-"),
                literal: Literal::None,
                line: 1,
            },
            right: Box::new(Expr::Literal {
                value: Literal::Number(123.0),
            }),
        }),
        operator: Token {
            token_type: crate::scanner::TokenType::Star,
            lexeme: String::from("*"),
            literal: Literal::None,
            line: 1,
        },
        right: Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                value: Literal::Number(45.67),
            }),
        }),
    };

    let printer = AstPrinter {};
    println!("{}", printer.print(&expression));
}
