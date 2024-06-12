use crate::{
    expr::{Expr, Visitor},
    scanner::Literal,
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
                Literal::None => String::from("nil"),
            },
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![left, right]),
        }
    }
}
