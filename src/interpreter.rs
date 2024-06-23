use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    environment::{EnvRef, Environment},
    expr::{self, Expr},
    logger::{Logger, StdoutLogger},
    scanner::{Literal, Token, TokenType},
    stmt::{self, Stmt},
    value::{Callable, Function, Value},
    RuntimeError,
};

type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    globals: EnvRef,
    environment: EnvRef,
    logger: Box<dyn Logger>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Environment::new_global();

        globals.borrow_mut().define(
            "clock".to_string(),
            Value::Function(Function {
                arity: 0,
                callable: |_, _| {
                    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    Value::Number(timestamp.as_millis() as f64)
                },
            }),
        );

        let environment = globals.clone();

        Interpreter {
            globals,
            environment,
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

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr::Visitor::visit_expr(self, expr)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>, environment: EnvRef) -> Result<()> {
        let previous = self.environment.clone();

        self.environment = environment;

        for statement in statements {
            let _ = self.execute(statement);
        }

        self.environment = previous;
        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        let local_env = Environment::new_local(self.environment.clone());
        self.execute_block(statements, local_env)
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr).map(|_| ())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        if Interpreter::is_truthy(&self.evaluate(condition)?) {
            return self.execute(then_branch);
        }
        match else_branch {
            Some(else_branch) => self.execute(else_branch),
            None => Ok(()),
        }
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<()> {
        let value = self.evaluate(expr)?;
        self.logger.print(format_args!("{}", value));

        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<()> {
        let mut value = Value::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(expr)?;
        }

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        while Interpreter::is_truthy(&self.evaluate(condition)?) {
            self.execute(body)?;
        }

        Ok(())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Value> {
        let value = self.evaluate(value)?;

        match self.environment.borrow_mut().assign(name, value.clone()) {
            Ok(_) => Ok(value),
            Err(e) => Err(e),
        }
    }

    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
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

    fn visit_call_expr(&mut self, callee: &Expr, paren: &Token, args: &Vec<Expr>) -> Result<Value> {
        let callee = self.evaluate(callee)?;

        let mut evaluated_args = vec![];
        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }

        match callee {
            Value::Function(callee) => {
                if evaluated_args.len() > callee.arity() {
                    return Err(RuntimeError {
                        token: paren.clone(),
                        message: format!(
                            "Expected {} arguments but got {}.",
                            callee.arity(),
                            evaluated_args.len()
                        ),
                    });
                }
                Ok(callee.call(self, evaluated_args))
            }
            _ => Err(RuntimeError {
                token: paren.clone(),
                message: String::from("Can only call functions and classes."),
            }),
        }
    }

    fn visit_literal_expr(&self, literal: &Literal) -> Value {
        match literal {
            Literal::String(value) => Value::String(value.clone()),
            Literal::Number(value) => Value::Number(*value),
            Literal::Bool(value) => Value::Boolean(*value),
            Literal::None => Value::Nil,
        }
    }

    fn visit_logical_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
        let left = self.evaluate(left)?;

        if operator.token_type == TokenType::Or {
            if Interpreter::is_truthy(&left) {
                return Ok(left);
            }
        } else if !Interpreter::is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate(right)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Value> {
        let right_expr = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => match right_expr {
                Value::Number(value) => Ok(Value::Number(-value)),
                _ => Err(Interpreter::number_operand_error(operator)),
            },
            TokenType::Bang => Ok(Value::Boolean(!Interpreter::is_truthy(&right_expr))),
            _ => Err(Interpreter::number_operand_error(operator)),
        }
    }

    fn visit_var_expr(&self, name: &Token) -> Result<Value> {
        self.environment.borrow().get(name)
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

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(value) => *value,
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

impl expr::Visitor<Result<Value>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Literal { value } => Ok(self.visit_literal_expr(value)),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Expr::Variable { name } => self.visit_var_expr(name),
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.visit_logical_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                args,
            } => self.visit_call_expr(callee, paren, args),
        }
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expr_stmt(expr),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Block(statements) => self.visit_block_stmt(statements),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, fmt::Arguments, rc::Rc, vec};

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
