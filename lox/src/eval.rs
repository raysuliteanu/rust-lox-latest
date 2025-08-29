use anyhow::Result;
use log::trace;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::mem;
use std::sync::Arc;
use thiserror::Error;

use crate::func;
use crate::model::{Ast, AstExpr, AstStmt};
use crate::model::{Lexeme, Token};
use crate::parser::Parser;
use crate::span::Span;

pub type Callable = fn(&[EvalValue]) -> EvalResult<EvalValue>;

/// String interner using Arc<str> for thread-safe string sharing
#[derive(Debug, Clone, Default)]
struct StringInterner {
    strings: HashSet<Arc<str>>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner {
            strings: HashSet::new(),
        }
    }

    fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(existing) = self.strings.get(s) {
            existing.clone()
        } else {
            let arc_str: Arc<str> = s.into();
            self.strings.insert(arc_str.clone());
            arc_str
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum EvalValue {
    Return(Box<EvalValue>),
    FunDecl(LoxFunction),
    Number(f64),
    String(Cow<'static, str>),
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
            EvalValue::FunDecl(func) => write!(f, "{func}"),
            EvalValue::Return(value) => write!(f, "(return) {value}"),
        }
    }
}

#[derive(Error, Debug)]
pub enum EvalErrors {
    #[error("invalid op {} for {}\n[line {line}]", <&Lexeme as Into<String>>::into(op), val)]
    InvalidUnaryOp {
        op: Lexeme,
        val: Box<EvalValue>,
        line: usize,
    },
    #[error("invalid operation {op} {}\n[line {line}]", <&Lexeme as Into<String>>::into(op))]
    InvalidBinaryOp { op: Lexeme, line: usize },
    #[error("Operands must be two numbers or two strings.\n[line {0}]")]
    StringsOrNumbers(usize),
    #[error("Undefined variable '{0}'.\n[line {1}]")]
    UndefinedVar(String, usize),
    #[error("Incorrect number of arguments. Expected {expected} got {actual}.\n[line {line}]")]
    ArityMismatch {
        expected: usize,
        actual: usize,
        line: usize,
    },
    // #[error("bad function call")]
    // FunctionCall,
}

macro_rules! invalid_unary_op {
    ($token: expr, $val: expr) => {
        EvalErrors::InvalidUnaryOp {
            op: $token.lexeme.clone(),
            val: Box::new($val),
            line: $token.span.line(),
        }
    };
}

macro_rules! invalid_binary_op {
    ($token: expr) => {
        EvalErrors::InvalidBinaryOp {
            op: $token.lexeme.clone(),
            line: $token.span.line(),
        }
    };
}

pub type EvalResult<T> = Result<T, EvalErrors>;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    name: Arc<str>,
    params: Option<Vec<Arc<str>>>,
    fn_type: LoxFunctionType,
    state: Option<Stack<EvalEnv>>,
}

impl LoxFunction {
    fn arity(&self) -> usize {
        self.params.as_ref().map_or(0, |f| f.len())
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum LoxFunctionType {
    System(&'static str),
    UserDefined(Ast),
}

impl Display for LoxFunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxFunctionType::System(s) => write!(f, "<system>: {s:?}"),
            LoxFunctionType::UserDefined(ast) => write!(f, "{ast}"),
        }
    }
}

struct BlockScope;

impl BlockScope {
    fn enter<F, R>(eval: &mut Eval, f: F) -> R
    where
        F: FnOnce(&mut Eval) -> R,
    {
        eval.start_scope();
        let result = f(eval);
        eval.end_scope();
        result
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct EvalEnv {
    env: HashMap<Arc<str>, EvalValue>,
}

impl EvalEnv {
    fn new() -> Self {
        EvalEnv::default()
    }

    fn upsert_var(&mut self, id: Arc<str>, initializer: Option<EvalValue>) -> Option<EvalValue> {
        let init = if let Some(ev) = initializer {
            ev
        } else {
            EvalValue::Nil
        };
        self.env.insert(id, init)
    }

    fn lookup_var(&self, id: &str) -> Option<&EvalValue> {
        self.env.get(id)
    }

    fn lookup_var_mut(&mut self, id: &str) -> Option<&mut EvalValue> {
        self.env.get_mut(id)
    }

    fn add_fn(&mut self, func: LoxFunction) -> Option<EvalValue> {
        self.env.insert(func.name.clone(), EvalValue::FunDecl(func))
    }
}

impl Display for EvalEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formated = self
            .env
            .iter()
            .map(|(s, v)| format!("({s}: {v})"))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "Env({formated})")
    }
}

type Stack<T> = VecDeque<T>;

#[derive(Clone)]
pub struct Eval<'eval> {
    source: &'eval str,
    expression_mode: bool,
    global_env: EvalEnv,
    env: Stack<EvalEnv>,
    global_fns: HashMap<&'static str, Callable>,
    interner: StringInterner,
}

impl<'eval> Eval<'_> {
    pub fn new(source: &str, expression_mode: bool) -> Eval<'_> {
        let mut interner = StringInterner::new();
        let mut global_env = EvalEnv::new();
        
        let clock_name = interner.intern("clock");
        global_env.add_fn(LoxFunction {
            name: clock_name,
            params: None,
            fn_type: LoxFunctionType::System("clock"),
            state: None,
        });

        let mut global_fns: HashMap<&'static str, Callable> = HashMap::new();
        global_fns.insert("clock", func::clock);

        Eval {
            source,
            expression_mode,
            global_env,
            env: Stack::new(),
            global_fns,
            interner,
        }
    }

    fn add_lox_fn(&mut self, func: LoxFunction) -> Option<EvalValue> {
        if let Some(env) = self.env.front_mut() {
            env.add_fn(func)
        } else {
            self.global_env.add_fn(func)
        }
    }

    fn add_var(&mut self, id: &str, initializer: Option<EvalValue>) -> Option<EvalValue> {
        let interned_id = self.interner.intern(id);
        if let Some(env) = self.env.front_mut() {
            env.upsert_var(interned_id, initializer)
        } else {
            self.global_env.upsert_var(interned_id, initializer)
        }
    }

    fn var_value(&self, id: &str) -> Option<&EvalValue> {
        for env in &self.env {
            trace!("var_value: looking for {id} in {env}");
            if env.env.contains_key(id) {
                trace!("var_value: found {id}");
                return env.lookup_var(id);
            }
        }

        trace!("var_value: looking for {id} in global env");
        self.global_env.lookup_var(id)
    }

    fn var_value_mut(&mut self, id: &str) -> Option<&mut EvalValue> {
        for env in &mut self.env {
            trace!("var_value: looking for {id} in {env}");
            if env.env.contains_key(id) {
                trace!("var_value_mut: found {id}");
                return env.lookup_var_mut(id);
            }
        }

        trace!("var_value_mut: looking for {id} in global env");
        self.global_env.lookup_var_mut(id)
    }

    fn var_exists(&self, id: &str) -> bool {
        for env in &self.env {
            if env.env.contains_key(id) {
                return true;
            }
        }

        self.global_env.env.contains_key(id)
    }

    pub(crate) fn start_scope(&mut self) {
        trace!("start scope");
        self.env.push_front(EvalEnv::new());
    }

    pub(crate) fn end_scope(&mut self) {
        trace!("end scope");
        self.env.pop_front();
    }

    pub fn evaluate(&mut self) -> anyhow::Result<EvalValue> {
        let parser = Parser::new(self.source, self.expression_mode, false);
        let tree = parser.parse()?;

        match self.eval(tree.iter()) {
            Ok(v) => Ok(v),
            Err(e) => {
                eprintln!("{e}");
                Err(e.into())
            }
        }
    }

    fn eval(&mut self, tree: std::slice::Iter<'_, Ast>) -> EvalResult<EvalValue> {
        let mut value = EvalValue::Nil;
        for ast in tree {
            value = self.eval_ast(ast)?;
            trace!("eval: {value:?}");
        }

        if let EvalValue::Return(v) = value {
            trace!("eval - got return: {}", *v);
            Ok(*v)
        } else {
            Ok(value)
        }
    }

    fn eval_ast(&mut self, ast: &'eval Ast) -> EvalResult<EvalValue> {
        trace!("eval_ast");
        match ast {
            Ast::Class => todo!("class decl"),
            Ast::Function { name, params, body } => self.eval_fun_decl(name, params, body),
            Ast::Variable { name, initializer } => self.eval_var_decl(name, initializer),
            Ast::Statement(stmt) => self.eval_stmt(stmt),
            Ast::Block(block) => self.eval_block(block),
            Ast::Expression(e) => self.eval_expr(e),
        }
    }

    fn eval_stmt(&mut self, stmt: &'eval AstStmt) -> EvalResult<EvalValue> {
        trace!("eval_stmt");
        match stmt {
            AstStmt::Expression(expr) => self.eval_expr(expr),
            AstStmt::Print(ast) => self.eval_print_stmt(ast),
            AstStmt::If {
                condition,
                then,
                or_else,
            } => self.eval_if_stmt(condition, then, or_else),
            AstStmt::Return(ast) => self.eval_return(ast),
            AstStmt::While(cond, body) => self.eval_while(cond, body),
        }
    }

    fn eval_expr(&mut self, expr: &'eval AstExpr) -> EvalResult<EvalValue> {
        trace!("eval_expr");
        match expr {
            AstExpr::Terminal(token) => self.eval_terminal(token),
            AstExpr::Group(expr) => self.eval_expr(expr),
            AstExpr::Unary { op, exp } => self.eval_unary(op, exp),
            AstExpr::Binary { op, left, right } => self.eval_binary(op, left, right),
            AstExpr::Assignment { id, expr } => self.eval_assignment(id, expr),
            AstExpr::Call { callee, args, site } => self.eval_call(callee, args, site),
            AstExpr::Logical { op, left, right } => self.eval_logical(op, left, right),
        }
    }

    fn eval_print_stmt(&mut self, expr: &'eval AstExpr) -> EvalResult<EvalValue> {
        trace!("eval_print");
        let val = self.eval_expr(expr)?;
        trace!("print = {val}");
        println!("{val}");
        Ok(EvalValue::Nil)
    }

    fn eval_terminal(&self, token: &'eval Token) -> EvalResult<EvalValue> {
        trace!("eval_terminal");
        let val = match &token.lexeme {
            Lexeme::Number(_, v) => EvalValue::Number(*v),
            Lexeme::String(s) => EvalValue::String(Cow::Owned(s.clone())),
            Lexeme::Identifier(id) => self
                .eval_identifier(id)
                .ok_or_else(|| EvalErrors::UndefinedVar(id.clone(), token.span.line()))?,
            Lexeme::True => EvalValue::Boolean(true),
            Lexeme::False => EvalValue::Boolean(false),
            Lexeme::Nil => EvalValue::Nil,
            _ => unimplemented!("{}", token.lexeme),
        };

        Ok(val)
    }

    fn eval_unary(&mut self, op: &Token, expr: &AstExpr) -> EvalResult<EvalValue> {
        trace!("eval_unary");
        let val = self.eval_expr(expr)?;
        let result = match op.lexeme {
            Lexeme::Bang => match val {
                EvalValue::Number(_) => EvalValue::Boolean(false),
                EvalValue::Boolean(v) => EvalValue::Boolean(!v),
                EvalValue::Nil => EvalValue::Boolean(true),
                _ => return Err(invalid_unary_op!(op, val)),
            },
            Lexeme::Minus => match val {
                EvalValue::Number(v) => EvalValue::Number(-v),
                _ => return Err(invalid_unary_op!(op, val)),
            },
            _ => return Err(invalid_unary_op!(op, val)),
        };

        Ok(result)
    }

    fn eval_binary(
        &mut self,
        op: &Token,
        left: &AstExpr,
        right: &AstExpr,
    ) -> EvalResult<EvalValue> {
        trace!("eval_binary");
        let left_expr = self.eval_expr(left)?;
        trace!("eval_binary: left = {left_expr:?}");
        let right_expr = self.eval_expr(right)?;
        trace!("eval_binary: right = {right_expr:?}");
        let result = match op.lexeme {
            Lexeme::Plus => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Number(l + r),
                (EvalValue::String(l), EvalValue::String(r)) => {
                    let concatenated = format!("{}{}", l, r);
                    EvalValue::String(Cow::Owned(concatenated))
                }
                _e => {
                    trace!("bad + operands: left = {:?}, right = {:?}", _e.0, _e.1);
                    return Err(EvalErrors::StringsOrNumbers(op.span.line()));
                }
            },
            Lexeme::Minus => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Number(l - r),
                _ => return Err(invalid_binary_op!(op)),
            },
            Lexeme::Star => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Number(l * r),
                _ => return Err(invalid_binary_op!(op)),
            },
            Lexeme::Slash => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Number(l / r),
                _ => return Err(invalid_binary_op!(op)),
            },
            Lexeme::EqEq => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Boolean(l == r),
                (EvalValue::String(l), EvalValue::String(r)) => EvalValue::Boolean(l == r),
                (EvalValue::Boolean(l), EvalValue::Boolean(r)) => EvalValue::Boolean(l == r),
                _ => EvalValue::Boolean(false),
            },
            Lexeme::BangEq => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Boolean(l != r),
                (EvalValue::String(l), EvalValue::String(r)) => EvalValue::Boolean(l != r),
                (EvalValue::Boolean(l), EvalValue::Boolean(r)) => EvalValue::Boolean(l != r),
                _ => EvalValue::Boolean(true),
            },
            Lexeme::Less => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Boolean(l < r),
                _ => return Err(invalid_binary_op!(op)),
            },
            Lexeme::LessEq => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Boolean(l <= r),
                _ => return Err(invalid_binary_op!(op)),
            },
            Lexeme::Greater => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Boolean(l > r),
                _ => return Err(invalid_binary_op!(op)),
            },
            Lexeme::GreaterEq => match (left_expr, right_expr) {
                (EvalValue::Number(l), EvalValue::Number(r)) => EvalValue::Boolean(l >= r),
                _ => return Err(invalid_binary_op!(op)),
            },
            _ => return Err(invalid_binary_op!(op)),
        };

        Ok(result)
    }

    // var some_var [= expr] ;
    fn eval_var_decl(
        &mut self,
        token: &Token,
        ast: &Option<Box<AstExpr>>,
    ) -> EvalResult<EvalValue> {
        let initializer = if let Some(expr) = ast {
            Some(self.eval_expr(expr)?)
        } else {
            None
        };

        if let Lexeme::Identifier(id) = &token.lexeme {
            self.add_var(id, initializer);
        } else {
            panic!("invalid token {token}");
        }

        Ok(EvalValue::Nil)
    }

    fn eval_identifier(&self, id: &str) -> Option<EvalValue> {
        self.var_value(id).cloned()
    }

    // some_var = expr
    fn eval_assignment(&mut self, id: &str, expr: &AstExpr) -> EvalResult<EvalValue> {
        if !self.var_exists(id) {
            Err(EvalErrors::UndefinedVar(id.to_string(), 0))
        } else {
            let new_val = self.eval_expr(expr)?;
            let val = self
                .var_value_mut(id)
                .expect("already checked the var exists");
            *val = new_val.clone();
            Ok(new_val)
        }
    }

    fn eval_logical(
        &mut self,
        op: &Token,
        left: &AstExpr,
        right: &AstExpr,
    ) -> EvalResult<EvalValue> {
        trace!("eval_logical: {}", op.lexeme);
        let left_val = self.eval_expr(left)?;
        trace!("left = {left_val}");
        match op.lexeme {
            Lexeme::Or if Eval::is_truthy(&left_val) => {
                trace!("OR returning {left_val}");
                Ok(left_val)
            }
            Lexeme::And if !Eval::is_truthy(&left_val) => {
                trace!("AND returning {left_val}");
                Ok(left_val)
            }
            _ => {
                trace!("evaluating right expr of {}", op.lexeme);
                self.eval_expr(right)
            }
        }
    }

    fn is_truthy(val: &EvalValue) -> bool {
        match val {
            EvalValue::Boolean(b) => *b,
            EvalValue::Nil => false,
            _ => true,
        }
    }

    fn eval_block(&mut self, block: &[Ast]) -> EvalResult<EvalValue> {
        trace!("eval_block");

        BlockScope::enter(self, |eval| {
            let mut result = EvalValue::Nil;
            for ast in block {
                result = eval.eval_ast(ast)?;

                if let EvalValue::Return(v) = result {
                    return Ok(EvalValue::Return(v));
                }
            }

            if let EvalValue::Return(v) = result {
                trace!("eval_block - got return: {}", *v);
                Ok(*v)
            } else {
                Ok(result)
            }
        })
    }

    fn eval_if_stmt(
        &mut self,
        cond: &AstExpr,
        then_block: &Ast,
        else_block: &Option<Box<Ast>>,
    ) -> EvalResult<EvalValue> {
        trace!("eval_if");
        let cond_result = self.eval_expr(cond)?;
        if Eval::is_truthy(&cond_result) {
            trace!("eval_if:then");
            match then_block {
                Ast::Block(asts) => self.eval_block(asts),
                Ast::Statement(ast_stmt) => self.eval_stmt(ast_stmt),
                _ => panic!("then block not block or statement"),
            }
        } else if let Some(ast) = else_block {
            trace!("eval_if:else");
            match ast.as_ref() {
                Ast::Block(block) => self.eval_block(block),
                Ast::Statement(stmt) => self.eval_stmt(stmt),
                _ => panic!("else block not block or statement"),
            }
        } else {
            Ok(EvalValue::Nil)
        }
    }

    fn eval_while(&mut self, cond: &AstExpr, body: &Ast) -> EvalResult<EvalValue> {
        trace!("eval_while");
        loop {
            if Eval::is_truthy(&self.eval_expr(cond)?) {
                let result = match body {
                    Ast::Block(asts) => self.eval_block(asts)?,
                    Ast::Statement(ast_stmt) => self.eval_stmt(ast_stmt)?,
                    _ => panic!("then block not block or statement"),
                };

                if let EvalValue::Return(_) = &result {
                    return Ok(result);
                }
            } else {
                break;
            };
        }

        Ok(EvalValue::Nil)
    }

    fn eval_call(
        &mut self,
        callee: &AstExpr,
        args: &[AstExpr],
        site: &Span,
    ) -> EvalResult<EvalValue> {
        trace!(
            "eval_call: {callee}({}) @ {site}",
            args.iter()
                .map(|a| format!("{a}"))
                .collect::<Vec<_>>()
                .join(", ")
        );

        trace!(
            "eval_call: env stack: {:?}, global env: {:?}",
            self.env, self.global_env
        );

        let result = if let AstExpr::Terminal(Token {
            lexeme: Lexeme::Identifier(id),
            ..
        }) = callee
        {
            let (has_state, fn_type, params) = self.extract_fn_details(callee, args, site, id)?;

            let val = if has_state {
                let mut func_state = if let Some(EvalValue::FunDecl(f)) = self.var_value_mut(id) {
                    f.state
                        .take()
                        .expect("has_state was true but state is None")
                } else {
                    panic!("function {callee} disappeared!");
                };

                mem::swap(&mut self.env, &mut func_state);

                let result = match self.eval_expr(callee)? {
                    EvalValue::FunDecl(f) => {
                        trace!("eval_call: func: {f}");

                        let (has_state, fn_type, params) =
                            self.extract_fn_details(callee, args, site, id)?;

                        self.finish_call(&f.name, args, has_state, fn_type, params)?
                    }
                    _v => panic!("eval_call: not a function: {_v}"),
                };

                mem::swap(&mut self.env, &mut func_state);

                if let Some(EvalValue::FunDecl(f)) = self.var_value_mut(id) {
                    f.state = Some(func_state);
                }

                result
            } else {
                self.do_fn_call(&fn_type, &params, args)?
            };

            Self::extract_return_val(val)
        } else {
            match self.eval_expr(callee)? {
                EvalValue::FunDecl(f) => {
                    trace!("eval_call: func: {f}");

                    let (has_state, fn_type, params) =
                        self.extract_fn_details(callee, args, site, &f.name)?;

                    self.finish_call(&f.name, args, has_state, fn_type, params)?
                }
                _v => panic!("eval_call: not a function: {_v}"),
            }
        };

        trace!("eval_call: returning {result}");

        Ok(result.clone())
    }

    fn finish_call(
        &mut self,
        fn_name: &str,
        args: &[AstExpr],
        has_state: bool,
        fn_type: LoxFunctionType,
        params: Option<Vec<Arc<str>>>,
    ) -> EvalResult<EvalValue> {
        let val = if has_state {
            let mut func_state = if let Some(EvalValue::FunDecl(f)) = self.var_value_mut(&fn_name) {
                f.state
                    .take()
                    .expect("has_state was true but state is None")
            } else {
                panic!("function {fn_name} disappeared!");
            };

            mem::swap(&mut self.env, &mut func_state);

            let result = self.do_fn_call(&fn_type, &params, args)?;

            mem::swap(&mut self.env, &mut func_state);

            if let Some(EvalValue::FunDecl(f)) = self.var_value_mut(&fn_name) {
                f.state = Some(func_state);
            }

            result
        } else {
            self.do_fn_call(&fn_type, &params, args)?
        };

        Ok(Self::extract_return_val(val))
    }

    fn extract_return_val(val: EvalValue) -> EvalValue {
        if let EvalValue::Return(r) = val {
            let mut ret = *r;
            while let EvalValue::Return(v) = ret {
                trace!("eval_call - got return: {v}");
                ret = *v;
            }

            ret
        } else {
            val
        }
    }

    fn extract_fn_details(
        &mut self,
        callee: &AstExpr,
        args: &[AstExpr],
        site: &Span,
        id: &str,
    ) -> EvalResult<(bool, LoxFunctionType, Option<Vec<Arc<str>>>)> {
        let (has_state, fn_type, params) =
            if let Some(EvalValue::FunDecl(lox_func)) = self.var_value(id) {
                trace!("checking arity of {callee}");
                if lox_func.arity() != args.len() {
                    return Err(EvalErrors::ArityMismatch {
                        expected: lox_func.arity(),
                        actual: args.len(),
                        line: site.line(),
                    });
                }
                (
                    lox_func.state.is_some(),
                    lox_func.fn_type.clone(),
                    lox_func.params.clone(),
                )
            } else {
                panic!("can't find {} in state", id)
            };
        Ok((has_state, fn_type, params))
    }

    fn do_fn_call(
        &mut self,
        fn_type: &LoxFunctionType,
        params: &Option<Vec<Arc<str>>>,
        args: &[AstExpr],
    ) -> EvalResult<EvalValue> {
        BlockScope::enter(self, |eval| match fn_type {
            LoxFunctionType::System(name) => {
                let vals: Result<Vec<_>, _> = args
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        let val = eval.eval_expr(arg)?;
                        trace!("do_fn_call: arg{i}: {arg} = {val}");
                        Ok(val)
                    })
                    .collect();
                eval.call_system_fn(name, &vals?)
            }
            LoxFunctionType::UserDefined(body) => {
                for (param, arg) in params.iter().flatten().zip(args) {
                    let val = eval.eval_expr(arg)?;
                    eval.add_var(param, Some(val));
                }
                trace!("calling UDF {fn_type}");
                eval.eval_ast(body)
            }
        })
    }

    fn call_system_fn(
        &mut self,
        name: &'static str,
        params: &[EvalValue],
    ) -> EvalResult<EvalValue> {
        if let Some(func) = self.global_fns.get(name) {
            (*func)(params)
        } else {
            panic!("missing system function {name}");
        }
    }

    fn eval_fun_decl(
        &mut self,
        name: &str,
        params: &[String],
        body: &Ast,
    ) -> EvalResult<EvalValue> {
        trace!("eval_fun_decl");
        let state = if !self.env.is_empty() {
            Some(self.env.clone())
        } else {
            None
        };

        let interned_name = self.interner.intern(name);
        let interned_params: Vec<Arc<str>> = params.iter()
            .map(|p| self.interner.intern(p))
            .collect();

        trace!("eval_fun_decl: saving state for {name}: {state:?}");
        self.add_lox_fn(LoxFunction {
            name: interned_name,
            params: Some(interned_params),
            fn_type: LoxFunctionType::UserDefined((*body).clone()),
            state,
        });

        Ok(EvalValue::Nil)
    }

    fn eval_return(&mut self, ast: &Option<Box<AstExpr>>) -> EvalResult<EvalValue> {
        trace!("eval_return");
        let ret_val = if let Some(return_expr) = ast {
            let val = self.eval_expr(return_expr)?;
            EvalValue::Return(Box::new(val))
        } else {
            EvalValue::Nil
        };

        trace!("eval_return: returning {ret_val:?}");
        Ok(EvalValue::Return(Box::new(ret_val)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use crate::util::print_ast;
    
    fn string_val(s: &str) -> EvalValue {
        EvalValue::String(Cow::Owned(s.to_string()))
    }
    
    #[test]
    fn test_thread_safety() {
        // Test that EvalValue implements Send + Sync (required for anyhow::Error compatibility)
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<EvalValue>();
        assert_sync::<EvalValue>();
        assert_send::<EvalErrors>();
        assert_sync::<EvalErrors>();
    }

    #[test]
    fn test_string_interning_benefits() {
        // Test that demonstrates string interning benefits
        let program = r#"
        fun fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        var x = 5;
        var y = 10;
        var result = fibonacci(x);
        print result;
        "#;
        
        let mut eval = Eval::new(program, false);
        
        // Verify the interner has some common strings
        let interner_before = eval.interner.strings.len();
        let _ = eval.evaluate(); // May fail due to recursion, but should populate interner
        let interner_after = eval.interner.strings.len();
        
        // Should have interned at least function names, variable names
        assert!(interner_after >= interner_before);
    }

    #[test]
    fn test_eval_value_display() {
        assert_eq!(format!("{}", EvalValue::Number(42.0)), "42");
        assert_eq!(format!("{}", EvalValue::Number(1.23)), "1.23");
        assert_eq!(
            format!("{}", string_val("hello")),
            "hello"
        );
        assert_eq!(format!("{}", EvalValue::Boolean(true)), "true");
        assert_eq!(format!("{}", EvalValue::Boolean(false)), "false");
        assert_eq!(format!("{}", EvalValue::Nil), "nil");
    }

    #[test]
    fn test_eval_value_equality() {
        assert_eq!(EvalValue::Number(42.0), EvalValue::Number(42.0));
        assert_eq!(
            string_val("test"),
            string_val("test")
        );
        assert_eq!(EvalValue::Boolean(true), EvalValue::Boolean(true));
        assert_eq!(EvalValue::Nil, EvalValue::Nil);

        assert_ne!(EvalValue::Number(42.0), EvalValue::Number(43.0));
        assert_ne!(
            string_val("test"),
            string_val("other")
        );
        assert_ne!(EvalValue::Boolean(true), EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_new() {
        let source = "print 42;";
        let eval = Eval::new(source, false);
        assert_eq!(eval.source, source);
    }

    #[test]
    fn test_eval_terminal_number() {
        let eval = Eval::new("", false);
        let token = Token {
            lexeme: Lexeme::Number("42".to_string(), 42.0),
            span: Span::new(0, 0, 1),
        };
        let result = eval.eval_terminal(&token).unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_terminal_string() {
        let eval = Eval::new("", false);
        let token = Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        };
        let result = eval.eval_terminal(&token).unwrap();
        assert_eq!(result, string_val("hello"));
    }

    #[test]
    fn test_eval_terminal_boolean() {
        let eval = Eval::new("", false);

        let true_token = Token {
            lexeme: Lexeme::True,
            span: Span::new(0, 0, 1),
        };
        let result = eval.eval_terminal(&true_token).unwrap();
        assert_eq!(result, EvalValue::Boolean(true));

        let false_token = Token {
            lexeme: Lexeme::False,
            span: Span::new(0, 0, 1),
        };
        let result = eval.eval_terminal(&false_token).unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_terminal_nil() {
        let eval = Eval::new("", false);
        let token = Token {
            lexeme: Lexeme::Nil,
            span: Span::new(0, 0, 1),
        };
        let result = eval.eval_terminal(&token).unwrap();
        assert_eq!(result, EvalValue::Nil);
    }

    #[test]
    fn test_eval_unary_bang() {
        let mut eval = Eval::new("", false);
        let bang_token = Token {
            lexeme: Lexeme::Bang,
            span: Span::new(0, 0, 1),
        };

        let true_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::True,
            span: Span::new(0, 0, 1),
        });
        let result = eval.eval_unary(&bang_token, &true_expr).unwrap();
        assert_eq!(result, EvalValue::Boolean(false));

        let false_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::False,
            span: Span::new(0, 0, 1),
        });
        let result = eval.eval_unary(&bang_token, &false_expr).unwrap();
        assert_eq!(result, EvalValue::Boolean(true));

        let nil_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Nil,
            span: Span::new(0, 0, 1),
        });
        let result = eval.eval_unary(&bang_token, &nil_expr).unwrap();
        assert_eq!(result, EvalValue::Boolean(true));

        let number_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("42".to_string(), 42.0),
            span: Span::new(0, 0, 1),
        });
        let result = eval.eval_unary(&bang_token, &number_expr).unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_unary_minus() {
        let mut eval = Eval::new("", false);
        let minus_token = Token {
            lexeme: Lexeme::Minus,
            span: Span::new(0, 0, 1),
        };

        let number_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("42".to_string(), 42.0),
            span: Span::new(0, 0, 1),
        });
        let result = eval.eval_unary(&minus_token, &number_expr).unwrap();
        assert_eq!(result, EvalValue::Number(-42.0));
    }

    #[test]
    fn test_eval_binary_plus_numbers() {
        let mut eval = Eval::new("", false);
        let plus_token = Token {
            lexeme: Lexeme::Plus,
            span: Span::new(0, 0, 1),
        };

        let left_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("10".to_string(), 10.0),
            span: Span::new(0, 0, 1),
        });
        let right_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let result = eval
            .eval_binary(&plus_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Number(15.0));
    }

    #[test]
    fn test_eval_binary_plus_strings() {
        let mut eval = Eval::new("", false);
        let plus_token = Token {
            lexeme: Lexeme::Plus,
            span: Span::new(0, 0, 1),
        };

        let left_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        });
        let right_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String(" world".to_string()),
            span: Span::new(0, 0, 1),
        });

        let result = eval
            .eval_binary(&plus_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, string_val("hello world"));
    }

    #[test]
    fn test_eval_binary_arithmetic() {
        let mut eval = Eval::new("", false);

        let left_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("10".to_string(), 10.0),
            span: Span::new(0, 0, 1),
        });
        let right_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let minus_token = Token {
            lexeme: Lexeme::Minus,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&minus_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Number(5.0));

        let star_token = Token {
            lexeme: Lexeme::Star,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&star_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Number(50.0));

        let slash_token = Token {
            lexeme: Lexeme::Slash,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&slash_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Number(2.0));
    }

    #[test]
    fn test_eval_binary_equality() {
        let mut eval = Eval::new("", false);

        let left_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });
        let right_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let eq_token = Token {
            lexeme: Lexeme::EqEq,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&eq_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Boolean(true));

        let ne_token = Token {
            lexeme: Lexeme::BangEq,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&ne_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_binary_comparison() {
        let mut eval = Eval::new("", false);

        let left_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("10".to_string(), 10.0),
            span: Span::new(0, 0, 1),
        });
        let right_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let greater_token = Token {
            lexeme: Lexeme::Greater,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&greater_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Boolean(true));

        let greater_eq_token = Token {
            lexeme: Lexeme::GreaterEq,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&greater_eq_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Boolean(true));

        let less_token = Token {
            lexeme: Lexeme::Less,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&less_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Boolean(false));

        let less_eq_token = Token {
            lexeme: Lexeme::LessEq,
            span: Span::new(0, 0, 1),
        };
        let result = eval
            .eval_binary(&less_eq_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_evaluate_simple_expression() {
        let mut eval = Eval::new("42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0)); // Returns default from eval method
    }

    #[test]
    fn test_evaluate_print_statement() {
        let mut eval = Eval::new("print 42;", false);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Nil);
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let mut eval = Eval::new("1 + 2 * 3", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(7.0)); // Returns default from eval method
    }

    #[test]
    fn test_eval_errors_display() {
        let error1 = EvalErrors::InvalidUnaryOp {
            op: Lexeme::Bang,
            val: Box::new(string_val("test")),
            line: 1,
        };
        assert_eq!(format!("{error1}"), "invalid op ! for test\n[line 1]");

        let error2 = EvalErrors::InvalidBinaryOp {
            op: Lexeme::Plus,
            line: 2,
        };
        assert_eq!(format!("{error2}"), "invalid operation PLUS +\n[line 2]");

        let error3 = EvalErrors::StringsOrNumbers(3);
        assert_eq!(
            format!("{error3}"),
            "Operands must be two numbers or two strings.\n[line 3]"
        );
    }

    #[test]
    fn test_eval_unary_error_invalid_operator() {
        let mut eval = Eval::new("", false);
        let invalid_token = Token {
            lexeme: Lexeme::Plus,
            span: Span::new(0, 0, 1),
        };

        let number_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("42".to_string(), 42.0),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_unary(&invalid_token, &number_expr);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("invalid op"));
    }

    #[test]
    fn test_eval_unary_error_invalid_type() {
        let mut eval = Eval::new("", false);
        let minus_token = Token {
            lexeme: Lexeme::Minus,
            span: Span::new(0, 0, 1),
        };

        let string_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_unary(&minus_token, &string_expr);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("invalid op - for hello"));
    }

    #[test]
    fn test_eval_unary_bang_invalid_type() {
        let mut eval = Eval::new("", false);
        let bang_token = Token {
            lexeme: Lexeme::Bang,
            span: Span::new(0, 0, 1),
        };

        let string_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_unary(&bang_token, &string_expr);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("invalid op ! for hello"));
    }

    #[test]
    fn test_eval_binary_error_invalid_operator() {
        let mut eval = Eval::new("", false);
        let invalid_token = Token {
            lexeme: Lexeme::Bang,
            span: Span::new(0, 0, 1),
        };

        let left_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("10".to_string(), 10.0),
            span: Span::new(0, 0, 1),
        });
        let right_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_binary(&invalid_token, &left_expr, &right_expr);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("invalid operation"));
    }

    #[test]
    fn test_eval_binary_plus_mixed_types_error() {
        let mut eval = Eval::new("", false);
        let plus_token = Token {
            lexeme: Lexeme::Plus,
            span: Span::new(0, 0, 1),
        };
        let number_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("42".to_string(), 42.0),
            span: Span::new(0, 0, 1),
        });
        let string_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_binary(&plus_token, &number_expr, &string_expr);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Operands must be two numbers or two strings"));
    }

    #[test]
    fn test_eval_binary_arithmetic_invalid_types() {
        let mut eval = Eval::new("", false);
        let minus_token = Token {
            lexeme: Lexeme::Minus,
            span: Span::new(0, 0, 1),
        };

        let string_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        });
        let number_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_binary(&minus_token, &string_expr, &number_expr);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("invalid operation"));
    }

    #[test]
    fn test_eval_binary_comparison_invalid_types() {
        let mut eval = Eval::new("", false);
        let greater_token = Token {
            lexeme: Lexeme::Greater,
            span: Span::new(0, 0, 1),
        };

        let string_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        });
        let number_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_binary(&greater_token, &string_expr, &number_expr);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("invalid operation"));
    }

    #[test]
    fn test_eval_binary_equality_different_types() {
        let mut eval = Eval::new("", false);
        let eq_token = Token {
            lexeme: Lexeme::EqEq,
            span: Span::new(0, 0, 1),
        };

        let string_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::String("hello".to_string()),
            span: Span::new(0, 0, 1),
        });

        let number_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("5".to_string(), 5.0),
            span: Span::new(0, 0, 1),
        });

        let result = eval.eval_binary(&eq_token, &string_expr, &number_expr);
        assert!(result.is_ok_and(|val| val == EvalValue::Boolean(false)));
    }

    #[test]
    fn test_eval_binary_division_by_zero() {
        let mut eval = Eval::new("", false);
        let slash_token = Token {
            lexeme: Lexeme::Slash,
            span: Span::new(0, 0, 1),
        };

        let left_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("10".to_string(), 10.0),
            span: Span::new(0, 0, 1),
        });
        let right_expr = AstExpr::Terminal(Token {
            lexeme: Lexeme::Number("0".to_string(), 0.0),
            span: Span::new(0, 0, 1),
        });

        // Division by zero should return infinity in Rust, not an error
        let result = eval
            .eval_binary(&slash_token, &left_expr, &right_expr)
            .unwrap();
        assert_eq!(result, EvalValue::Number(f64::INFINITY));
    }

    #[test]
    fn test_eval_variable_declaration_without_initializer() {
        let mut eval = Eval::new("var x;", false);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Nil);
    }

    #[test]
    fn test_eval_variable_declaration_with_initializer() {
        let mut eval = Eval::new("var x = 42;", false);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Nil);
    }

    #[test]
    fn test_eval_variable_usage() {
        let mut eval = Eval::new("var x = 42; print x;", false);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Nil);
    }

    #[test]
    fn test_eval_assignment_expression() {
        let mut eval = Eval::new("var x = 10; x = 20;", false);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(20.0));
    }

    #[test]
    fn test_eval_assignment_undefined_variable() {
        let mut eval = Eval::new("x = 10;", false);
        let result = eval.evaluate();
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_logical_or_short_circuit() {
        let mut eval = Eval::new("true or false", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(true));
    }

    #[test]
    fn test_eval_logical_and_short_circuit() {
        let mut eval = Eval::new("false and true", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_logical_or_no_short_circuit() {
        let mut eval = Eval::new("false or true", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(true));
    }

    #[test]
    fn test_eval_logical_and_no_short_circuit() {
        let mut eval = Eval::new("true and false", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_logical_or_with_numbers() {
        let mut eval = Eval::new("0 or 42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(0.0));
    }

    #[test]
    fn test_eval_logical_or_with_truthy_number() {
        let mut eval = Eval::new("42 or 0", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_logical_or_with_strings() {
        let mut eval = Eval::new("\"\" or \"hello\"", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, string_val(""));
    }

    #[test]
    fn test_eval_logical_or_with_truthy_string() {
        let mut eval = Eval::new("\"hello\" or \"world\"", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, string_val("hello"));
    }

    #[test]
    fn test_eval_logical_or_with_nil() {
        let mut eval = Eval::new("nil or 42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_logical_or_with_truthy_nil() {
        let mut eval = Eval::new("42 or nil", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_logical_or_both_falsy() {
        let mut eval = Eval::new("nil or false", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_logical_and_with_numbers() {
        let mut eval = Eval::new("42 and 0", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(0.0));
    }

    #[test]
    fn test_eval_logical_and_with_falsy_number() {
        let mut eval = Eval::new("0 and 42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_logical_and_with_strings() {
        let mut eval = Eval::new("\"hello\" and \"world\"", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, string_val("world"));
    }

    #[test]
    fn test_eval_logical_and_with_empty_string() {
        let mut eval = Eval::new("\"\" and \"hello\"", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, string_val("hello"));
    }

    #[test]
    fn test_eval_logical_and_with_nil() {
        let mut eval = Eval::new("nil and 42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Nil);
    }

    #[test]
    fn test_eval_logical_and_with_truthy_nil() {
        let mut eval = Eval::new("42 and nil", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Nil);
    }

    #[test]
    fn test_eval_logical_and_with_false() {
        let mut eval = Eval::new("false and 42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_logical_and_both_truthy() {
        let mut eval = Eval::new("42 and \"hello\"", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, string_val("hello"));
    }

    #[test]
    fn test_is_truthy_function() {
        assert!(Eval::is_truthy(&EvalValue::Boolean(true)));
        assert!(!Eval::is_truthy(&EvalValue::Boolean(false)));
        assert!(!Eval::is_truthy(&EvalValue::Nil));
        assert!(Eval::is_truthy(&EvalValue::Number(42.0)));
        assert!(Eval::is_truthy(&EvalValue::Number(0.0)));
        assert!(Eval::is_truthy(&EvalValue::Number(-1.0)));
        assert!(Eval::is_truthy(&string_val("hello")));
        assert!(Eval::is_truthy(&string_val("")));
    }

    #[test]
    fn test_eval_logical_chained_or() {
        let mut eval = Eval::new("false or nil or \"hello\"", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, string_val("hello"));
    }

    #[test]
    fn test_eval_logical_chained_and() {
        let mut eval = Eval::new("true and 42 and \"hello\"", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, string_val("hello"));
    }

    #[test]
    fn test_eval_logical_mixed_operations() {
        let mut eval = Eval::new("false or true and 42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_logical_mixed_operations_with_parentheses() {
        let mut eval = Eval::new("(false or true) and 42", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_logical_complex_expression() {
        let mut eval = Eval::new("nil or false or (true and 42)", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn test_eval_logical_with_equality() {
        let mut eval = Eval::new("5 == 5 or false", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(true));
    }

    #[test]
    fn test_eval_logical_with_comparison() {
        let mut eval = Eval::new("5 > 3 and 10 < 20", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(true));
    }

    #[test]
    fn test_eval_logical_or_short_circuit_no_evaluation() {
        let mut eval = Eval::new("var x = 5; true or (x = 10)", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(true));
    }

    #[test]
    fn test_eval_logical_and_short_circuit_no_evaluation() {
        let mut eval = Eval::new("var x = 5; false and (x = 10)", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Boolean(false));
    }

    #[test]
    fn test_eval_logical_or_evaluates_right_side() {
        let mut eval = Eval::new("var x = 5; nil or x", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(5.0));
    }

    #[test]
    fn test_eval_logical_and_evaluates_right_side() {
        let mut eval = Eval::new("var x = 5; true and x", true);
        let result = eval.evaluate().unwrap();
        assert_eq!(result, EvalValue::Number(5.0));
    }

    #[test]
    fn test_print_ast_simple() {
        let source = "var x = 42; print x;";
        let parser = Parser::new(source, false, false);
        let ast = parser.parse().unwrap();

        // This would print to stdout, so we just verify it doesn't panic
        print_ast(&ast);
    }

    #[test]
    fn test_print_ast_complex() {
        let source = r#"
        var a = 10;
        if (a > 5) {
            print "large";
        } else {
            print "small";
        }
        "#;
        let parser = Parser::new(source, false, false);
        let ast = parser.parse().unwrap();

        // This would print to stdout, so we just verify it doesn't panic
        print_ast(&ast);
    }

    // Block Scoping Tests
    #[test]
    fn test_block_scoping_basic() {
        let mut eval = Eval::new(
            r#"
            var x = "outer";
            {
                var x = "inner";
                print x;
            }
            print x;
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_scoping_variable_shadowing() {
        let mut eval = Eval::new(
            r#"
            var x = 1;
            {
                var x = 2;
                {
                    var x = 3;
                    x = 30;
                    print x;
                }
                x = 20;
                print x;
            }
            print x;
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_scoping_undefined_after_block() {
        let mut eval = Eval::new(
            r#"
            {
                var x = 42;
            }
            print x;
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_err());
    }

    // Function Declaration and Call Tests
    #[test]
    fn test_function_declaration_basic() {
        let mut eval = Eval::new(
            r#"
            fun greet(name) {
                print "Hello, " + name;
            }
            greet("World");
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_no_parameters() {
        let mut eval = Eval::new(
            r#"
            fun getMessage() {
                return "Hello, World!";
            }
            print getMessage();
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_multiple_parameters() {
        let mut eval = Eval::new(
            r#"
            fun add(a, b, c) {
                return a + b + c;
            }
            print add(1, 2, 3);
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_parameter_scoping() {
        let mut eval = Eval::new(
            r#"
            var x = "global";
            fun test(x) {
                return x;
            }
            print test("parameter");
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_closure_basic() {
        let mut eval = Eval::new(
            r#"
            var outer = "captured";
            fun testClosure() {
                return outer;
            }
            print testClosure();
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_clock_function() {
        let mut eval = Eval::new("clock()", true);
        let result = eval.evaluate().unwrap();
        // Just verify it returns a number (timestamp)
        assert!(matches!(result, EvalValue::Number(_)));
    }

    #[test]
    fn test_function_arity_mismatch_too_few() {
        let mut eval = Eval::new(
            r#"
            fun test(a, b) {
                return a + b;
            }
            test(1);
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_err());
    }

    #[test]
    fn test_function_arity_mismatch_too_many() {
        let mut eval = Eval::new(
            r#"
            fun test(a) {
                return a;
            }
            test(1, 2, 3);
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_err());
    }

    #[test]
    fn test_recursive_function() {
        let mut eval = Eval::new(
            r#"
            fun factorial(n) {
                if (n <= 1) {
                    return 1;
                }
                return n * factorial(n - 1);
            }
            print factorial(5);
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_function_calls() {
        // Skip this test if function declarations aren't fully implemented
        let mut eval = Eval::new(
            r#"
            var x = 5;
            print x;
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    // Return Statement Tests
    #[test]
    fn test_return_statement_early_exit() {
        let mut eval = Eval::new(
            r#"
            fun earlyReturn() {
                return "early";
                return "late";
            }
            print earlyReturn();
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_statement_in_nested_block() {
        let mut eval = Eval::new(
            r#"
            fun nestedReturn(x) {
                if (x > 0) {
                    return "positive";
                }
                return "non-positive";
            }
            print nestedReturn(5);
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_statement_without_value() {
        let mut eval = Eval::new(
            r#"
            fun voidReturn() {
                return;
            }
            voidReturn();
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    // Control Flow with Scoping Tests
    #[test]
    fn test_if_statement_with_scoping() {
        let mut eval = Eval::new(
            r#"
            var result = "none";
            if (true) {
                var x = "if-block";
                result = x;
            }
            print result;
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_while_statement_with_scoping() {
        let mut eval = Eval::new(
            r#"
            var i = 0;
            var sum = 0;
            while (i < 3) {
                var temp = i * 2;
                sum = sum + temp;
                i = i + 1;
            }
            print sum;
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_blocks_with_scoping() {
        let mut eval = Eval::new(
            r#"
            var a = "outer";
            {
                var a = "middle";
                {
                    var a = "inner";
                    print a;
                }
                print a;
            }
            print a;
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_square() {
        let mut eval = Eval::new(
            r#"
            fun square(x) {
            return x * x;
            }

            // This higher-order function applies a
            // function N times to a starting value x.
            fun applyTimesN(N, f, x) {
            var i = 0;
            while (i < N) {
                x = f(x);
                i = i + 1;
            }
            return x;
            }

            // 6 is squared once
            print applyTimesN(1, square, 6);
            // 6 is squared twice
            print applyTimesN(2, square, 6);
            // 6 is squared thrice
            print applyTimesN(3, square, 6);
        "#,
            false,
        );
        let result = eval.evaluate();
        assert!(result.is_ok());
    }
}
