use std::collections::HashMap;

use crate::{
    expr::{self, Expr},
    interpreter::Interpreter,
    print_error,
    scanner::Token,
    stmt::{self, Stmt},
};

struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) {}

    fn resolve_block(&mut self, statements: &Vec<Stmt>) {
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

    fn resolve_function(&self, params: Vec<Token>, body: Vec<Stmt>) {
        self.begin_scope();

        for param in params {
            self.declare(&param);
            self.define(&param);
        }
        self.resolve_block(&body);

        self.end_scope();
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
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.peek_scopes_mut();
        scope.insert(name.lexeme.clone(), true);
    }

    fn resolve_local(&self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
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

    fn visit_function_stmt(&mut self, function_stmt: &Stmt, name: &Token) {
        self.declare(name);
        self.define(name);
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) {
        self.declare(name);

        if let Some(initializer) = initializer {
            self.resolve_expr(initializer);
        }

        self.define(name);
    }

    fn visit_assign_expr(&mut self, var_expr: &Expr, name: &Token, value: &Expr) {
        self.resolve_expr(value);
        self.resolve_local(var_expr, name);
    }

    fn visit_var_expr(&self, var_expr: &Expr, name: &Token) {
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

impl expr::Visitor<()> for Resolver {
    fn visit_expr(&mut self, expression: &expr::Expr) {
        todo!()
    }
}

impl stmt::Visitor<()> for Resolver {
    fn visit_stmt(&mut self, stmt: &stmt::Stmt) {
        todo!()
    }
}
