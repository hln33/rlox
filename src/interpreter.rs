use std::fmt::Display;

use crate::{
    environment::Environment,
    expr::{self, Expr},
    scanner::{Literal, Token, TokenType},
    stmt::{self, Stmt},
    RuntimeError,
};

#[derive(Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s;
        match self {
            Value::Boolean(value) => s = value.to_string(),
            Value::Number(value) => {
                s = value.to_string();
                if s.ends_with(".0") {
                    s = s.strip_suffix(".0").unwrap().to_string();
                }
            }
            Value::String(value) => s = value.clone(),
            Value::Nil => s = String::from("nil"),
        }

        write!(f, "{}", s)
    }
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            match self.execute(&statement) {
                Ok(_) => (),
                Err(e) => e.error(),
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        expr::Visitor::visit_expr(self, expr)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    fn visit_expr_stmt(&self, expr: &Expr) -> Result<(), RuntimeError> {
        self.evaluate(expr).map(|_| ())
    }

    fn visit_print_stmt(&self, expr: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(
        &mut self,
        name: &Token,
        initializer: &Option<Expr>,
    ) -> Result<(), RuntimeError> {
        let mut value = Value::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(expr)?;
        }

        self.environment.define(name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_binary(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            // arithmetic
            TokenType::Minus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                _ => Err(Interpreter::number_operands_error(operator)),
            },
            TokenType::Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Err(Interpreter::number_operands_error(operator)),
            },
            TokenType::Star => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Err(Interpreter::number_operands_error(operator)),
            },
            TokenType::Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => {
                    let mut res = left.to_owned();
                    res.push_str(&right);
                    Ok(Value::String(res))
                }
                _ => Err(Interpreter::number_operands_error(operator)),
            },

            // comparison
            TokenType::Greater => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
                _ => Err(Interpreter::number_operands_error(operator)),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
                _ => Err(Interpreter::number_operands_error(operator)),
            },
            TokenType::Less => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
                _ => Err(Interpreter::number_operands_error(operator)),
            },
            TokenType::LessEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
                _ => Err(Interpreter::number_operands_error(operator)),
            },

            // equality
            TokenType::BangEqual => Ok(Value::Boolean(!Interpreter::is_equal(left, right))),
            TokenType::Equal => Ok(Value::Boolean(Interpreter::is_equal(left, right))),

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

    fn visit_unary(&self, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        let right_expr = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => match right_expr {
                Value::Number(value) => Ok(Value::Number(-value)),
                _ => Err(Interpreter::number_operand_error(operator)),
            },
            TokenType::Bang => Ok(Value::Boolean(!Interpreter::is_truthy(right_expr))),
            _ => Err(Interpreter::number_operand_error(operator)),
        }
    }

    fn visit_var_expr(&self, name: &Token) -> Result<Value, RuntimeError> {
        self.environment.get(name)
    }

    fn number_operand_error(operator: &Token) -> RuntimeError {
        RuntimeError {
            token: operator.clone(),
            message: String::from("Operand must be a number."),
        }
    }

    fn number_operands_error(operator: &Token) -> RuntimeError {
        RuntimeError {
            token: operator.clone(),
            message: String::from("Operands must be a numbers."),
        }
    }

    fn is_truthy(value: Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(value) => value,
            _ => true,
        }
    }

    fn is_equal(left: Value, right: Value) -> bool {
        match (left, right) {
            (Value::Nil, Value::Nil) => true,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            _ => false,
        }
    }
}

impl expr::Visitor<Result<Value, RuntimeError>> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Literal { value } => Ok(self.visit_literal(value)),
            Expr::Unary { operator, right } => self.visit_unary(operator, right),
            Expr::Variable { name } => self.visit_var_expr(name),
        }
    }
}

impl stmt::Visitor<Result<(), RuntimeError>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expr_stmt(expr),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
        }
    }
}
