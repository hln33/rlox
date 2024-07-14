use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::{EnvRef, Environment},
    impls::{
        class::Class,
        function::{Callable, Function, NativeFunction},
    },
    syntax::{
        expr::{self, Expr},
        stmt::{self, Stmt},
        token::{Literal, Token, TokenType},
        value::Value,
    },
    utils::logger::{Logger, StdoutLogger},
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
    pub fn new(logger: Option<Box<dyn Logger>>) -> Interpreter {
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
        let logger = if let Some(logger) = logger {
            logger
        } else {
            Box::new(StdoutLogger)
        };

        Interpreter {
            globals,
            environment,
            locals: HashMap::new(),
            logger,
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

    fn evaluate_super_class(
        &mut self,
        class_name: &Token,
        super_class_expr: &Expr,
    ) -> Result<Class> {
        let evaluated = self.evaluate(super_class_expr)?;
        match evaluated {
            Value::Class(class) => Ok(class),
            _ => Exception::runtime_error(
                class_name.clone(),
                String::from("Superclass must be a class"),
            ),
        }
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        let local_env = Environment::new_local(&self.environment);
        self.execute_block(statements, local_env)
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        super_class: &Option<Box<Expr>>,
        methods: &Vec<Stmt>,
    ) -> Result<()> {
        let super_class = if let Some(super_class_expr) = super_class {
            let class = self.evaluate_super_class(name, super_class_expr)?;
            Some(Box::new(class))
        } else {
            None
        };

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Value::Nil);

        let prev_environment = self.environment.clone();
        if let Some(super_class) = super_class.clone() {
            self.environment = Environment::new_local(&self.environment);
            self.environment
                .borrow_mut()
                .define(String::from("super"), Value::Class(*super_class));
        }

        let mut runtime_methods = HashMap::new();
        for method in methods {
            match method {
                Stmt::Function { name, .. } => {
                    let function = Function::new(
                        method.clone(),
                        self.environment.clone(),
                        name.lexeme == "init",
                    );
                    runtime_methods.insert(name.lexeme.clone(), function);
                }
                _ => panic!("Statement is not a method!"),
            }
        }

        let class = Class::new(name.lexeme.clone(), super_class.clone(), runtime_methods);

        if super_class.is_some() {
            self.environment = prev_environment;
        }

        self.environment
            .borrow_mut()
            .assign(name, &Value::Class(class))?;

        Ok(())
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr).map(|_| ())
    }

    fn visit_function_stmt(&mut self, name: &Token, function_stmt: &Stmt) -> Result<()> {
        let function = Function::new(function_stmt.clone(), self.environment.clone(), false);
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

    fn visit_super_expr(&mut self, expr: &Expr, method: &Token) -> Result<Value> {
        let distance = self
            .locals
            .get(expr)
            .expect("Super class to have been resolved");

        let super_class = self
            .environment
            .borrow()
            .get_at(*distance, "super")
            .expect("'super' to have been resolved");

        let super_class = match super_class {
            Value::Class(super_class) => super_class,
            _ => panic!("Expected superclass to be a class!"),
        };

        let method = super_class.find_method(&method.lexeme).ok_or_else(|| {
            Exception::runtime_error::<()>(
                method.clone(),
                format!("Undefined property {}.", method.lexeme),
            )
            .unwrap_err()
        })?;

        let this = self
            .environment
            .borrow()
            // "this" is always right inside where "super" is stored
            .get_at(*distance - 1, "this")
            .expect("'this' to have been resolved");
        match method {
            Value::Function(method) => match this {
                Value::ClassInstance(instance) => Ok(Value::Function(method.bind(instance))),
                _ => panic!("Expected 'this' to be a class instance!"),
            },
            _ => panic!("Expected method to be a function!"),
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
            Expr::Super { method, .. } => self.visit_super_expr(expr, method),
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
            Stmt::Class {
                name,
                super_class,
                methods,
            } => self.visit_class_stmt(name, super_class, methods),
        }
    }
}
