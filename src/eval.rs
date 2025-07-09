use crate::parser::{Ast, AstType, ExpressionType, Parser};

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
            match &ast.ty {
                AstType::Class => todo!(),
                AstType::Function => todo!(),
                AstType::Variable(token, ast) => todo!(),
                AstType::ExprStatement(expr) => todo!(),
                AstType::ForStatement => todo!(),
                AstType::IfStatement(ast, ast1, ast2) => todo!(),
                AstType::PrintStatement(ast) => todo!(),
                AstType::ReturnStatement(ast) => todo!(),
                AstType::WhileStatement(ast, ast1) => todo!(),
                AstType::Block => todo!(),
                AstType::Group(ast) => todo!(),
                AstType::Terminal(token) => todo!(),
                AstType::Expression(expression_type) => match expression_type {
                    ExpressionType::Unary { op, exp } => todo!(),
                    ExpressionType::Binary { op, left, right } => todo!(),
                },
            }
        }

        Ok(EvalValue::Number(1.0))
    }
}
