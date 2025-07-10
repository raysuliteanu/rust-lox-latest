use std::fmt::Display;

use crate::model::{Ast, AstExpr, AstStmt};
use crate::parser::Parser;

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

    fn eval(&self, mut tree: std::slice::Iter<'_, Ast>) -> EvalResult {
        while let Some(ast) = tree.next() {
            let _result = self.eval_ast(ast)?;
        }

        Ok(EvalValue::Number(0.0))
    }

    fn eval_ast(&self, ast: &Ast) -> EvalResult {
        match ast {
            Ast::Class => todo!(),
            Ast::Function => todo!(),
            Ast::Variable(_token, _ast) => todo!(),
            Ast::Statement(stmt) => self.eval_stmt(stmt),
            Ast::Block(_b) => todo!(),
            Ast::Expression(e) => self.eval_expr(e),
        }
    }

    fn eval_stmt(&self, stmt: &AstStmt) -> EvalResult {
        match stmt {
            AstStmt::Expression(expr) => self.eval_expr(expr),
            AstStmt::For => todo!(),
            AstStmt::If(_ast, _ast1, _ast2) => todo!(),
            AstStmt::Print(ast) => self.eval_print_stmt(ast),
            AstStmt::Return(_ast) => todo!(),
            AstStmt::While(_ast, _ast1) => todo!(),
        }
    }

    fn eval_expr(&self, expr: &AstExpr) -> EvalResult {
        match expr {
            AstExpr::Assignment { id, expr } => todo!("assignment"),
            AstExpr::Logical { op, left, right } => todo!("logical expr"),
            AstExpr::Terminal(token) => todo!("terminal"),
            AstExpr::Group(ast) => todo!("group"),
            AstExpr::Unary { op, exp } => todo!("unary"),
            AstExpr::Binary { op, left, right } => todo!("binary"),
        }
    }

    fn eval_print_stmt(&self, expr: &AstExpr) -> EvalResult {
        println!("{}", self.eval_expr(expr)?);
        Ok(EvalValue::Nil)
    }
}
