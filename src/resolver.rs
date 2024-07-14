use std::collections::HashMap;

use crate::{
    interpreter::Interpreter,
    print_error,
    syntax::{
        expr::{self, Expr},
        stmt::{self, Stmt},
        token::Token,
    },
    RuntimeError,
};

#[derive(Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver<'_> {
    pub fn new(interpreter: &mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_block(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        stmt::Visitor::visit_stmt(self, stmt);
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        expr::Visitor::visit_expr(self, expr);
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        function_type: FunctionType,
    ) {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        self.begin_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_block(body);

        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("stack of scopes to not be empty.");
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.peek_scopes_mut();

        if scope.contains_key(&name.lexeme) {
            RuntimeError {
                token: name.clone(),
                message: "Already a variable with this name in this scope.".to_string(),
            }
            .error();
        }

        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.peek_scopes_mut();
        scope.insert(name.lexeme.clone(), true);
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.lexeme) {
                let hops_away = self.scopes.len() - 1 - i;
                self.interpreter.resolve(expr, hops_away);
                return;
            }
        }
    }

    fn resolve_super_class(&mut self, class_name: &Token, super_class_expr: &Expr) {
        match super_class_expr {
            Expr::Variable {
                name: super_class_name,
                ..
            } => {
                if class_name.lexeme == super_class_name.lexeme {
                    RuntimeError {
                        token: super_class_name.clone(),
                        message: "A class can't inherit from itself.".to_string(),
                    }
                    .error()
                }

                self.resolve_expr(super_class_expr);
            }
            _ => panic!("super_class is not an expression!"),
        }
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) {
        self.begin_scope();
        self.resolve_block(statements);
        self.end_scope();
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        super_class: &Option<Box<Expr>>,
        methods: &Vec<Stmt>,
    ) {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(name);
        self.define(name);

        if let Some(super_class) = super_class {
            self.current_class = ClassType::Subclass;
            self.resolve_super_class(name, super_class);

            self.begin_scope();
            self.peek_scopes_mut().insert(String::from("super"), true);
        }

        self.begin_scope();
        self.peek_scopes_mut().insert(String::from("this"), true);

        for method in methods {
            match method {
                Stmt::Function { params, body, name } => {
                    let declaration = if name.lexeme == "init" {
                        FunctionType::Initializer
                    } else {
                        FunctionType::Method
                    };

                    self.resolve_function(params, body, declaration)
                }
                _ => panic!("Method is not a function!"),
            }
        }

        self.end_scope();

        if super_class.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) {
        self.resolve_expr(expr);
    }

    fn visit_function_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<Stmt>) {
        self.declare(name);
        self.define(name);

        self.resolve_function(params, body, FunctionType::Function);
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch);
        }
    }

    fn visit_print_stmt(&mut self, value: &Expr) {
        self.resolve_expr(value);
    }

    fn visit_return_stmt(&mut self, name: &Token, value: &Option<Box<Expr>>) {
        if let FunctionType::None = self.current_function {
            RuntimeError {
                token: name.clone(),
                message: "Can't return from top-level code".to_string(),
            }
            .error();
        }

        if let Some(value) = value {
            if let FunctionType::Initializer = self.current_function {
                RuntimeError {
                    token: name.clone(),
                    message: "Can't return a value from an initializer.".to_string(),
                }
                .error()
            }

            self.resolve_expr(value);
        }
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) {
        self.declare(name);

        if let Some(initializer) = initializer {
            self.resolve_expr(initializer);
        }

        self.define(name);
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) {
        self.resolve_expr(condition);
        self.resolve_stmt(body);
    }

    fn visit_assign_expr(&mut self, var_expr: &Expr, name: &Token, value: &Expr) {
        self.resolve_expr(value);
        self.resolve_local(var_expr, name);
    }

    fn visit_binary_expr(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn visit_call_expr(&mut self, callee: &Expr, args: &Vec<Expr>) {
        self.resolve_expr(callee);

        for arg in args {
            self.resolve_expr(arg);
        }
    }

    fn visit_get_expr(&mut self, object: &Expr) {
        self.resolve_expr(object);
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) {
        self.resolve_expr(expression);
    }

    fn visit_literal_expr(&self) {}

    fn visit_logical_expr(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn visit_set_expr(&mut self, object: &Expr, value: &Expr) {
        self.resolve_expr(value);
        self.resolve_expr(object);
    }

    fn visit_super_expr(&mut self, expr: &Expr, keyword: &Token) {
        match self.current_class {
            ClassType::None => print_error(
                keyword.line,
                keyword.lexeme.clone(),
                "Can't use 'super' outside of a class.",
            ),
            ClassType::Class => print_error(
                keyword.line,
                keyword.lexeme.clone(),
                "Can't use 'super' in a class with no superclass.",
            ),
            ClassType::Subclass => {}
        }

        self.resolve_local(expr, keyword);
    }

    fn visit_this_expr(&mut self, expr: &Expr, keyword: &Token) {
        if let ClassType::None = self.current_class {
            print_error(
                keyword.line,
                keyword.lexeme.clone(),
                "Can't use 'this' outside of a class.",
            );
            return;
        }

        self.resolve_local(expr, keyword);
    }

    fn visit_unary_expr(&mut self, right: &Expr) {
        self.resolve_expr(right);
    }

    fn visit_var_expr(&mut self, var_expr: &Expr, name: &Token) {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(&name.lexeme) {
                print_error(
                    name.line,
                    name.lexeme.clone(),
                    "Can't read local variable in its own initializer.",
                )
            }
        }

        self.resolve_local(var_expr, name)
    }

    fn peek_scopes_mut(&mut self) -> &mut HashMap<String, bool> {
        self.scopes
            .last_mut()
            .expect("stack of scopes to be non-empty")
    }
}

impl expr::Visitor<()> for Resolver<'_> {
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, right, .. } => self.visit_binary_expr(left, right),
            Expr::Grouping { expression, .. } => self.visit_grouping_expr(expression),
            Expr::Literal { .. } => self.visit_literal_expr(),
            Expr::Unary { right, .. } => self.visit_unary_expr(right),
            Expr::Variable { name, .. } => self.visit_var_expr(expr, name),
            Expr::Assign { name, value, .. } => self.visit_assign_expr(expr, name, value),
            Expr::Logical { left, right, .. } => self.visit_logical_expr(left, right),
            Expr::Call { callee, args, .. } => self.visit_call_expr(callee, args),
            Expr::Get { object, .. } => self.visit_get_expr(object),
            Expr::Set { object, value, .. } => self.visit_set_expr(object, value),
            Expr::This { keyword, .. } => self.visit_this_expr(expr, keyword),
            Expr::Super { keyword, .. } => self.visit_super_expr(expr, keyword),
        }
    }
}

impl stmt::Visitor<()> for Resolver<'_> {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr) => self.visit_expr_stmt(expr),
            Stmt::Print(value) => self.visit_print_stmt(value),
            Stmt::Block(statements) => self.visit_block_stmt(statements),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Function { name, params, body } => self.visit_function_stmt(name, params, body),
            Stmt::Return { name, value } => self.visit_return_stmt(name, value),
            Stmt::Class {
                name,
                super_class,
                methods,
            } => self.visit_class_stmt(name, super_class, methods),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use crate::{interpreter::Interpreter, parser::Parser, runtime_error, scanner::Scanner};

    use super::*;

    fn test_for_resolution_error(file_path: &str) {
        let lox_code = fs::read_to_string(file_path).expect("file to be readable");
        resolve_code(lox_code);

        assert!(runtime_error())
    }

    fn resolve_code(lox_code: String) {
        env::set_var("RUST_BACKTRACE", "1");

        let mut interpreter = Interpreter::new(None);

        let mut scanner = Scanner::new(lox_code);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        let mut resolver = Resolver::new(&mut interpreter);
        resolver.resolve_block(&statements);
    }

    #[test]
    fn variable_resolution_error() {
        test_for_resolution_error("test_files/variable_resolution_error.lox")
    }

    #[test]
    fn invalid_return_error() {
        test_for_resolution_error("test_files/invalid_return_error.lox")
    }

    #[test]
    fn invalid_use_of_this() {
        test_for_resolution_error("test_files/invalid_use_of_this_keyword.lox")
    }

    #[test]
    fn return_from_init_error() {
        test_for_resolution_error("test_files/return_from_init_error.lox")
    }

    #[test]
    fn using_super_without_superclass() {
        test_for_resolution_error("test_files/using_super_without_superclass.lox")
    }

    #[test]
    fn top_level_super_use() {
        test_for_resolution_error("test_files/top_level_super.lox")
    }
}
