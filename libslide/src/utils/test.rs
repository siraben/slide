/// Parses a statement.
#[macro_export]
macro_rules! parse_stmt {
    ($expr:expr) => {{
        use crate::{parse_statements, scan};

        let tokens = scan($expr).tokens;
        parse_statements(tokens, $expr).program
    }};
}

/// Parses an expression.
#[macro_export]
macro_rules! parse_expr {
    ($expr:expr) => {{
        use crate::grammar::*;
        match crate::parse_stmt!($expr).into_iter().next().unwrap().kind {
            StmtKind::Expr(expr) => expr,
            _ => unreachable!(),
        }
    }};
}

/// Parses an assignment.
#[macro_export]
macro_rules! parse_asgn {
    ($asgn:expr) => {{
        use crate::grammar::*;
        match crate::parse_stmt!($asgn).into_iter().next().unwrap().kind {
            StmtKind::Assignment(asgn) => asgn,
            _ => unreachable!(),
        }
    }};
}
