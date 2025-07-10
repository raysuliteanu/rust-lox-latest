use std::fmt::Display;

use crate::token::{Lexeme, Token};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Ast {
    Class,
    Function,
    // name, initializer
    Variable(Token, Option<Box<Ast>>),
    Block(Vec<AstStmt>),
    Statement(AstStmt),
    Expression(AstExpr),
}

#[derive(Debug, PartialEq)]
pub enum AstStmt {
    // condition, then, else
    If(Box<Ast>, Box<Ast>, Option<Box<Ast>>),
    // cond, body
    While(Box<Ast>, Box<Ast>),
    For,
    Return(Option<Box<Ast>>),
    Print(AstExpr),
    Expression(AstExpr),
}

#[derive(Debug, PartialEq)]
pub enum AstExpr {
    Assignment {
        id: String,
        expr: Box<AstExpr>,
    },
    // expr AND/OR expr
    Logical {
        op: Box<Ast>,
        left: Box<Ast>,
        right: Box<Ast>,
    },
    Terminal(Token),
    Group(Box<Ast>),
    Unary {
        op: Box<Ast>,
        exp: Box<Ast>,
    },
    Binary {
        op: Box<Ast>,
        left: Box<Ast>,
        right: Box<Ast>,
    },
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Class => todo!(),
            Ast::Function => todo!(),
            Ast::Variable(ident, expr) => {
                let name = match &ident.lexeme {
                    Lexeme::Identifier(name) => name,
                    _ => panic!("Variable declaration must have an identifier token"),
                };
                write!(f, "var {name}")?;
                if let Some(ast) = expr {
                    write!(f, " = {ast}")?;
                }
                write!(f, ";")
            }
            Ast::Block(_b) => todo!(),
            Ast::Statement(s) => write!(f, "{s}"),
            Ast::Expression(e) => write!(f, "{e}"),
        }
    }
}

impl Display for AstExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstExpr::Assignment { id, expr } => todo!(),
            AstExpr::Logical { op, left, right } => todo!("logical"),
            AstExpr::Unary { op, exp } => write!(f, "({op} {exp})"),
            AstExpr::Binary { op, left, right } => {
                write!(f, "({op} {left} {right})")
            }
            AstExpr::Group(ast) => write!(f, "(group {ast})"),
            AstExpr::Terminal(token) => match &token.lexeme {
                Lexeme::True(v)
                | Lexeme::False(v)
                | Lexeme::Nil(v)
                | Lexeme::And(v)
                | Lexeme::Or(v)
                | Lexeme::Class(v)
                | Lexeme::For(v)
                | Lexeme::Fun(v)
                | Lexeme::If(v)
                | Lexeme::Else(v)
                | Lexeme::Return(v)
                | Lexeme::Super(v)
                | Lexeme::This(v)
                | Lexeme::Var(v)
                | Lexeme::While(v)
                | Lexeme::Print(v)
                | Lexeme::EqEq(v)
                | Lexeme::BangEq(v)
                | Lexeme::LessEq(v)
                | Lexeme::GreaterEq(v)
                | Lexeme::Identifier(v)
                | Lexeme::String(v) => write!(f, "{}", v.to_lowercase()),
                Lexeme::LeftParen(v)
                | Lexeme::RightParen(v)
                | Lexeme::LeftBrace(v)
                | Lexeme::RightBrace(v)
                | Lexeme::Comma(v)
                | Lexeme::Dot(v)
                | Lexeme::Minus(v)
                | Lexeme::Plus(v)
                | Lexeme::SemiColon(v)
                | Lexeme::Star(v)
                | Lexeme::Eq(v)
                | Lexeme::Bang(v)
                | Lexeme::Less(v)
                | Lexeme::Greater(v)
                | Lexeme::Slash(v) => write!(f, "{v}"),
                Lexeme::Number(_, v) => {
                    if *v == v.trunc() {
                        write!(f, "{v}.0")
                    } else {
                        write!(f, "{v}")
                    }
                }

                Lexeme::Eof(_) => unreachable!(),
            },
        }
    }
}

impl Display for AstStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstStmt::If(cond, then_stmt, else_stmt) => {
                writeln!(f, "if {cond} {{ {then_stmt} }}")?;
                if let Some(else_stmt) = else_stmt {
                    write!(f, "else {{ {else_stmt} }}")?;
                }
                Ok(())
            }
            AstStmt::While(cond, body) => {
                writeln!(f, "while {cond} {{")?;
                writeln!(f, "{body} }}")
            }
            AstStmt::For => todo!(),
            AstStmt::Return(ast) => {
                write!(f, "return")?;
                if let Some(ast) = ast {
                    write!(f, " {ast}")?;
                }
                write!(f, ";")
            }
            AstStmt::Print(ast) => write!(f, "print {ast};"),
            AstStmt::Expression(e) => write!(f, "{e}"),
        }
    }
}
