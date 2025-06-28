use std::fmt::Display;
use thiserror::Error;

use crate::span::Span;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("[line {}] Error: Unexpected character: {}", .span.line(), .ch) ]
    InvalidToken { ch: &'static str, span: Span },

    #[error("[line {}] Error: Unterminated string: {}", .span.line(),
    .src[.span.offset()..(.span.offset() + .span.len())].to_string())]
    UnterminatedString { src: &'static str, span: Span },
}

#[derive(Clone, Debug)]
pub struct Token {
    pub(crate) lexeme: Lexeme,
    pub(crate) span: Span,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lexeme = &self.lexeme;
        match lexeme {
            Lexeme::String(value) => {
                write!(f, "{lexeme} \"{value}\" {value}")
            }
            Lexeme::Number(value) => {
                let val_as_str = &format!("{value}");
                let trimmed = strip_suffix(val_as_str, ".0");
                write!(f, "{lexeme} {trimmed} {val_as_str}")
            }
            Lexeme::Identifier(value) => {
                write!(f, "{lexeme} {value} null")
            }
            _ => write!(f, "{lexeme} {lexeme} null"),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct CharValue(char);
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct StringValue(String);
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct NumberValue(f64);

trait ValueType: Clone {
    type T;

    fn value(&self) -> Self::T;
}

impl ValueType for CharValue {
    type T = char;

    fn value(&self) -> Self::T {
        self.0
    }
}

impl Display for CharValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ValueType for StringValue {
    type T = String;

    fn value(&self) -> Self::T {
        self.0.clone()
    }
}

impl Display for StringValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ValueType for NumberValue {
    type T = f64;

    fn value(&self) -> Self::T {
        self.0
    }
}

impl Display for NumberValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Lexeme {
    // keywords
    True(StringValue),
    False(StringValue),
    Nil(StringValue),
    And(StringValue),
    Or(StringValue),
    Class(StringValue),
    For(StringValue),
    Fun(StringValue),
    If(StringValue),
    Else(StringValue),
    Return(StringValue),
    Super(StringValue),
    This(StringValue),
    Var(StringValue),
    While(StringValue),
    Print(StringValue),

    // literals
    LeftParen(CharValue),
    RightParen(CharValue),
    LeftBrace(CharValue),
    RightBrace(CharValue),
    Comma(CharValue),
    Dot(CharValue),
    Minus(CharValue),
    Plus(CharValue),
    SemiColon(CharValue),
    Star(CharValue),
    Eq(CharValue),
    EqEq(StringValue),
    Bang(CharValue),
    BangEq(StringValue),
    Less(CharValue),
    LessEq(StringValue),
    Greater(CharValue),
    GreaterEq(StringValue),
    Slash(CharValue),

    // value holders
    Number(NumberValue),
    Identifier(StringValue),
    String(StringValue),

    // symbolic placeholder
    Eof,
}

impl From<&'static str> for Lexeme {
    fn from(value: &'static str) -> Self {
        match value {
            "(" => Lexeme::RightParen(CharValue('(')),
            ")" => Lexeme::LeftParen(CharValue(')')),
            "{" => Lexeme::RightBrace(CharValue('{')),
            "}" => Lexeme::LeftBrace(CharValue('}')),
            "," => Lexeme::Comma(CharValue(',')),
            "." => Lexeme::Dot(CharValue('.')),
            ";" => Lexeme::SemiColon(CharValue(';')),
            "+" => Lexeme::Plus(CharValue('+')),
            "-" => Lexeme::Minus(CharValue('-')),
            "*" => Lexeme::Star(CharValue('*')),
            "/" => Lexeme::Slash(CharValue('/')),
            "=" => Lexeme::Eq(CharValue('=')),
            "!" => Lexeme::Bang(CharValue('!')),
            "<" => Lexeme::Less(CharValue('<')),
            ">" => Lexeme::Greater(CharValue('>')),
            "==" => Lexeme::EqEq(StringValue("==".to_string())),
            "!=" => Lexeme::BangEq(StringValue("!=".to_string())),
            "<=" => Lexeme::LessEq(StringValue("<=".to_string())),
            ">=" => Lexeme::GreaterEq(StringValue(">=".to_string())),
            "print" => Lexeme::Print(StringValue("PRINT".to_string())),
            "true" => Lexeme::True(StringValue("TRUE".to_string())),
            "false" => Lexeme::False(StringValue("FALSE".to_string())),
            "nil" => Lexeme::Nil(StringValue("NIL".to_string())),
            "and" => Lexeme::And(StringValue("AND".to_string())),
            "or" => Lexeme::Or(StringValue("OR".to_string())),
            "fun" => Lexeme::Fun(StringValue("FUN".to_string())),
            "return" => Lexeme::Return(StringValue("RETURN".to_string())),
            "if" => Lexeme::If(StringValue("IF".to_string())),
            "else" => Lexeme::Else(StringValue("ELSE".to_string())),
            "for" => Lexeme::For(StringValue("FOR".to_string())),
            "while" => Lexeme::While(StringValue("WHILE".to_string())),
            "class" => Lexeme::Class(StringValue("CLASS".to_string())),
            "super" => Lexeme::Super(StringValue("SUPER".to_string())),
            "this" => Lexeme::This(StringValue("THIS".to_string())),
            _ => panic!("invalid token {value}"),
        }
    }
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lexeme::Number(_) => write!(f, "NUMBER"),
            Lexeme::Identifier(_) => write!(f, "IDENTIFIER"),
            Lexeme::String(_) => write!(f, "STRING"),
            Lexeme::True(_) => write!(f, "TRUE"),
            Lexeme::False(_) => write!(f, "FALSE"),
            Lexeme::Nil(_) => write!(f, "NIL"),
            Lexeme::And(_) => write!(f, "AND"),
            Lexeme::Or(_) => write!(f, "OR"),
            Lexeme::Class(_) => write!(f, "CLASS"),
            Lexeme::For(_) => write!(f, "FOR"),
            Lexeme::Fun(_) => write!(f, "FUN"),
            Lexeme::If(_) => write!(f, "IF"),
            Lexeme::Else(_) => write!(f, "ELSE"),
            Lexeme::Return(_) => write!(f, "RETURN"),
            Lexeme::Super(_) => write!(f, "SUPER"),
            Lexeme::This(_) => write!(f, "THIS"),
            Lexeme::Var(_) => write!(f, "VAR"),
            Lexeme::While(_) => write!(f, "WHILE"),
            Lexeme::Print(_) => write!(f, "PRINT"),
            Lexeme::LeftParen(_) => write!(f, "LEFT_PAREN"),
            Lexeme::RightParen(_) => write!(f, "RIGHT_PAREN"),
            Lexeme::LeftBrace(_) => write!(f, "LEFT_BRACE"),
            Lexeme::RightBrace(_) => write!(f, "RIGHT_BRACE"),
            Lexeme::Dot(_) => write!(f, "DOT"),
            Lexeme::Comma(_) => write!(f, "COMMA"),
            Lexeme::Minus(_) => write!(f, "MINUS"),
            Lexeme::Plus(_) => write!(f, "PLUS"),
            Lexeme::SemiColon(_) => write!(f, "SEMICOLON"),
            Lexeme::Star(_) => write!(f, "STAR"),
            Lexeme::Eq(_) => write!(f, "EQUAL"),
            Lexeme::EqEq(_) => write!(f, "EQUAL_EQUAL"),
            Lexeme::Bang(_) => write!(f, "BANG"),
            Lexeme::BangEq(_) => write!(f, "BANG_EQUAL"),
            Lexeme::Less(_) => write!(f, "LESS"),
            Lexeme::LessEq(_) => write!(f, "LESS_EQUAL"),
            Lexeme::Greater(_) => write!(f, "GREATER"),
            Lexeme::GreaterEq(_) => write!(f, "GREATER_EQUAL"),
            Lexeme::Slash(_) => write!(f, "SLASH"),
            Lexeme::Eof => write!(f, "EOF"),
        }
    }
}

impl Token {
    pub fn new(lexeme: Lexeme, span: Span) -> Token {
        Token { lexeme, span }
    }
}

fn strip_suffix(s: &String, p: &str) -> String {
    s.strip_suffix(p).unwrap_or(s.as_str()).to_string()
}

pub struct Scanner;

impl Scanner {
    pub fn scan(source: &'static str) -> anyhow::Result<Vec<Token>> {
        let mut line: usize = 1;
        let mut tokens: Vec<Token> = Vec::new();
        let mut peekable_iter = source.char_indices().peekable();

        #[allow(clippy::while_let_loop)]
        loop {
            match peekable_iter.next() {
                Some((i, c)) => match c {
                    c if one_of(c, "\n\r") => {
                        line += 1;
                        continue;
                    }
                    c if c.is_whitespace() => continue,
                    '(' => tokens.push(Token::new("(".into(), (line, i, 1).into())),
                    ')' => tokens.push(Token::new(")".into(), (line, i, 1).into())),
                    '{' => tokens.push(Token::new("{".into(), (line, i, 1).into())),
                    '}' => tokens.push(Token::new("}".into(), (line, i, 1).into())),
                    ',' => tokens.push(Token::new(",".into(), (line, i, 1).into())),
                    '.' => tokens.push(Token::new(".".into(), (line, i, 1).into())),
                    '+' => tokens.push(Token::new("+".into(), (line, i, 1).into())),
                    '-' => tokens.push(Token::new("-".into(), (line, i, 1).into())),
                    ';' => tokens.push(Token::new(";".into(), (line, i, 1).into())),
                    '*' => tokens.push(Token::new("*".into(), (line, i, 1).into())),
                    '=' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new("==".into(), (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new("=".into(), (line, i, 1).into()))
                        }
                    }
                    '<' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new("<=".into(), (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new("<".into(), (line, i, 1).into()))
                        }
                    }
                    '>' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new(">=".into(), (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new(">".into(), (line, i, 1).into()))
                        }
                    }
                    '!' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new("!=".into(), (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new("!".into(), (line, i, 1).into()))
                        }
                    }
                    '\"' => {
                        let mut str = String::new();

                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| !matches!(*l, '"'))
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            str.push(ch);
                        }

                        if peekable_iter.peek().is_none() {
                            return Err(TokenError::UnterminatedString {
                                src: source.as_str(),
                                span: Span::new(line, i, str.len()),
                            }
                            .into());
                        } else {
                            // consume terminating "
                            let (l, _) = peekable_iter.next().unwrap();
                            tokens.push(Token::new(
                                Lexeme::String(StringValue(str)),
                                (line, i + 1, l - 1).into(),
                            ))
                        }
                    }
                    c if c.is_ascii_digit() => {
                        let mut number = String::from(c);
                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| l.is_ascii_digit())
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            number.push(ch);
                        }

                        if peekable_iter.peek().is_some_and(|(_, l)| matches!(*l, '.')) {
                            let (_, ch) = peekable_iter.next().unwrap();
                            number.push(ch);
                        }

                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| l.is_ascii_digit())
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            number.push(ch);
                        }

                        let mut num_literal = number.as_str();

                        // There are 3 possibilities now. The number can be one of
                        // 1. just digits e.g. 123
                        // 2. digits plus a trailing . e.g. 123.
                        // 3. digits before and after a . e.g. 123.45
                        // So doing the splitn(3, '.') will result in
                        // 1. Some, None, None => the _ case
                        // 2. Some, Some, None => the 123. case
                        // 3. Some, Some, Some => the 123.45 case
                        let mut split = num_literal.splitn(3, '.');
                        match (split.next(), split.next(), split.next()) {
                            (Some(first), Some(second), Some(_)) => {
                                num_literal = &num_literal[..first.len() + 1 + second.len()]; // +1 for the dot sep
                            }
                            (Some(first), Some(second), None) => {
                                if second.is_empty() {
                                    num_literal = &num_literal[..first.len()];
                                }
                            }
                            _ => {}
                        }

                        let value: f64 = match num_literal.parse() {
                            Ok(value) => value,
                            Err(_e) => {
                                return Err(TokenError::InvalidToken {
                                    ch: number.clone().as_str(),
                                    span: Span::new(line, i, num_literal.len()),
                                }
                                .into());
                            }
                        };

                        tokens.push(Token::new(
                            Lexeme::Number(NumberValue(value)),
                            (line, i, num_literal.len()).into(),
                        ))
                    }
                    c if c.is_alphabetic() | (c == '_') => {
                        let mut s = String::from(c);
                        while peekable_iter
                            .peek()
                            .is_some_and(|(_, l)| matches!(*l, '_' | 'a'..='z'|'A'..='Z'))
                        {
                            let (_, ch) = peekable_iter.next().unwrap();
                            s.push(ch);
                        }

                        if let Some(keyword) = keyword_token(s.to_lowercase().as_str()) {
                            tokens.push(Token::new(keyword, (line, i, s.len()).into()))
                        } else {
                            let len = s.len();
                            tokens.push(Token::new(
                                Lexeme::Identifier(StringValue(s)),
                                (line, i, len).into(),
                            ));
                        }
                    }
                    '/' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '/') {
                            // line comment so each chars till EOL chars
                            while peekable_iter
                                .peek()
                                .is_some_and(|(_i, c)| !one_of(*c, "\n\r"))
                            {
                                peekable_iter.next();
                            }
                        } else {
                            tokens.push(Token::new("/".into(), (line, i, 1).into()))
                        }
                    }
                    _ => {
                        return Err(TokenError::InvalidToken {
                            ch: source,
                            span: Span::new(line, i, 1),
                        }
                        .into());
                    }
                },
                None => {
                    break;
                }
            }
        }

        Ok(tokens)
    }
}

fn keyword_token(s: &'static str) -> Option<Lexeme> {
    match s {
        "true" | "false" | "nil" | "and" | "or" | "class" | "for" | "fun" | "if" | "else"
        | "return" | "super" | "this" | "var" | "while" | "print" => Some(s.into()),
        _ => None,
    }
}

fn one_of(c: char, chars: &str) -> bool {
    chars.chars().any(|possible| possible == c)
}

#[cfg(test)]
mod tests {}
