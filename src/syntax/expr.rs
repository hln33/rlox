use std::hash::Hash;

use super::token::{Literal, Token};

pub trait Visitor<T> {
    fn visit_expr(&mut self, expression: &Expr) -> T;
}

#[derive(Clone, Debug)]
pub enum Expr {
    Binary {
        uid: u8,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        uid: u8,
        expression: Box<Expr>,
    },
    Literal {
        uid: u8,
        value: Literal,
    },
    Unary {
        uid: u8,
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        uid: u8,
        name: Token,
    },
    Assign {
        uid: u8,
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        uid: u8,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        uid: u8,
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Get {
        uid: u8,
        object: Box<Expr>,
        name: Token,
    },
    Set {
        uid: u8,
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This {
        uid: u8,
        keyword: Token,
    },
    Super {
        uid: u8,
        keyword: Token,
        method: Token,
    },
}

impl Expr {
    fn get_uid(&self) -> u8 {
        match self {
            Expr::Binary { uid, .. } => *uid,
            Expr::Grouping { uid, .. } => *uid,
            Expr::Literal { uid, .. } => *uid,
            Expr::Unary { uid, .. } => *uid,
            Expr::Variable { uid, .. } => *uid,
            Expr::Assign { uid, .. } => *uid,
            Expr::Logical { uid, .. } => *uid,
            Expr::Call { uid, .. } => *uid,
            Expr::Get { uid, .. } => *uid,
            Expr::Set { uid, .. } => *uid,
            Expr::This { uid, .. } => *uid,
            Expr::Super { uid, .. } => *uid,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // core::mem::discriminant(self).hash(state);
        self.get_uid().hash(state);
    }
}
