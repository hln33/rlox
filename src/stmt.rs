use crate::expr::Expr;

pub trait Visitor<T> {
    fn visit_stmt(&self, stmt: &Stmt) -> T;
}

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
}
