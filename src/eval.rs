use crate::model::{Ast, AstExpr, AstStmt};
use crate::parser::Parser;

pub enum EvalValue<'e> {
    Number(f64),
    String(&'e str),
    Boolean(bool),
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
            match ast {
                Ast::Class => todo!(),
                Ast::Function => todo!(),
                Ast::Variable(_token, _ast) => todo!(),
                Ast::Statement(stmt) => match stmt {
                    AstStmt::ExpressionStatement(expr) => match expr.as_ref() {
                        Ast::Expression(AstExpr::Terminal(_token)) => todo!(),
                        Ast::Expression(AstExpr::Group(_ast)) => todo!(),
                        Ast::Expression(AstExpr::Logical { op: _, left: _, right: _ }) => todo!(),
                        Ast::Expression(AstExpr::Unary { op: _, exp: _ }) => todo!(),
                        Ast::Expression(AstExpr::Binary { op: _, left: _, right: _ }) => todo!(),
                        _ => todo!(),
                    },
                    AstStmt::ForStatement => todo!(),
                    AstStmt::IfStatement(_ast, _ast1, _ast2) => todo!(),
                    AstStmt::PrintStatement(_ast) => todo!(),
                    AstStmt::ReturnStatement(_ast) => todo!(),
                    AstStmt::WhileStatement(_ast, _ast1) => todo!(),
                },
                Ast::Block(_) => todo!(),
                Ast::Expression(_) => todo!(),
            }
        }

        Ok(EvalValue::Number(1.0))
    }
}
