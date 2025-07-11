use std::fmt::Display;

use crate::model::{Ast, AstExpr, AstStmt};
use crate::parser::Parser;
use crate::token::{Lexeme, Token};

pub enum EvalValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Display for EvalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalValue::Number(v) => write!(f, "{v}"),
            EvalValue::String(s) => write!(f, "{s}"),
            EvalValue::Boolean(b) => write!(f, "{b}"),
            EvalValue::Nil => write!(f, "nil"),
        }
    }
}

pub type EvalResult = Result<EvalValue, u8>;

pub struct Eval<'eval> {
    source: &'eval str,
}

impl<'eval> Eval<'_> {
    pub fn new(source: &str) -> Eval {
        Eval { source }
    }

    pub fn evaluate(&self) -> EvalResult {
        let parser = Parser::new(self.source, false);
        let tree = parser.parse()?;

        self.eval(tree.iter())
    }

    fn eval(&self, tree: std::slice::Iter<'_, Ast>) -> EvalResult {
        for ast in tree {
            let _result = self.eval_ast(ast)?;
            // TODO: deal with result
        }

        Ok(EvalValue::Number(0.0))
    }

    fn eval_ast(&self, ast: &'eval Ast) -> EvalResult {
        match ast {
            Ast::Class => todo!(),
            Ast::Function => todo!(),
            Ast::Variable(_token, _ast) => todo!(),
            Ast::Statement(stmt) => self.eval_stmt(stmt),
            Ast::Block(_b) => todo!(),
            Ast::Expression(e) => self.eval_expr(e),
        }
    }

    fn eval_stmt(&self, stmt: &'eval AstStmt) -> EvalResult {
        match stmt {
            AstStmt::Expression(expr) => self.eval_expr(expr),
            AstStmt::Print(ast) => self.eval_print_stmt(ast),
            AstStmt::For => todo!(),
            AstStmt::If(_ast, _ast1, _ast2) => todo!(),
            AstStmt::Return(_ast) => todo!(),
            AstStmt::While(_ast, _ast1) => todo!(),
        }
    }

    fn eval_expr(&self, expr: &'eval AstExpr) -> EvalResult {
        match expr {
            AstExpr::Terminal(token) => self.eval_terminal(token),
            AstExpr::Group(expr) => self.eval_expr(expr),
            AstExpr::Unary { op, exp } => self.eval_unary(op, exp),
            AstExpr::Binary { op, left, right } => self.eval_binary(op, left, right),
            AstExpr::Assignment { id: _, expr: _ } => todo!("assignment"),
            AstExpr::Logical {
                op: _,
                left: _,
                right: _,
            } => todo!("logical expr"),
        }
    }

    fn eval_print_stmt(&self, expr: &'eval AstExpr) -> EvalResult {
        println!("{}", self.eval_expr(expr)?);
        Ok(EvalValue::Nil)
    }

    fn eval_terminal(&self, token: &'eval Token) -> EvalResult {
        let val = match &token.lexeme {
            Lexeme::Number(_, v) => EvalValue::Number(*v),
            Lexeme::String(s) => EvalValue::String(s.to_string()),
            Lexeme::Identifier(_i) => todo!("identifier"),
            Lexeme::True(_) => EvalValue::Boolean(true),
            Lexeme::False(_) => EvalValue::Boolean(false),
            Lexeme::Nil(_) => EvalValue::Nil,
            _ => unimplemented!("{}", token.lexeme),
        };

        Ok(val)
    }

    fn eval_unary(&self, op: &Token, expr: &AstExpr) -> EvalResult {
        let val = self.eval_expr(expr)?;
        let result = match op.lexeme {
            Lexeme::Bang(_) => match val {
                EvalValue::Number(_) => EvalValue::Boolean(false),
                EvalValue::Boolean(v) => EvalValue::Boolean(!v),
                EvalValue::Nil => EvalValue::Boolean(true),
                _ => todo!("invalid op {} for {val}", op.lexeme),
            },
            Lexeme::Minus(_) => match val {
                EvalValue::Number(v) => EvalValue::Number(-v),
                _ => todo!("invalid op {} for {val}", op.lexeme),
            },
            _ => todo!("invalid op {} for {val}", op.lexeme),
        };

        Ok(result)
    }

    fn eval_binary(&self, op: &Token, left: &AstExpr, right: &AstExpr) -> EvalResult {
        let left_expr = self.eval_expr(left)?;
        let right_expr = self.eval_expr(right)?;
        let result = match op.lexeme {
            Lexeme::Plus(_) => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Number(l + r),
                (EvalValue::String(s_l), EvalValue::String(s_r)) => EvalValue::String(s_l + &s_r),
                _ => todo!("Operands must be two numbers or two strings."),
            },
            Lexeme::Minus(_) => todo!(),
            Lexeme::Star(_) => todo!(),
            Lexeme::Slash(_) => todo!(),
            Lexeme::EqEq(_) => todo!(),
            Lexeme::BangEq(_) => todo!(),
            Lexeme::Less(_) => todo!(),
            Lexeme::LessEq(_) => todo!(),
            Lexeme::Greater(_) => todo!(),
            Lexeme::GreaterEq(_) => todo!(),
            _ => todo!(),
        };

        Ok(result)
    }
}
