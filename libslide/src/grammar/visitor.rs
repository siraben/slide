//! Traits for visiting slide grammar trees.

use super::*;
use crate::Span;

/// Describes a [statement list](super::StmtList) visitor.
pub trait StmtVisitor<'a> {
    fn visit(&mut self, stmt_list: &'a StmtList) {
        for stmt in stmt_list.iter() {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        self.visit_stmt_kind(&stmt.kind)
    }

    fn visit_stmt_kind(&mut self, stmt_kind: &'a StmtKind) {
        match stmt_kind {
            StmtKind::Expr(expr) => self.visit_expr(expr),
            StmtKind::Assignment(asgn) => self.visit_asgn(asgn),
        }
    }

    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        self.visit_var(&asgn.var);
        self.visit_asgn_op(&asgn.asgn_op);
        self.visit_expr(&asgn.rhs);
    }

    fn visit_asgn_op(&mut self, _asgn_op: &'a AssignmentOp) {}

    fn visit_expr(&mut self, expr: &'a RcExpr) {
        match expr.as_ref() {
            Expr::Const(k) => self.visit_const(k),
            Expr::Var(v) => self.visit_var(v),
            Expr::BinaryExpr(b) => self.visit_binary(b),
            Expr::UnaryExpr(u) => self.visit_unary(u, expr.span),
            Expr::Parend(p) => self.visit_parend(p, expr.span),
            Expr::Bracketed(b) => self.visit_bracketed(b, expr.span),
        }
    }

    fn visit_const(&mut self, _konst: &'a f64) {}

    fn visit_var(&mut self, _var: &'a InternedStr) {}

    fn visit_binary_op(&mut self, _op: BinaryOperator) {}

    fn visit_binary(&mut self, expr: &'a BinaryExpr<RcExpr>) {
        self.visit_expr(&expr.lhs);
        self.visit_binary_op(expr.op);
        self.visit_expr(&expr.rhs);
    }

    fn visit_unary_op(&mut self, _op: UnaryOperator) {}

    fn visit_unary(&mut self, expr: &'a UnaryExpr<RcExpr>, _span: Span) {
        self.visit_unary_op(expr.op);
        self.visit_expr(&expr.rhs);
    }

    fn visit_parend(&mut self, expr: &'a RcExpr, _span: Span) {
        self.visit_expr(expr);
    }

    fn visit_bracketed(&mut self, expr: &'a RcExpr, _span: Span) {
        self.visit_expr(expr);
    }
}

/// Describes an [expression pattern](super::ExprPat) visitor.
pub trait ExprPatVisitor<'a> {
    fn visit(&mut self, expr_pat: &'a RcExprPat) {
        match expr_pat.as_ref() {
            ExprPat::Const(k) => self.visit_const(k),
            ExprPat::VarPat(v) => self.visit_var_pat(v, expr_pat.span),
            ExprPat::ConstPat(k) => self.visit_const_pat(k, expr_pat.span),
            ExprPat::AnyPat(a) => self.visit_any_pat(a, expr_pat.span),
            ExprPat::BinaryExpr(b) => self.visit_binary(b),
            ExprPat::UnaryExpr(u) => self.visit_unary(u, expr_pat.span),
            ExprPat::Parend(p) => self.visit_parend(p, expr_pat.span),
            ExprPat::Bracketed(b) => self.visit_bracketed(b, expr_pat.span),
        }
    }

    fn visit_const(&mut self, _konst: &f64) {}

    fn visit_var_pat(&mut self, _var_pat: &'a str, _span: Span) {}

    fn visit_const_pat(&mut self, _const_pat: &'a str, _span: Span) {}

    fn visit_any_pat(&mut self, _any_pat: &'a str, _span: Span) {}

    fn visit_binary_op(&mut self, _op: BinaryOperator) {}

    fn visit_binary(&mut self, expr: &'a BinaryExpr<RcExprPat>) {
        self.visit(&expr.lhs);
        self.visit_binary_op(expr.op);
        self.visit(&expr.rhs);
    }

    fn visit_unary_op(&mut self, _op: UnaryOperator) {}

    fn visit_unary(&mut self, expr: &'a UnaryExpr<RcExprPat>, _span: Span) {
        self.visit_unary_op(expr.op);
        self.visit(&expr.rhs);
    }

    fn visit_parend(&mut self, expr: &'a RcExprPat, _span: Span) {
        self.visit(expr);
    }

    fn visit_bracketed(&mut self, expr: &'a RcExprPat, _span: Span) {
        self.visit(expr);
    }
}
