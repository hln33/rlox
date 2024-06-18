use crate::{
    expr::{Expr, Visitor},
    scanner::{Literal, Token, TokenType},
};

enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Nil,
}

pub struct Interpreter {}

impl Interpreter {
    fn evaluate(&self, expr: &Expr) -> Value {
        self.visit_expr(expr)
    }

    fn visit_binary(&self, left: &Expr, operator: &Token, right: &Expr) -> Value {
        let left_expr = self.evaluate(left);
        let right_expr = self.evaluate(right);

        match operator.token_type {
            // arithmetic
            TokenType::Minus => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Number(left - right),
                _ => panic!("unexpected values for minus operation"),
            },
            TokenType::Slash => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Number(left / right),
                _ => panic!("unexpected values for division operation"),
            },
            TokenType::Star => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Number(left * right),
                _ => panic!("unexpected values for multiplication operation"),
            },
            TokenType::Plus => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Number(left + right),
                (Value::String(left), Value::String(right)) => {
                    let mut res = left.to_owned();
                    res.push_str(&right);
                    Value::String(res)
                }
                _ => panic!("unexpected values for plus operation"),
            },

            // comparison
            TokenType::Greater => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Boolean(left > right),
                _ => panic!("unexpected values for greater than operation"),
            },
            TokenType::GreaterEqual => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Boolean(left >= right),
                _ => panic!("unexpected values for greater than or equal operation"),
            },
            TokenType::Less => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Boolean(left < right),
                _ => panic!("unexpected values for less than operation"),
            },
            TokenType::LessEqual => match (left_expr, right_expr) {
                (Value::Number(left), Value::Number(right)) => Value::Boolean(left <= right),
                _ => panic!("unexpected values for less than or equal operation"),
            },

            // equality
            TokenType::BangEqual => Value::Boolean(!self.is_equal(left_expr, right_expr)),
            TokenType::Equal => Value::Boolean(self.is_equal(left_expr, right_expr)),

            _ => panic!("unexpected operator for binary expression"),
        }
    }

    fn visit_literal(&self, literal: &Literal) -> Value {
        match literal {
            Literal::String(value) => Value::String(value.clone()),
            Literal::Number(value) => Value::Number(*value),
            Literal::Bool(value) => Value::Boolean(*value),
            Literal::None => Value::Nil,
        }
    }

    fn visit_unary(&self, operator: &Token, right: &Expr) -> Value {
        let right_expr = self.evaluate(right);

        match operator.token_type {
            TokenType::Minus => match right_expr {
                Value::Number(value) => Value::Number(-value),
                _ => panic!("expected number for right expression"),
            },
            TokenType::Bang => Value::Boolean(!self.is_truthy(right_expr)),
            _ => panic!("unexpected operator for unary expression"),
        }
    }

    fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(value) => value,
            _ => true,
        }
    }

    fn is_equal(&self, left: Value, right: Value) -> bool {
        match (left, right) {
            (Value::Nil, Value::Nil) => true,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            _ => false,
        }
    }
}

impl Visitor<Value> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Literal { value } => self.visit_literal(value),
            Expr::Unary { operator, right } => self.visit_unary(operator, right),
        }
    }
}
