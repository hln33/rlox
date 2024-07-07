use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::{EnvRef, Environment},
    expr::{self, Expr},
    function::{Callable, Function, NativeFunction},
    logger::{Logger, StdoutLogger},
    scanner::{Literal, Token, TokenType},
    stmt::{self, Stmt},
    value::Value,
    Exception,
};

type Result<T> = std::result::Result<T, Exception>;

pub struct Interpreter {
    pub globals: EnvRef,
    environment: EnvRef,
    locals: HashMap<Expr, usize>,
    logger: Box<dyn Logger>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Environment::new_global();
        globals.borrow_mut().define(
            "clock".to_string(),
            Value::NativeFunction(NativeFunction {
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
            locals: HashMap::new(),
            logger: Box::new(StdoutLogger),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            match self.execute(&statement) {
                Ok(_) => (),
                Err(e) => match e {
                    Exception::RuntimeError(e) => e.error(),
                    Exception::Return(_) => panic!("Return statement not handled!"),
                },
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr::Visitor::visit_expr(self, expr)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, environment: EnvRef) -> Result<()> {
        let previous = self.environment.clone();

        self.environment = environment;
        for statement in statements {
            if let Err(e) = self.execute(statement) {
                self.environment = previous;
                return Err(e);
            }
        }

        self.environment = previous;
        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        let local_env = Environment::new_local(&self.environment);
        self.execute_block(statements, local_env)
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr).map(|_| ())
    }

    fn visit_function_stmt(&mut self, name: &Token, function_stmt: &Stmt) -> Result<()> {
        let function = Function::new(function_stmt.clone(), self.environment.clone());
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Value::Function(function));
        Ok(())
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

    fn visit_return_stmt(&mut self, value: &Option<Box<Expr>>) -> Result<()> {
        match value {
            Some(value) => Err(Exception::Return(self.evaluate(value)?)),
            None => Err(Exception::Return(Value::Nil)),
        }
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

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr, expr: &Expr) -> Result<Value> {
        let value = self.evaluate(value)?;

        let distance = self.locals.get(expr);
        match distance {
            Some(distance) => self
                .environment
                .borrow_mut()
                .assign_at(*distance, name, &value),
            None => self.globals.borrow_mut().assign(name, &value)?,
        };

        Ok(value)
    }

    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            // arithmetic
            TokenType::Minus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Star => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => {
                    let mut res = left.to_owned();
                    res.push_str(&right);
                    Ok(Value::String(res))
                }
                _ => Interpreter::number_operands_error(operator),
            },

            // comparison
            TokenType::Greater => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Less => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::LessEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
                _ => Interpreter::number_operands_error(operator),
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
                callee.check_arity(evaluated_args.len(), paren)?;
                callee.call(self, evaluated_args)
            }
            Value::NativeFunction(callee) => {
                callee.check_arity(evaluated_args.len(), paren)?;
                callee.call(self, evaluated_args)
            }
            _ => Exception::runtime_error(
                paren.clone(),
                String::from("Can only call functions and classes."),
            ),
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
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::Bang => Ok(Value::Boolean(!Interpreter::is_truthy(&right_expr))),
            _ => Interpreter::number_operand_error(operator),
        }
    }

    fn visit_var_expr(&self, name: &Token, expr: &Expr) -> Result<Value> {
        self.lookup_variable(name, expr)
    }

    fn lookup_variable(&self, name: &Token, expr: &Expr) -> Result<Value> {
        let distance = self.locals.get(expr);

        if let Some(distance) = distance {
            self.environment.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.borrow().get(name)
        }
    }

    fn number_operand_error<T>(operator: &Token) -> Result<T> {
        Exception::runtime_error(operator.clone(), String::from("Operands must be a number."))
    }

    fn number_operands_error<T>(operator: &Token) -> Result<T> {
        Exception::runtime_error(operator.clone(), String::from("Operands must be numbers."))
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
                ..
            } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression, .. } => self.evaluate(expression),
            Expr::Literal { value, .. } => Ok(self.visit_literal_expr(value)),
            Expr::Unary {
                operator, right, ..
            } => self.visit_unary_expr(operator, right),
            Expr::Variable { name, .. } => self.visit_var_expr(name, expr),
            Expr::Assign { name, value, .. } => self.visit_assign_expr(name, value, expr),
            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => self.visit_logical_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                args,
                ..
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
            Stmt::Function { name, .. } => self.visit_function_stmt(name, stmt),
            Stmt::Return { value, .. } => self.visit_return_stmt(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, env, fmt::Arguments, fs, rc::Rc, vec};

    use crate::{parser::Parser, resolver::Resolver, scanner::Scanner};

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

    fn execute_code(lox_code: String, logger: Box<MockLogger>) {
        env::set_var("RUST_BACKTRACE", "1");

        let mut interpreter = Interpreter::new();
        interpreter.logger = logger;

        let mut scanner = Scanner::new(lox_code);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        let mut resolver = Resolver::new(&mut interpreter);
        resolver.resolve_block(&statements);

        interpreter.interpret(statements);
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
                uid: 0,
                value: Literal::Number(10.0),
            }),
        };
        assert!(interpreter.execute(&var_stmt).is_ok());

        let result = interpreter.environment.borrow().get(&variable_token);
        assert_eq!(result.unwrap().to_string(), "10");

        // assignment
        let assign_stmt = Stmt::Expression(Expr::Assign {
            uid: 0,
            name: variable_token.clone(),
            value: Box::new(Expr::Literal {
                uid: 0,
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
            uid: 0,
            left: Box::new(Expr::Literal {
                uid: 0,
                value: Literal::Number(10.0),
            }),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: Literal::None,
                line: 1,
            },
            right: Box::new(Expr::Literal {
                uid: 0,
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
                    uid: 0,
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
                    uid: 0,
                    value: Literal::Number(20.0),
                }),
            },
            Stmt::Expression(Expr::Binary {
                uid: 0,
                left: Box::new(Expr::Variable {
                    uid: 0,
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
                    uid: 0,
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
        let lox_code =
            fs::read_to_string("test_files/variable_scoping.lox").expect("file to be readable");

        let logger = Box::new(MockLogger::new());
        let logs = logger.logs.clone();
        execute_code(lox_code, logger);

        let expected_logs = [
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
        assert_eq!(expected_logs.len(), logs.borrow().len());
        for (index, log) in logs.borrow().iter().enumerate() {
            assert_eq!(log.to_owned(), expected_logs[index]);
        }
    }

    #[test]
    fn loops() {
        let lox_code = fs::read_to_string("test_files/loops.lox").expect("file to be readable");

        let logger = Box::new(MockLogger::new());
        let logs = logger.logs.clone();
        execute_code(lox_code, logger);

        let expected_logs = ["0", "1", "2", "3", "4"];
        assert_eq!(expected_logs.len(), logs.borrow().len());
        for (i, log) in logs.borrow().iter().enumerate() {
            assert_eq!(log.to_owned(), expected_logs[i])
        }
    }

    #[test]
    fn function_calls() {
        let lox_code =
            fs::read_to_string("test_files/function_calls.lox").expect("file to be readable");

        let logger = Box::new(MockLogger::new());
        let logs = logger.logs.clone();
        execute_code(lox_code, logger);

        let expected_logs = ["Hi, Dear Reader!"];
        assert_eq!(expected_logs.len(), logs.borrow().len());
        for (i, log) in logs.borrow().iter().enumerate() {
            assert_eq!(log.to_owned(), expected_logs[i])
        }
    }

    #[test]
    fn recursive_functions() {
        let lox_code =
            fs::read_to_string("test_files/recursive_functions.lox").expect("file to be readable");

        let logger = Box::new(MockLogger::new());
        let logs = logger.logs.clone();
        execute_code(lox_code, logger);

        let expected_logs = [
            "0", "1", "1", "2", "3", "5", "8", "13", "21", "34", "55", "89", "144", "233", "377",
            "610", "987", "1597", "2584", "4181",
        ];
        assert_eq!(expected_logs.len(), logs.borrow().len());
        for (i, log) in logs.borrow().iter().enumerate() {
            assert_eq!(log.to_owned(), expected_logs[i])
        }
    }

    #[test]
    fn closures() {
        let lox_code = fs::read_to_string("test_files/closures.lox").expect("file to be readable");

        let logger = Box::new(MockLogger::new());
        let logs = logger.logs.clone();
        execute_code(lox_code, logger);

        let expected_logs = ["1"];
        assert_eq!(expected_logs.len(), logs.borrow().len());
        for (i, log) in logs.borrow().iter().enumerate() {
            assert_eq!(log.to_owned(), expected_logs[i])
        }
    }
}
