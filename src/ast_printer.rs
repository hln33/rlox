use crate::{
    expr::{Expr, Visitor},
    scanner::{Literal, Token},
};

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }

    fn parenthesize(&self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut string = String::from("(");
        string.push_str(name);

        for expr in exprs {
            string.push(' ');
            string.push_str(&self.visit_expr(expr));
        }

        string.push(')');
        string
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Grouping { expression } => self.parenthesize("group", vec![expression]),
            Expr::Unary { operator, right } => self.parenthesize(&operator.lexeme, vec![right]),
            Expr::Literal { value } => match value {
                Literal::Number(value) => value.to_string(),
                Literal::String(value) => value.to_string(),
                Literal::Bool(value) => value.to_string(),
                Literal::None => String::from("nil"),
            },
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![left, right]),
            Expr::Variable { name } => todo!(),
        }
    }
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
