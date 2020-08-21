//! Emit strategies for the libslide grammar IR.

use crate::grammar::*;

use core::fmt;
use std::rc::Rc;

/// The format in which a slide grammar should be emitted.
#[derive(Copy, Clone)]
pub enum EmitFormat {
    /// Canonical, human-readable form.
    /// For example, `1+1` is output as `1 + 1`.
    Pretty,
    /// S-expression form.
    /// For example, `1+1` is output as `(+ 1 1)`.
    SExpression,
    /// LaTeX output form.
    /// For example, `(1 + 1)` is output as `\left\(1 + 1\right\)`.
    /// NB: this is not yet implemented.
    Latex,
    /// Slide internal debug form.
    /// NB: this form is not stable, and no assumptions should be made about it.
    Debug,
}

impl From<String> for EmitFormat {
    fn from(ef: String) -> Self {
        match ef.as_ref() {
            "pretty" => EmitFormat::Pretty,
            "s-expression" => EmitFormat::SExpression,
            "latex" => EmitFormat::Latex,
            "debug" => EmitFormat::Debug,
            _ => unreachable!(),
        }
    }
}

/// Implements the emission of a type in an [EmitFormat][EmitFormat].
pub trait Emit
where
    // These are trivially implementable using `emit_pretty` and `emit_debug`. The easiest way to
    // do this is with the `fmt_emit_impl` macro.
    Self: fmt::Display + fmt::Debug,
{
    /// Emit `self` with the given [EmitFormat][EmitFormat].
    fn emit(&self, form: EmitFormat) -> String {
        match form {
            EmitFormat::Pretty => self.emit_pretty(),
            EmitFormat::SExpression => self.emit_s_expression(),
            EmitFormat::Latex => self.emit_latex(),
            EmitFormat::Debug => self.emit_debug(),
        }
    }

    /// Emit `self` with the [pretty emit format][EmitFormat::Pretty]
    fn emit_pretty(&self) -> String;

    /// Emit `self` with the [debug emit format][EmitFormat::Debug]
    fn emit_debug(&self) -> String {
        format!("{:#?}", self)
    }

    /// Emit `self` with the [s_expression emit format][EmitFormat::SExpression]
    fn emit_s_expression(&self) -> String {
        unimplemented!();
    }

    /// Emit `self` with the [LaTeX emit format][EmitFormat::Latex]
    fn emit_latex(&self) -> String;
}

#[inline]
fn emit_pretty(arg: &impl Emit) -> String {
    arg.emit_pretty()
}

#[inline]
fn emit_latex(arg: &impl Emit) -> String {
    arg.emit_latex()
}

/// Implements `core::fmt::Display` for a type implementing `Emit`.
/// TODO: Maybe this can be a proc macro?
macro_rules! fmt_emit_impl {
    ($S:path) => {
        impl core::fmt::Display for $S {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.emit_pretty(),)
            }
        }
    };
}

macro_rules! normal_wrap {
    (($expr:expr)) => {
        format!("({})", $expr)
    };
    ([$expr:expr]) => {
        format!("[{}]", $expr)
    };
}

macro_rules! latex_wrap {
    (($expr:expr)) => {
        format!("\\left\\({}\\right\\)", $expr)
    };
    ([$expr:expr]) => {
        format!("\\left\\[{}\\right\\]", $expr)
    };
}

fmt_emit_impl!(Stmt);
impl Emit for Stmt {
    fn emit_pretty(&self) -> String {
        match self {
            Self::Expr(expr) => expr.emit_pretty(),
            Self::Assignment(asgn) => asgn.emit_pretty(),
        }
    }

    fn emit_s_expression(&self) -> String {
        match self {
            Self::Expr(expr) => expr.emit_s_expression(),
            Self::Assignment(Assignment { var, rhs }) => {
                format!("(= {} {})", var, rhs.emit_s_expression())
            }
        }
    }

    fn emit_latex(&self) -> String {
        match self {
            Self::Expr(expr) => expr.emit_latex(),
            Self::Assignment(asgn) => asgn.emit_latex(),
        }
    }
}

fmt_emit_impl!(Assignment);
impl Emit for Assignment {
    fn emit_pretty(&self) -> String {
        format!("{} = {}", self.var, self.rhs.emit_pretty())
    }

    fn emit_latex(&self) -> String {
        format!("{} = {}", self.var, self.rhs.emit_latex())
    }
}

fmt_emit_impl!(Expr);
impl Emit for Expr {
    fn emit_pretty(&self) -> String {
        match self {
            Self::Const(num) => num.to_string(),
            Self::Var(var) => var.to_string(),
            Self::BinaryExpr(binary_expr) => binary_expr.emit_pretty(),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_pretty(),
            Self::Parend(expr) => normal_wrap!((expr.emit_pretty())),
            Self::Bracketed(expr) => normal_wrap!([expr.emit_pretty()]),
        }
    }

    fn emit_s_expression(&self) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::Var(var) => var.to_string(),
            Self::BinaryExpr(BinaryExpr { op, lhs, rhs }) => format!(
                "({} {} {})",
                op.emit_pretty(),
                lhs.emit_s_expression(),
                rhs.emit_s_expression()
            ),
            Self::UnaryExpr(UnaryExpr { op, rhs }) => {
                format!("({} {})", op.emit_pretty(), rhs.emit_s_expression())
            }
            Self::Parend(inner) => normal_wrap!((inner.emit_s_expression())),
            Self::Bracketed(inner) => normal_wrap!([inner.emit_s_expression()]),
        }
    }

    fn emit_latex(&self) -> String {
        match self {
            Self::Const(num) => num.to_string(),
            Self::Var(var) => var.to_string(),
            Self::BinaryExpr(binary_expr) => binary_expr.emit_latex(),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_latex(),
            Self::Parend(expr) => latex_wrap!((expr.emit_latex())),
            Self::Bracketed(expr) => latex_wrap!([expr.emit_latex()]),
        }
    }
}

impl Emit for Rc<Expr> {
    fn emit_pretty(&self) -> String {
        self.as_ref().emit_pretty()
    }

    fn emit_s_expression(&self) -> String {
        self.as_ref().emit_s_expression()
    }

    fn emit_latex(&self) -> String {
        self.as_ref().emit_latex()
    }
}

fmt_emit_impl!(BinaryOperator);
impl Emit for BinaryOperator {
    fn emit_pretty(&self) -> String {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Mult => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Exp => "^",
        }
        .to_owned()
    }

    fn emit_latex(&self) -> String {
        self.emit_pretty()
    }
}

macro_rules! format_binary_operand {
    ($E:ident, $parent_expr:ident, $operand:expr, $is_right_operand:expr, $emit:ident, $wrap:ident) => {
        match $operand.as_ref() {
            // We want to format items like
            //    v--------- child op
            //         v---- parent op
            // (3 + 5) ^ 2 [1]
            //  3 + 5  + 2
            //  3 - 5  + 2
            //  3 * 5  + 2
            // and
            //   v---------- parent op
            //        v----- child op
            // 2 +  3 + 5
            // 2 - (3 + 5)
            // 2 * (3 + 5)
            //
            // So the idea here is as follows:
            // - if the child op precedence is less than the parent op, we must always parenthesize
            //   it ([1])
            // - if the op precedences are equivalent, then
            //   - if the child is on the LHS, we can always unwrap it
            //   - if the child is on the RHS, we parenthesize it unless the parent op is
            //     associative
            //
            // I think this is enough, but maybe we're overlooking left/right associativity?
            $E::BinaryExpr(child) => {
                if child.op.precedence() < $parent_expr.op.precedence()
                    || ($is_right_operand
                        && child.op.precedence() == $parent_expr.op.precedence()
                        && !$parent_expr.op.is_associative())
                {
                    $wrap!(($emit(child)))
                } else {
                    $emit(child)
                }
            }
            expr => $emit(expr),
        }
    };
}

macro_rules! display_binary_expr {
    (<$expr:ident>) => {
        fmt_emit_impl!(BinaryExpr<$expr>);
        impl Emit for BinaryExpr<$expr> {
            fn emit_pretty(&self) -> String {
                format!(
                    "{} {} {}",
                    format_binary_operand!($expr, self, &self.lhs, false, emit_pretty, normal_wrap),
                    self.op.emit_pretty(),
                    format_binary_operand!($expr, self, &self.rhs, true, emit_pretty, normal_wrap),
                )
            }

            fn emit_latex(&self) -> String {
                format!(
                    "{} {} {}",
                    format_binary_operand!($expr, self, &self.lhs, false, emit_latex, latex_wrap),
                    self.op.emit_latex(),
                    format_binary_operand!($expr, self, &self.rhs, true, emit_latex, latex_wrap),
                )
            }
        }
    };
}
display_binary_expr!(<Expr>);
display_binary_expr!(<ExprPat>);

fmt_emit_impl!(UnaryOperator);
impl Emit for UnaryOperator {
    fn emit_pretty(&self) -> String {
        match self {
            Self::SignPositive => "+",
            Self::SignNegative => "-",
        }
        .to_owned()
    }

    fn emit_latex(&self) -> String {
        self.emit_pretty()
    }
}

macro_rules! display_unary_expr {
    (<$expr:ident>) => {
        fmt_emit_impl!(UnaryExpr<$expr>);
        impl Emit for UnaryExpr<$expr> {
            fn emit_pretty(&self) -> String {
                let format_arg = |arg: &Rc<$expr>| match arg.as_ref() {
                    $expr::BinaryExpr(l) => format!("({})", l),
                    expr => expr.emit_pretty(),
                };
                format!("{}{}", self.op.emit_pretty(), format_arg(&self.rhs))
            }

            fn emit_latex(&self) -> String {
                let format_arg = |arg: &Rc<$expr>| match arg.as_ref() {
                    $expr::BinaryExpr(l) => latex_wrap!((l)),
                    expr => expr.emit_latex(),
                };
                format!("{}{}", self.op.emit_latex(), format_arg(&self.rhs))
            }
        }
    };
}
display_unary_expr!(<Expr>);
display_unary_expr!(<ExprPat>);

fmt_emit_impl!(ExprPat);
impl Emit for ExprPat {
    fn emit_pretty(&self) -> String {
        match self {
            Self::Const(num) => num.to_string(),
            Self::VarPat(var) | Self::ConstPat(var) | Self::AnyPat(var) => var.to_string(),
            Self::BinaryExpr(binary_expr) => binary_expr.emit_pretty(),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_pretty(),
            Self::Parend(expr) => normal_wrap!((expr.emit_pretty())),
            Self::Bracketed(expr) => normal_wrap!([expr.emit_pretty()]),
        }
    }

    fn emit_s_expression(&self) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::VarPat(pat) | Self::ConstPat(pat) | Self::AnyPat(pat) => pat.to_string(),
            Self::BinaryExpr(BinaryExpr { op, lhs, rhs }) => format!(
                "({} {} {})",
                op.to_string(),
                lhs.emit_s_expression(),
                rhs.emit_s_expression()
            ),
            Self::UnaryExpr(UnaryExpr { op, rhs }) => {
                format!("({} {})", op.to_string(), rhs.emit_s_expression())
            }
            Self::Parend(inner) => normal_wrap!((inner.emit_s_expression())),
            Self::Bracketed(inner) => normal_wrap!([inner.emit_s_expression()]),
        }
    }

    fn emit_latex(&self) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::VarPat(pat) | Self::ConstPat(pat) | Self::AnyPat(pat) => {
                // $a, #a, _a all need to be escaped as \$a, \#a, \_a.
                format!("\\{}", pat.to_string())
            }
            Self::BinaryExpr(binary_expr) => binary_expr.emit_latex(),
            Self::UnaryExpr(unary_expr) => unary_expr.emit_latex(),
            Self::Parend(inner) => latex_wrap!((inner.emit_latex())),
            Self::Bracketed(inner) => latex_wrap!([inner.emit_latex()]),
        }
    }
}

impl Emit for Rc<ExprPat> {
    fn emit_pretty(&self) -> String {
        self.as_ref().emit_pretty()
    }

    fn emit_s_expression(&self) -> String {
        self.as_ref().emit_s_expression()
    }

    fn emit_latex(&self) -> String {
        self.as_ref().emit_latex()
    }
}