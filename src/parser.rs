use std::{fmt::Display, iter::Peekable};

use log::trace;
use miette::Diagnostic;
use thiserror::Error;

use crate::token::{Scanner, Token, TokenType};

type PeekableTokenIter<'a> = Peekable<std::slice::Iter<'a, Token>>;

#[derive(Debug)]
pub struct Ast {
    ty: AstType,
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum AstType {
    Class,
    Function,
    Variable,
    ExprStatement,
    ForStatement,
    IfStatement,
    PrintStatement(Box<Ast>),
    ReturnStatement(Option<Box<Ast>>),
    WhileStatement,
    Block,
    Group(Box<Ast>),
    Expression(ExpressionType),
    Terminal(Token),
}

impl Display for AstType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstType::Class => todo!(),
            AstType::Function => todo!(),
            AstType::Variable => todo!(),
            AstType::ExprStatement => todo!(),
            AstType::ForStatement => todo!(),
            AstType::IfStatement => todo!(),
            AstType::PrintStatement(ast) => {
                write!(f, "print {};", ast)
            }
            AstType::ReturnStatement(ast) => {
                write!(f, "return")?;
                if ast.is_some() {
                    write!(f, " {}", ast.as_ref().unwrap())?;
                }
                write!(f, ";")
            }
            AstType::WhileStatement => todo!(),
            AstType::Block => todo!(),
            AstType::Group(ast) => {
                write!(f, "(group {ast})")
            }
            AstType::Expression(e) => {
                write!(f, "({e})")
            }
            AstType::Terminal(t) => {
                write!(f, "{}", t.lexeme)
            }
        }
    }
}

#[derive(Debug)]
pub enum ExpressionType {
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

impl Display for ExpressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionType::Unary { op, exp } => {
                write!(f, "{op}{exp}")
            }
            ExpressionType::Binary { op, left, right } => {
                write!(f, "{left} {op} {right}")
            }
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[diagnostic(code("65"))]
pub enum ParseError {
    #[error("missing token '{}' got '{}'", expected, actual)]
    MissingToken {
        expected: TokenType,
        actual: TokenType,
    },

    #[error("unexpected token '{actual}'")]
    UnexpectedToken { actual: Token },

    #[error("Unexpected EOF")]
    UnexpectedEof,
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
}

macro_rules! ast_missing_token {
    ($e: expr, $a: expr) => {
        ParseError::MissingToken {
            expected: $e,
            actual: $a,
        }
        .into()
    };
}

/// macro for handling error case when next token was not what was expected.
/// it could be either because the next token was some other "real" token or
/// it could because the next "token" was actually no more tokens i.e. "eof"
macro_rules! ast_expected_token {
    ($t: expr, $e: expr) => {
        Err(ParseError::MissingToken {
            expected: $e,
            actual: $t.lexeme.clone(),
        }
        .into())
    };
}

/// convenience macro for creating a new binary expression AST node
macro_rules! ast_binary {
    ($op: expr, $l: expr, $r: expr) => {
        Ast {
            ty: AstType::Expression(ExpressionType::Binary {
                op: Box::new($op),
                left: Box::new($l),
                right: Box::new($r),
            }),
        }
    };
}

/// convenience macro for creating a new expression group AST node
macro_rules! ast_group {
    ($e: expr) => {
        Ast {
            ty: AstType::Group(Box::new($e)),
        }
    };
}

/// convenience macro for creating a new unary expression AST node
macro_rules! ast_unary {
    ($op: expr, $exp: expr) => {
        Ast {
            ty: AstType::Expression(ExpressionType::Unary {
                op: Box::new(Ast {
                    ty: AstType::Terminal($op.clone()),
                }),
                exp: Box::new($exp),
            }),
        }
    };
}

/// convenience macro for creating a new terminal AST node
macro_rules! ast_terminal {
    ($t: expr) => {
        Ast {
            ty: AstType::Terminal($t.clone()),
        }
    };
}

impl Parser<'_> {
    pub fn new(source: &str) -> Parser {
        Parser {
            scanner: Scanner::new(source),
        }
    }

    pub fn parse(&mut self) -> miette::Result<Vec<Ast>> {
        let tokens = &self.scanner.scan()?;
        if tokens.is_empty() {
            return Ok(vec![]);
        }

        Parser::program(&mut tokens.iter().peekable())
    }

    fn program(tokens: &mut PeekableTokenIter) -> miette::Result<Vec<Ast>> {
        let mut ast: Vec<Ast> = Vec::new();

        while let Some(_token) = tokens.peek() {
            let statement = Parser::statement(tokens)?;
            ast.push(statement);
        }

        Ok(ast)
    }

    fn statement(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("statement: {:?}", tokens.peek());
        if let Some(token) = tokens.peek() {
            match token.lexeme {
                // left brace token indicates block start
                crate::token::TokenType::LeftBrace => Parser::parse_block(tokens),
                crate::token::TokenType::For => Parser::for_stmt(tokens),
                crate::token::TokenType::If => Parser::if_stmt(tokens),
                crate::token::TokenType::Print => Parser::print_stmt(tokens),
                crate::token::TokenType::Return => Parser::return_stmt(tokens),
                crate::token::TokenType::While => Parser::while_stmt(tokens),
                _ => Parser::expression_statement(tokens),
            }
        } else {
            // TODO: I think this should never happen since in program() there's already a peek()
            Err(ParseError::UnexpectedEof.into())
        }
    }

    fn parse_block(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("block_stmt: {:?}", tokens.peek());
        todo!("parse block")
    }

    fn for_stmt(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("for_stmt: {:?}", tokens.peek());
        todo!("parse for stmt")
    }

    fn if_stmt(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("if_stmt: {:?}", tokens.peek());
        todo!("parse if stmt")
    }

    fn print_stmt(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("print_stmt: {:?}", tokens.peek());

        let print_token = tokens.next().unwrap();
        assert_eq!(TokenType::Print, print_token.lexeme);

        let expr = Parser::expression(tokens)?;
        if tokens
            .next_if(|t| t.lexeme == TokenType::SemiColon)
            .is_some()
        {
            Ok(Ast {
                ty: AstType::PrintStatement(Box::new(expr)),
            })
        } else if tokens.peek().is_some() {
            ast_expected_token!(print_token, TokenType::SemiColon)
        } else {
            Err(ParseError::UnexpectedEof.into())
        }
    }

    fn return_stmt(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("return_stmt: {:?}", tokens.peek());
        assert_eq!(TokenType::Return, tokens.next().unwrap().lexeme);

        if tokens
            .next_if(|t| t.lexeme == TokenType::SemiColon)
            .is_some()
        {
            // a 'return' without expression i.e. "return;"
            Ok(Ast {
                ty: AstType::ReturnStatement(None),
            })
        } else if tokens.peek().is_some_and(|t| {
            !matches!(
                t.lexeme,
                TokenType::LeftBrace
                    | TokenType::For
                    | TokenType::While
                    | TokenType::If
                    | TokenType::Print
                    | TokenType::Return
            )
        }) {
            let ast = Parser::expression(tokens)?;
            if tokens
                .next_if(|t| t.lexeme == TokenType::SemiColon)
                .is_some()
            {
                Ok(Ast {
                    ty: AstType::ReturnStatement(Some(Box::new(ast))),
                })
            } else if tokens.peek().is_some() {
                ast_expected_token!(tokens.peek().unwrap(), TokenType::SemiColon)
            } else {
                Err(ast_missing_token!(TokenType::SemiColon, TokenType::Eof))
            }
        } else {
            Err(ast_missing_token!(TokenType::SemiColon, TokenType::Eof))
        }
    }

    fn while_stmt(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("while_stmt: {:?}", tokens.peek());
        todo!("parse while stmt")
    }

    fn expression_statement(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        let token = tokens.peek();
        trace!("expr_stmt: {:?}", token);
        let ast = Parser::expression(tokens)?;
        if tokens
            .next_if(|t| t.lexeme == TokenType::SemiColon)
            .is_some()
        {
            Ok(ast)
        } else if tokens.peek().is_some() {
            ast_expected_token!(tokens.peek().unwrap(), TokenType::SemiColon)
        } else {
            Err(ast_missing_token!(TokenType::SemiColon, TokenType::Eof))
        }
    }

    fn expression(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("expr: {:?}", tokens.peek());
        Parser::equality(tokens)
    }

    fn equality(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("equality: {:?}", tokens.peek());
        let mut left = Parser::comparison(tokens)?;

        while let Some(t) =
            tokens.next_if(|t| matches!(t.lexeme, TokenType::BangEq | TokenType::EqEq))
        {
            let right = Parser::comparison(tokens)?;
            let op = ast_terminal!(t);
            left = ast_binary!(op, left, right);
        }

        Ok(left)
    }

    fn comparison(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("comparison: {:?}", tokens.peek());
        let mut left = Parser::term(tokens)?;

        while let Some(t) = tokens.next_if(|t| {
            matches!(
                t.lexeme,
                TokenType::Greater | TokenType::GreaterEq | TokenType::Less | TokenType::LessEq
            )
        }) {
            let right = Parser::term(tokens)?;
            let op = ast_terminal!(t);
            left = ast_binary!(op, left, right);
        }

        Ok(left)
    }

    fn term(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("term: {:?}", tokens.peek());
        let mut left = Parser::factor(tokens)?;

        while let Some(t) =
            tokens.next_if(|t| matches!(t.lexeme, TokenType::Plus | TokenType::Minus))
        {
            let right = Parser::factor(tokens)?;
            let op = ast_terminal!(t);
            left = ast_binary!(op, left, right);
        }

        Ok(left)
    }

    fn factor(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("factor: {:?}", tokens.peek());
        let mut left = Parser::unary(tokens)?;

        while let Some(t) =
            tokens.next_if(|t| matches!(t.lexeme, TokenType::Star | TokenType::Slash))
        {
            let right = Parser::unary(tokens)?;
            let op = ast_terminal!(t);
            left = ast_binary!(op, left, right);
        }

        Ok(left)
    }

    fn unary(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("unary: {:?}", tokens.peek());
        if let Some(t) = tokens.next_if(|t| matches!(t.lexeme, TokenType::Minus | TokenType::Bang))
        {
            let right = Parser::unary(tokens)?;
            Ok(ast_unary!(t, right))
        } else {
            Parser::primary(tokens)
        }
    }

    fn primary(tokens: &mut PeekableTokenIter) -> miette::Result<Ast> {
        trace!("primary: {:?}", tokens.peek());
        if let Some(token) = tokens.next_if(|t| {
            matches!(
                t.lexeme,
                TokenType::True | TokenType::False | TokenType::Nil
            )
        }) {
            Ok(ast_terminal!(token))
        } else if let Some(_left_paren) =
            tokens.next_if(|t| matches!(t.lexeme, TokenType::LeftParen))
        {
            let expr = Parser::expression(tokens)?;
            if let Some(_right_paren) = tokens.next_if(|t| t.lexeme == TokenType::RightParen) {
                Ok(ast_group!(expr))
            } else if tokens.peek().is_some() {
                // something other than a closing ')'
                Err(ParseError::MissingToken {
                    expected: TokenType::RightParen,
                    actual: tokens.next().unwrap().lexeme.clone(),
                }
                .into())
            } else {
                Err(ParseError::UnexpectedEof.into())
            }
        } else if let Some(token) = tokens.next_if(|t| {
            matches!(
                t.lexeme,
                TokenType::Number { .. } | TokenType::String { .. } | TokenType::Identifier { .. }
            )
        }) {
            Ok(ast_terminal!(token.clone()))
        } else {
            Err(ParseError::UnexpectedToken {
                actual: tokens.next().unwrap().clone(),
            }
            .into())
        }
    }
}
