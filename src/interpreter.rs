use std::{
    cell::RefCell,
    fmt::{Arguments, Display},
    rc::Rc,
};

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

trait Logger {
    fn print(&mut self, value: Arguments);
}

struct StdoutLogger;

impl Logger for StdoutLogger {
    fn print(&mut self, value: Arguments) {
        println!("{}", value)
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    logger: Box<dyn Logger>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new_global())),
            logger: Box::new(StdoutLogger),
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

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        expr::Visitor::visit_expr(self, expr)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        let previous = self.environment.clone();

        self.environment = environment;

        for statement in statements {
            let _ = self.execute(statement);
        }

        self.environment = previous;
        Ok(())
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        // let value = self.evaluate(expr);
        // match value {
        //     Ok(value) => println!("{}", value),
        //     Err(_) => todo!(),
        // }

        self.evaluate(expr).map(|_| ())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expr)?;
        // println!("{}", value);
        self.logger.print(format_args!("{}", value));
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

        // !!!!!!!!!
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);
        // self.environment.define(name.lexeme.clone(), value);

        Ok(())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Value, RuntimeError> {
        let value = self.evaluate(value)?;

        // !!!!!!!!!!!!!!!!!!!!!
        match self.environment.borrow_mut().assign(name, value.clone()) {
            Ok(_) => Ok(value),
            Err(e) => Err(e),
        }
        // match self.environment.assign(name, value.clone()) {
        //     Ok(_) => Ok(value),
        //     Err(e) => Err(e),
        // }
    }

    fn visit_binary(
        &mut self,
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

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
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
        // !!!!!!!!!!!!!!!!!!!!!!!!!
        self.environment.borrow().get(name)
        // self.environment.get(name)
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
    fn visit_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
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
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
        }
    }
}

impl stmt::Visitor<Result<(), RuntimeError>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expr_stmt(expr),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Block(statements) => {
                let env_ref = self.environment.clone();

                let enclosing = Some(env_ref);

                let local_env = Rc::new(RefCell::new(Environment::new_local(enclosing)));

                self.execute_block(statements, local_env)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    struct MockLogger {
        logs: Rc<RefCell<Vec<String>>>,
    }
    impl MockLogger {
        fn new() -> MockLogger {
            MockLogger {
                logs: Rc::new(RefCell::new(vec![])),
            }
        }
    }
    impl Logger for MockLogger {
        fn print(&mut self, value: Arguments) {
            self.logs.borrow_mut().push(value.to_string());
        }
    }

    #[test]
    fn variable_declaration_and_assignment() {
        let mut interpreter = Interpreter::new();
        let variable_token = Token {
            token_type: TokenType::Identifier,
            lexeme: "x".to_string(),
            literal: Literal::None,
            line: 1,
        };

        // declaration
        let var_stmt = Stmt::Var {
            name: variable_token.clone(),
            initializer: Some(Expr::Literal {
                value: Literal::Number(10.0),
            }),
        };
        assert!(interpreter.execute(&var_stmt).is_ok());

        let result = interpreter.environment.borrow().get(&variable_token);
        assert_eq!(result.unwrap().to_string(), "10");

        // assignment
        let assign_stmt = Stmt::Expression(Expr::Assign {
            name: variable_token.clone(),
            value: Box::new(Expr::Literal {
                value: Literal::Number(20.5),
            }),
        });
        assert!(interpreter.execute(&assign_stmt).is_ok());

        // check that variable got updated
        let result = interpreter.environment.borrow().get(&variable_token);
        assert_eq!(result.unwrap().to_string(), "20.5");
    }

    #[test]
    fn expression_evaluation() {
        let mut interpreter = Interpreter::new();

        // Assuming you have defined a helper function or builder for creating expressions
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(10.0),
            }),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: Literal::None,
                line: 1,
            },
            right: Box::new(Expr::Literal {
                value: Literal::Number(5.0),
            }),
        };

        let value = interpreter.evaluate(&expr).unwrap();
        assert_eq!(value.to_string(), "15");
    }

    #[test]
    fn block_execution() {
        let mut interpreter = Interpreter::new();

        let stmts = vec![
            Stmt::Var {
                name: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "x".to_string(),
                    literal: Literal::None,
                    line: 1,
                },
                initializer: Some(Expr::Literal {
                    value: Literal::Number(10.0),
                }),
            },
            Stmt::Var {
                name: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "x".to_string(),
                    literal: Literal::None,
                    line: 1,
                },
                initializer: Some(Expr::Literal {
                    value: Literal::Number(20.0),
                }),
            },
            Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Variable {
                    name: Token {
                        token_type: TokenType::Identifier,
                        lexeme: "x".to_string(),
                        literal: Literal::None,
                        line: 3,
                    },
                }),
                operator: Token {
                    token_type: TokenType::Plus,
                    lexeme: "+".to_string(),
                    literal: Literal::None,
                    line: 3,
                },
                right: Box::new(Expr::Variable {
                    name: Token {
                        token_type: TokenType::Identifier,
                        lexeme: "x".to_string(),
                        literal: Literal::None,
                        line: 1,
                    },
                }),
            }),
        ];

        assert!(interpreter
            .execute_block(&stmts, interpreter.environment.clone())
            .is_ok());
    }

    #[test]
    fn variable_scoping() {
        let logger = Box::new(MockLogger::new());
        let logs = logger.logs.clone();

        let mut interpreter = Interpreter::new();
        interpreter.logger = logger;

        // these are used for print statements, the line number should not matter
        let a = Token {
            token_type: TokenType::Identifier,
            lexeme: "a".to_string(),
            literal: Literal::None,
            line: 9999999,
        };
        let b = Token {
            token_type: TokenType::Identifier,
            lexeme: "b".to_string(),
            literal: Literal::None,
            line: 9999999,
        };
        let c = Token {
            token_type: TokenType::Identifier,
            lexeme: "c".to_string(),
            literal: Literal::None,
            line: 9999999,
        };

        let statements = vec![
            Stmt::Var {
                name: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "a".to_string(),
                    literal: Literal::None,
                    line: 1,
                },
                initializer: Some(Expr::Literal {
                    value: Literal::String(String::from("global a")),
                }),
            },
            Stmt::Var {
                name: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "b".to_string(),
                    literal: Literal::None,
                    line: 2,
                },
                initializer: Some(Expr::Literal {
                    value: Literal::String(String::from("global b")),
                }),
            },
            Stmt::Var {
                name: Token {
                    token_type: TokenType::Identifier,
                    lexeme: "c".to_string(),
                    literal: Literal::None,
                    line: 3,
                },
                initializer: Some(Expr::Literal {
                    value: Literal::String(String::from("global c")),
                }),
            },
            Stmt::Block(vec![
                Stmt::Var {
                    name: Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a".to_string(),
                        literal: Literal::None,
                        line: 5,
                    },
                    initializer: Some(Expr::Literal {
                        value: Literal::String(String::from("outer a")),
                    }),
                },
                Stmt::Var {
                    name: Token {
                        token_type: TokenType::Identifier,
                        lexeme: "b".to_string(),
                        literal: Literal::None,
                        line: 6,
                    },
                    initializer: Some(Expr::Literal {
                        value: Literal::String(String::from("outer b")),
                    }),
                },
                Stmt::Block(vec![
                    Stmt::Var {
                        name: Token {
                            token_type: TokenType::Identifier,
                            lexeme: "a".to_string(),
                            literal: Literal::None,
                            line: 8,
                        },
                        initializer: Some(Expr::Literal {
                            value: Literal::String(String::from("inner a")),
                        }),
                    },
                    Stmt::Print(Expr::Variable { name: a.clone() }),
                    Stmt::Print(Expr::Variable { name: b.clone() }),
                    Stmt::Print(Expr::Variable { name: c.clone() }),
                ]),
                Stmt::Print(Expr::Variable { name: a.clone() }),
                Stmt::Print(Expr::Variable { name: b.clone() }),
                Stmt::Print(Expr::Variable { name: c.clone() }),
            ]),
            Stmt::Print(Expr::Variable { name: a.clone() }),
            Stmt::Print(Expr::Variable { name: b.clone() }),
            Stmt::Print(Expr::Variable { name: c.clone() }),
        ];

        interpreter.interpret(statements);

        let expected_logs = vec![
            String::from("inner a"),
            String::from("outer b"),
            String::from("global c"),
            String::from("outer a"),
            String::from("outer b"),
            String::from("global c"),
            String::from("global a"),
            String::from("global b"),
            String::from("global c"),
        ];
        for (index, log) in logs.borrow().iter().enumerate() {
            assert_eq!(log.to_owned(), expected_logs[index]);
        }
    }
}
