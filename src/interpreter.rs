use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    class::Class,
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
            // If an exception occurs we still need to restore to the previous environment.
            // This mimicks Java's try-finally logic
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

    fn visit_class_stnt(&self, name: &Token, methods: &Vec<Stmt>) -> Result<()> {
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Value::Nil);

        let mut runtime_methods = HashMap::new();
        for method in methods {
            match method {
                Stmt::Function { name, .. } => {
                    let function = Function::new(method.clone(), self.environment.clone());
                    runtime_methods.insert(name.lexeme.clone(), function);
                }
                _ => panic!("Statement is not a method!"),
            }
        }

        let class = Class::new(name.lexeme.clone(), runtime_methods);

        self.environment
            .borrow_mut()
            .assign(name, &Value::Class(class))?;

        Ok(())
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
            Value::Class(callee) => callee.call(self, vec![]),
            _ => Exception::runtime_error(
                paren.clone(),
                String::from("Can only call functions and classes."),
            ),
        }
    }

    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<Value> {
        let object = self.evaluate(object)?;
        match object {
            Value::ClassInstance(instance) => {
                // pass instance_ref in case .get() needs to bind a method to 'this'
                let instance_ref = instance.clone();
                instance.borrow().get(name, instance_ref)
            }
            _ => Exception::runtime_error(
                name.clone(),
                String::from("Only instances have properties."),
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

    fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<Value> {
        let object = self.evaluate(object)?;
        match object {
            Value::ClassInstance(instance) => {
                let value = self.evaluate(value)?;
                instance.borrow_mut().set(name, value.clone());
                Ok(value)
            }
            _ => {
                Exception::runtime_error(name.clone(), String::from("Only instances have fields."))
            }
        }
    }

    fn visit_this_expr(&mut self, expr: &Expr, keyword: &Token) -> Result<Value> {
        self.lookup_variable(keyword, expr)
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
            Expr::Get { object, name, .. } => self.visit_get_expr(object, name),
            Expr::Set {
                object,
                name,
                value,
                ..
            } => self.visit_set_expr(object, name, value),
            Expr::This { keyword, .. } => self.visit_this_expr(expr, keyword),
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
            Stmt::Class { name, methods } => self.visit_class_stnt(name, methods),
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

    fn assert_prints(file_path: &str, expected_prints: &[String]) {
        let lox_code = fs::read_to_string(file_path).expect("file to be readable");
        let logger = Box::new(MockLogger::new());
        let logs = logger.logs.clone();
        execute_code(lox_code, logger);

        assert_eq!(expected_prints.len(), logs.borrow().len());
        for (index, log) in logs.borrow().iter().enumerate() {
            assert_eq!(log.to_owned(), expected_prints[index]);
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
        assert_prints(
            "test_files/declaration_and_assignment.lox",
            &[String::from("1"), String::from("25.1")],
        )
    }

    #[test]
    fn expression_evaluation() {
        assert_prints("test_files/expression_eval.lox", &[String::from("15")])
    }

    #[test]
    fn variable_scoping() {
        assert_prints(
            "test_files/variable_scoping.lox",
            &[
                String::from("inner a"),
                String::from("outer b"),
                String::from("global c"),
                String::from("outer a"),
                String::from("outer b"),
                String::from("global c"),
                String::from("global a"),
                String::from("global b"),
                String::from("global c"),
            ],
        )
    }

    #[test]
    fn loops() {
        assert_prints(
            "test_files/loops.lox",
            &[
                String::from("0"),
                String::from("1"),
                String::from("2"),
                String::from("3"),
                String::from("4"),
            ],
        )
    }

    #[test]
    fn function_calls() {
        assert_prints(
            "test_files/function_calls.lox",
            &[String::from("Hi, Dear Reader!")],
        )
    }

    #[test]
    fn recursive_functions() {
        assert_prints(
            "test_files/recursive_functions.lox",
            &[
                String::from("0"),
                String::from("1"),
                String::from("1"),
                String::from("2"),
                String::from("3"),
                String::from("5"),
                String::from("8"),
                String::from("13"),
                String::from("21"),
                String::from("34"),
                String::from("55"),
                String::from("89"),
                String::from("144"),
                String::from("233"),
                String::from("377"),
                String::from("610"),
                String::from("987"),
                String::from("1597"),
                String::from("2584"),
                String::from("4181"),
            ],
        )
    }

    #[test]
    fn closures() {
        assert_prints("test_files/closures.lox", &[String::from("1")])
    }

    #[test]
    fn print_class_name() {
        assert_prints(
            "test_files/print_class_name.lox",
            &[String::from("DevonshireCream")],
        )
    }

    #[test]
    fn print_class_instance() {
        assert_prints(
            "test_files/print_class_instance.lox",
            &[String::from("Bagel instance")],
        )
    }

    #[test]
    fn basic_method() {
        assert_prints(
            "test_files/basic_method.lox",
            &[String::from("inside method")],
        );
    }

    #[test]
    fn bound_methods() {
        assert_prints(
            "test_files/bound_methods.lox",
            &[
                String::from("chocolate"),
                String::from("chocolate"),
                String::from("vanilla"),
                String::from("vanilla"),
                String::from("strawberry"),
                String::from("strawberry"),
            ],
        )
    }

    #[test]
    fn init_class() {
        assert_prints("test_files/init_class.lox", &[String::from("hello!")])
    }
}
