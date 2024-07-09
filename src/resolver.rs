use std::collections::HashMap;

use crate::{
    expr::{self, Expr},
    interpreter::Interpreter,
    print_error,
    scanner::Token,
    stmt::{self, Stmt},
    RuntimeError,
};

#[derive(Clone)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver<'_> {
    pub fn new(interpreter: &mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
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
        let enclosing_function = self.current_function.clone();
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

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) {
        self.begin_scope();
        self.resolve_block(statements);
        self.end_scope();
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

    fn visit_grouping_expr(&mut self, expression: &Expr) {
        self.resolve_expr(expression);
    }

    fn visit_literal_expr(&self) {}

    fn visit_logical_expr(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
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
        }
    }
}
