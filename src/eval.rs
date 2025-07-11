use std::fmt::Display;

use crate::model::{Ast, AstExpr, AstStmt};
use crate::parser::Parser;
use crate::token::Token;

pub enum EvalValue<'e> {
    Number(f64),
    String(&'e str),
    Boolean(bool),
    Nil,
}

impl Display for EvalValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalValue::Number(v) => write!(f, "{v}"),
            EvalValue::String(s) => write!(f, "{s}"),
            EvalValue::Boolean(b) => write!(f, "{b}"),
            EvalValue::Nil => write!(f, "nil"),
        }
    }
}

pub type EvalResult<'e> = Result<EvalValue<'e>, u8>;

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

    fn eval_ast(&self, ast: &'eval Ast) -> EvalResult<'eval> {
        match ast {
            Ast::Class => todo!(),
            Ast::Function => todo!(),
            Ast::Variable(_token, _ast) => todo!(),
            Ast::Statement(stmt) => self.eval_stmt(stmt),
            Ast::Block(_b) => todo!(),
            Ast::Expression(e) => self.eval_expr(e),
        }
    }

    fn eval_stmt(&self, stmt: &'eval AstStmt) -> EvalResult<'eval> {
        match stmt {
            AstStmt::Expression(expr) => self.eval_expr(expr),
            AstStmt::Print(ast) => self.eval_print_stmt(ast),
            AstStmt::For => todo!(),
            AstStmt::If(_ast, _ast1, _ast2) => todo!(),
            AstStmt::Return(_ast) => todo!(),
            AstStmt::While(_ast, _ast1) => todo!(),
        }
    }

    fn eval_expr(&self, expr: &'eval AstExpr) -> EvalResult<'eval> {
        match expr {
            AstExpr::Terminal(token) => self.eval_terminal(token),
            AstExpr::Group(expr) => self.eval_expr(expr),
            AstExpr::Unary { op, exp } => self.eval_unary(op, exp),
            AstExpr::Binary { op, left, right } => self.eval_binary(op, left, right),
            AstExpr::Assignment { id, expr } => todo!("assignment"),
            AstExpr::Logical { op, left, right } => todo!("logical expr"),
        }
    }

    fn eval_print_stmt(&self, expr: &'eval AstExpr) -> EvalResult<'eval> {
        println!("{}", self.eval_expr(expr)?);
        Ok(EvalValue::Nil)
    }

    fn eval_terminal(&self, token: &'eval Token) -> EvalResult<'eval> {
        let val = match &token.lexeme {
            crate::token::Lexeme::Number(_, v) => EvalValue::Number(*v),
            crate::token::Lexeme::String(s) => EvalValue::String(s),
            crate::token::Lexeme::Identifier(_i) => todo!("identifier"),
            crate::token::Lexeme::True(_) => EvalValue::Boolean(true),
            crate::token::Lexeme::False(_) => EvalValue::Boolean(false),
            crate::token::Lexeme::Nil(_) => EvalValue::Nil,
            _ => unimplemented!("{}", token.lexeme),
        };

        Ok(val)
    }

    fn eval_unary(&self, op: &Token, expr: &AstExpr) -> EvalResult<'eval> {
        let val = self.eval_expr(expr)?;
        let result = match op.lexeme {
            crate::token::Lexeme::Bang(_) => match val {
                EvalValue::Number(_) => EvalValue::Boolean(false),
                EvalValue::Boolean(v) => EvalValue::Boolean(!v),
                EvalValue::Nil => EvalValue::Boolean(true),
                _ => todo!("invalid op {} for {val}", op.lexeme),
            },
            crate::token::Lexeme::Minus(_) => match val {
                EvalValue::Number(v) => EvalValue::Number(-v),
                _ => todo!("invalid op {} for {val}", op.lexeme),
            },
            _ => todo!("invalid op {} for {val}", op.lexeme),
        };

        Ok(result)
    }

    fn eval_binary(&self, op: &Token, left: &AstExpr, right: &AstExpr) -> EvalResult<'eval> {
        todo!()
    }
}
