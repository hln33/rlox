use crate::{expr::Expr, scanner::Token};

pub trait Visitor<T> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> T;
}

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}
