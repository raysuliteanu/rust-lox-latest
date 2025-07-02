use std::fmt::Display;
use thiserror::Error;

use crate::span::Span;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("[line {}] Error: Unexpected character: {}", .span.line(),
    .src[.span.offset()..(.span.offset() + .span.len())].to_string())]
    InvalidToken { src: String, span: Span },

    #[error("[line {}] Error: Unterminated string: {}", .span.line(),
    .src[.span.offset()..(.span.offset() + .span.len())].to_string())]
    UnterminatedString { src: String, span: Span },
}

#[derive(Clone, Debug)]
pub struct Token {
    pub(crate) lexeme: Lexeme,
    pub(crate) span: Span,
}

impl Token {
    pub fn new(lexeme: Lexeme, span: Span) -> Token {
        Token { lexeme, span }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lexeme = &self.lexeme;
        match lexeme {
            Lexeme::String(value) => {
                write!(f, "{lexeme} \"{value}\" {value}")
            }
            Lexeme::Number(raw, value) => {
                if *value == value.trunc() {
                    // tests require that integers are printed as N.0
                    write!(f, "NUMBER {raw} {value}.0")
                } else {
                    write!(f, "NUMBER {raw} {value}")
                }
            }
            Lexeme::Identifier(value) => {
                write!(f, "{lexeme} {value} null")
            }
            Lexeme::LeftParen(v)
            | Lexeme::RightParen(v)
            | Lexeme::LeftBrace(v)
            | Lexeme::RightBrace(v)
            | Lexeme::Dot(v)
            | Lexeme::Comma(v)
            | Lexeme::Minus(v)
            | Lexeme::Plus(v)
            | Lexeme::SemiColon(v)
            | Lexeme::Star(v)
            | Lexeme::Eq(v)
            | Lexeme::Bang(v)
            | Lexeme::Less(v)
            | Lexeme::Greater(v)
            | Lexeme::Slash(v) => write!(f, "{lexeme} {v} null"),
            Lexeme::EqEq(v)
            | Lexeme::BangEq(v)
            | Lexeme::LessEq(v)
            | Lexeme::GreaterEq(v)
            | Lexeme::True(v)
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
            | Lexeme::Print(v) => {
                let v = v.to_lowercase();
                write!(f, "{lexeme} {v} null")
            }
            Lexeme::Eof(_) => {
                write!(f, "{lexeme}  null")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Lexeme {
    // keywords
    True(String),
    False(String),
    Nil(String),
    And(String),
    Or(String),
    Class(String),
    For(String),
    Fun(String),
    If(String),
    Else(String),
    Return(String),
    Super(String),
    This(String),
    Var(String),
    While(String),
    Print(String),

    // literals
    LeftParen(char),
    RightParen(char),
    LeftBrace(char),
    RightBrace(char),
    Comma(char),
    Dot(char),
    Minus(char),
    Plus(char),
    SemiColon(char),
    Star(char),
    Eq(char),
    EqEq(String),
    Bang(char),
    BangEq(String),
    Less(char),
    LessEq(String),
    Greater(char),
    GreaterEq(String),
    Slash(char),

    // value holders
    Number(String, f64),
    Identifier(String),
    String(String),

    // symbolic placeholder
    Eof(String),
}

impl From<&str> for Lexeme {
    fn from(value: &str) -> Self {
        match value {
            "(" => Lexeme::LeftParen('('),
            ")" => Lexeme::RightParen(')'),
            "{" => Lexeme::LeftBrace('{'),
            "}" => Lexeme::RightBrace('}'),
            "," => Lexeme::Comma(','),
            "." => Lexeme::Dot('.'),
            ";" => Lexeme::SemiColon(';'),
            "+" => Lexeme::Plus('+'),
            "-" => Lexeme::Minus('-'),
            "*" => Lexeme::Star('*'),
            "/" => Lexeme::Slash('/'),
            "=" => Lexeme::Eq('='),
            "!" => Lexeme::Bang('!'),
            "<" => Lexeme::Less('<'),
            ">" => Lexeme::Greater('>'),
            "==" => Lexeme::EqEq("==".to_string()),
            "!=" => Lexeme::BangEq("!=".to_string()),
            "<=" => Lexeme::LessEq("<=".to_string()),
            ">=" => Lexeme::GreaterEq(">=".to_string()),
            "print" => Lexeme::Print("PRINT".to_string()),
            "true" => Lexeme::True("TRUE".to_string()),
            "false" => Lexeme::False("FALSE".to_string()),
            "nil" => Lexeme::Nil("NIL".to_string()),
            "and" => Lexeme::And("AND".to_string()),
            "or" => Lexeme::Or("OR".to_string()),
            "fun" => Lexeme::Fun("FUN".to_string()),
            "return" => Lexeme::Return("RETURN".to_string()),
            "if" => Lexeme::If("IF".to_string()),
            "else" => Lexeme::Else("ELSE".to_string()),
            "for" => Lexeme::For("FOR".to_string()),
            "while" => Lexeme::While("WHILE".to_string()),
            "class" => Lexeme::Class("CLASS".to_string()),
            "super" => Lexeme::Super("SUPER".to_string()),
            "this" => Lexeme::This("THIS".to_string()),
            "var" => Lexeme::Var("VAR".to_string()),
            "eof" => Lexeme::Eof("EOF".to_string()),
            _ => panic!("invalid token {value}"),
        }
    }
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lexeme::Number(_, _) => write!(f, "NUMBER"),
            Lexeme::Identifier(_) => write!(f, "IDENTIFIER"),
            Lexeme::String(_) => write!(f, "STRING"),
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
            | Lexeme::Eof(v) => write!(f, "{v}"),
        }
    }
}

pub struct Scanner<'scanner> {
    source: &'scanner str,
}

impl<'scanner> Scanner<'scanner> {
    pub fn new(source: &'scanner str) -> Self {
        Scanner { source }
    }

    pub fn scan(self) -> anyhow::Result<Vec<Token>> {
        let mut line: usize = 1;
        let mut tokens: Vec<Token> = Vec::new();
        let mut peekable_iter = self.source.char_indices().peekable();

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
                                src: self.source.to_string(),
                                span: Span::new(line, i, str.len()),
                            }
                            .into());
                        } else {
                            // consume terminating "
                            let (l, _) = peekable_iter.next().unwrap();
                            tokens
                                .push(Token::new(Lexeme::String(str), (line, i + 1, l - 1).into()))
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

                        let num = match num_literal.parse::<f64>() {
                            Ok(value) => value,
                            Err(_e) => {
                                return Err(TokenError::InvalidToken {
                                    src: number.clone(),
                                    span: Span::new(line, i, num_literal.len()),
                                }
                                .into());
                            }
                        };

                        tokens.push(Token::new(
                            Lexeme::Number(num_literal.to_string(), num),
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

                        if let Some(keyword) = keyword_token(&s) {
                            tokens.push(Token::new(keyword, (line, i, s.len()).into()))
                        } else {
                            let len = s.len();
                            tokens.push(Token::new(
                                Lexeme::Identifier(s.clone()),
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
                            src: self.source.to_string(),
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

        tokens.push(Token::new(
            "eof".into(),
            Span::new(line, self.source.len(), 0),
        ));

        Ok(tokens)
    }
}

fn keyword_token(s: &str) -> Option<Lexeme> {
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
