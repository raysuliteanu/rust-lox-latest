use thiserror::Error;

use crate::{
    model::{Lexeme, Token},
    span::Span,
    util::StringInterner,
};

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("[line {}] Error: Unexpected character: {}", .span.line(),
    &.src[.span.offset()..(.span.offset() + .span.len())])]
    InvalidToken { src: String, span: Span },

    // CC/Book wants error message exactly like this even though easy
    // enough to have the part of the unterminated string in the error
    #[error("[line {}] Error: Unterminated string.", .span.line())]
    UnterminatedString { src: String, span: Span },
}

pub type TokenResult<T> = Result<T, u8>;

pub struct Scanner<'scanner> {
    source: &'scanner str,
    interner: StringInterner,
    print_tokens: bool,
}

impl<'scanner> Scanner<'scanner> {
    pub fn new(source: &'scanner str, print_tokens: bool) -> Self {
        Scanner {
            source,
            interner: StringInterner::new(),
            print_tokens,
        }
    }

    pub fn scan(&mut self) -> TokenResult<Vec<Token>> {
        let mut line: usize = 1;
        let mut tokens: Vec<Token> = Vec::new();
        let mut peekable_iter = self.source.char_indices().peekable();
        let mut has_error = false;

        #[allow(clippy::while_let_loop)]
        loop {
            match peekable_iter.next() {
                Some((i, c)) => match c {
                    c if one_of(c, "\n\r") => {
                        line += 1;
                        continue;
                    }
                    c if c.is_whitespace() => continue,
                    '(' => tokens.push(Token::new(Lexeme::LeftParen, (line, i, 1).into())),
                    ')' => tokens.push(Token::new(Lexeme::RightParen, (line, i, 1).into())),
                    '{' => tokens.push(Token::new(Lexeme::LeftBrace, (line, i, 1).into())),
                    '}' => tokens.push(Token::new(Lexeme::RightBrace, (line, i, 1).into())),
                    ',' => tokens.push(Token::new(Lexeme::Comma, (line, i, 1).into())),
                    '.' => tokens.push(Token::new(Lexeme::Dot, (line, i, 1).into())),
                    '+' => tokens.push(Token::new(Lexeme::Plus, (line, i, 1).into())),
                    '-' => tokens.push(Token::new(Lexeme::Minus, (line, i, 1).into())),
                    ';' => tokens.push(Token::new(Lexeme::SemiColon, (line, i, 1).into())),
                    '*' => tokens.push(Token::new(Lexeme::Star, (line, i, 1).into())),
                    '=' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new(Lexeme::EqEq, (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new(Lexeme::Eq, (line, i, 1).into()))
                        }
                    }
                    '<' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new(Lexeme::LessEq, (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new(Lexeme::Less, (line, i, 1).into()))
                        }
                    }
                    '>' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new(Lexeme::GreaterEq, (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new(Lexeme::Greater, (line, i, 1).into()))
                        }
                    }
                    '!' => {
                        if peekable_iter.peek().is_some_and(|(_, l)| *l == '=') {
                            peekable_iter.next();
                            tokens.push(Token::new(Lexeme::BangEq, (line, i, 2).into()))
                        } else {
                            tokens.push(Token::new(Lexeme::Bang, (line, i, 1).into()))
                        }
                    }
                    '\"' => {
                        let start_idx = i + 1; // Skip opening quote
                        let mut end_idx = start_idx;

                        while peekable_iter
                            .peek()
                            .is_some_and(|(_idx, ch)| !matches!(*ch, '"'))
                        {
                            let (idx, _) = peekable_iter.next().unwrap();
                            end_idx = idx + 1;
                        }

                        if peekable_iter.peek().is_none() {
                            let error = TokenError::UnterminatedString {
                                src: self.source.into(),
                                span: Span::new(line, i, end_idx - i),
                            };
                            eprintln!("{error}");
                            has_error = true;
                        } else {
                            // consume terminating "
                            let (_closing_quote_idx, _) = peekable_iter.next().unwrap();
                            let string_content = &self.source[start_idx..end_idx];
                            tokens.push(Token::new(
                                Lexeme::String(self.interner.intern(string_content)),
                                (line, start_idx, end_idx - start_idx).into()
                            ));
                        }
                    }
                    c if c.is_ascii_digit() => {
                        let start_idx = i;
                        let mut end_idx = i + 1;

                        // Parse integer part
                        while peekable_iter
                            .peek()
                            .is_some_and(|(_idx, ch)| ch.is_ascii_digit())
                        {
                            let (idx, _) = peekable_iter.next().unwrap();
                            end_idx = idx + 1;
                        }

                        // Check for decimal point
                        let mut has_decimal = false;
                        if peekable_iter.peek().is_some_and(|(_, ch)| *ch == '.') {
                            let (idx, _) = peekable_iter.next().unwrap();
                            end_idx = idx + 1;
                            has_decimal = true;
                        }

                        // Parse fractional part if decimal point exists
                        if has_decimal {
                            while peekable_iter
                                .peek()
                                .is_some_and(|(_idx, ch)| ch.is_ascii_digit())
                            {
                                let (idx, _) = peekable_iter.next().unwrap();
                                end_idx = idx + 1;
                            }
                        }

                        let mut num_slice = &self.source[start_idx..end_idx];

                        // Handle trailing dot case (123. should become 123)
                        if has_decimal && num_slice.ends_with('.') {
                            // Check if there are digits after the dot
                            let dot_pos = num_slice.rfind('.').unwrap();
                            if dot_pos == num_slice.len() - 1 {
                                // Trailing dot with no fractional digits, remove it
                                num_slice = &num_slice[..dot_pos];
                                end_idx = start_idx + num_slice.len();
                            }
                        }

                        if let Ok(num) = num_slice.parse::<f64>() {
                            tokens.push(Token::new(
                                Lexeme::Number(self.interner.intern(num_slice), num),
                                (line, start_idx, end_idx - start_idx).into(),
                            ));
                        } else {
                            let error = TokenError::InvalidToken {
                                src: self.source.into(),
                                span: Span::new(line, start_idx, end_idx - start_idx),
                            };
                            eprintln!("{error}");
                            has_error = true;
                        }
                    }
                    c if c.is_alphabetic() | (c == '_') => {
                        let start_idx = i;
                        let mut end_idx = i + 1;

                        #[allow(clippy::almost_complete_range)]
                        while peekable_iter.peek().is_some_and(
                            |(_idx, ch)| matches!(*ch, '_' | 'a'..='z'|'A'..='Z' | '0'..'9'),
                        ) {
                            let (idx, _) = peekable_iter.next().unwrap();
                            end_idx = idx + 1;
                        }

                        let identifier_slice = &self.source[start_idx..end_idx];
                        let len = end_idx - start_idx;

                        if let Some(keyword) = keyword_token(identifier_slice) {
                            tokens.push(Token::new(keyword, (line, start_idx, len).into()))
                        } else {
                            tokens.push(Token::new(
                                Lexeme::Identifier(self.interner.intern(identifier_slice)),
                                (line, start_idx, len).into(),
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
                            tokens.push(Token::new(Lexeme::Slash, (line, i, 1).into()))
                        }
                    }
                    _ => {
                        let error = TokenError::InvalidToken {
                            src: self.source.into(),
                            span: Span::new(line, i, 1),
                        };
                        eprintln!("{error}");
                        has_error = true;
                    }
                },
                None => {
                    break;
                }
            }
        }

        tokens.push(Token::new(
            Lexeme::Eof,
            Span::new(line, self.source.len(), 0),
        ));

        if self.print_tokens {
            tokens.iter().for_each(|t| println!("{t}"));
        }

        if has_error { Err(65) } else { Ok(tokens) }
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
mod tests {
    use super::*;

    #[test]
    fn test_token_new() {
        let span = Span::new(1, 0, 1);
        let lexeme = Lexeme::LeftParen;
        let token = Token::new(lexeme.clone(), span.clone());

        assert_eq!(token.lexeme, lexeme);
        assert_eq!(token.span, span);
    }

    #[test]
    fn test_token_display_string() {
        let token = Token::new(Lexeme::String("hello".into()), Span::new(1, 0, 7));
        assert_eq!(format!("{token}"), "STRING \"hello\" hello");
    }

    #[test]
    fn test_token_display_number_integer() {
        let token = Token::new(Lexeme::Number("42".into(), 42.0), Span::new(1, 0, 2));
        assert_eq!(format!("{token}"), "NUMBER 42 42.0");
    }

    #[test]
    fn test_token_display_number_float() {
        let token = Token::new(Lexeme::Number("1.23".into(), 1.23), Span::new(1, 0, 4));
        assert_eq!(format!("{token}"), "NUMBER 1.23 1.23");
    }

    #[test]
    fn test_token_display_identifier() {
        let token = Token::new(
            Lexeme::Identifier("variable".into()),
            Span::new(1, 0, 8),
        );
        assert_eq!(format!("{token}"), "IDENTIFIER variable null");
    }

    #[test]
    fn test_token_display_single_char() {
        let token = Token::new(Lexeme::LeftParen, Span::new(1, 0, 1));
        assert_eq!(format!("{token}"), "LEFT_PAREN ( null");
    }

    #[test]
    fn test_token_display_keyword() {
        let token = Token::new(Lexeme::True, Span::new(1, 0, 4));
        assert_eq!(format!("{token}"), "TRUE true null");
    }

    #[test]
    fn test_token_display_eof() {
        let token = Token::new(Lexeme::Eof, Span::new(1, 0, 0));
        assert_eq!(format!("{token}"), "EOF  null");
    }

    #[test]
    fn test_lexeme_from_single_chars() {
        assert_eq!(Lexeme::from("("), Lexeme::LeftParen);
        assert_eq!(Lexeme::from(")"), Lexeme::RightParen);
        assert_eq!(Lexeme::from("{"), Lexeme::LeftBrace);
        assert_eq!(Lexeme::from("}"), Lexeme::RightBrace);
        assert_eq!(Lexeme::from(","), Lexeme::Comma);
        assert_eq!(Lexeme::from("."), Lexeme::Dot);
        assert_eq!(Lexeme::from(";"), Lexeme::SemiColon);
        assert_eq!(Lexeme::from("+"), Lexeme::Plus);
        assert_eq!(Lexeme::from("-"), Lexeme::Minus);
        assert_eq!(Lexeme::from("*"), Lexeme::Star);
        assert_eq!(Lexeme::from("/"), Lexeme::Slash);
        assert_eq!(Lexeme::from("="), Lexeme::Eq);
        assert_eq!(Lexeme::from("!"), Lexeme::Bang);
        assert_eq!(Lexeme::from("<"), Lexeme::Less);
        assert_eq!(Lexeme::from(">"), Lexeme::Greater);
    }

    #[test]
    fn test_lexeme_from_double_chars() {
        assert_eq!(Lexeme::from("=="), Lexeme::EqEq);
        assert_eq!(Lexeme::from("!="), Lexeme::BangEq);
        assert_eq!(Lexeme::from("<="), Lexeme::LessEq);
        assert_eq!(Lexeme::from(">="), Lexeme::GreaterEq);
    }

    #[test]
    fn test_lexeme_from_keywords() {
        assert_eq!(Lexeme::from("print"), Lexeme::Print);
        assert_eq!(Lexeme::from("true"), Lexeme::True);
        assert_eq!(Lexeme::from("false"), Lexeme::False);
        assert_eq!(Lexeme::from("nil"), Lexeme::Nil);
        assert_eq!(Lexeme::from("and"), Lexeme::And);
        assert_eq!(Lexeme::from("or"), Lexeme::Or);
        assert_eq!(Lexeme::from("fun"), Lexeme::Fun);
        assert_eq!(Lexeme::from("return"), Lexeme::Return);
        assert_eq!(Lexeme::from("if"), Lexeme::If);
        assert_eq!(Lexeme::from("else"), Lexeme::Else);
        assert_eq!(Lexeme::from("for"), Lexeme::For);
        assert_eq!(Lexeme::from("while"), Lexeme::While);
        assert_eq!(Lexeme::from("class"), Lexeme::Class);
        assert_eq!(Lexeme::from("super"), Lexeme::Super);
        assert_eq!(Lexeme::from("this"), Lexeme::This);
        assert_eq!(Lexeme::from("var"), Lexeme::Var);
        assert_eq!(Lexeme::from("eof"), Lexeme::Eof);
    }

    #[test]
    #[should_panic(expected = "invalid token invalid")]
    fn test_lexeme_from_invalid() {
        let _ = Lexeme::from("invalid");
    }

    #[test]
    fn test_lexeme_display() {
        assert_eq!(
            format!("{}", Lexeme::Number("42".into(), 42.0)),
            "NUMBER"
        );
        assert_eq!(
            format!("{}", Lexeme::Identifier("var".into())),
            "IDENTIFIER"
        );
        assert_eq!(format!("{}", Lexeme::String("hello".into())), "STRING");
        assert_eq!(format!("{}", Lexeme::LeftParen), "LEFT_PAREN");
        assert_eq!(format!("{}", Lexeme::True), "TRUE");
        assert_eq!(format!("{}", Lexeme::Eof), "EOF");
    }

    #[test]
    fn test_scanner_new() {
        let source = "test source";
        let scanner = Scanner::new(source, true);
        assert_eq!(scanner.source, source);
    }

    #[test]
    fn test_scanner_empty_source() {
        let mut scanner = Scanner::new("", true);
        let tokens = scanner.scan().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].lexeme, Lexeme::Eof);
    }

    #[test]
    fn test_scanner_single_tokens() {
        let mut scanner = Scanner::new("(){},.+-;*", true);
        let tokens = scanner.scan().unwrap();

        let expected_lexemes = vec![
            Lexeme::LeftParen,
            Lexeme::RightParen,
            Lexeme::LeftBrace,
            Lexeme::RightBrace,
            Lexeme::Comma,
            Lexeme::Dot,
            Lexeme::Plus,
            Lexeme::Minus,
            Lexeme::SemiColon,
            Lexeme::Star,
            Lexeme::Eof,
        ];

        assert_eq!(tokens.len(), expected_lexemes.len());
        for (token, expected) in tokens.iter().zip(expected_lexemes.iter()) {
            assert_eq!(&token.lexeme, expected);
        }
    }

    #[test]
    fn test_scanner_comparison_operators() {
        let mut scanner = Scanner::new("== ! != < <= > >=", true);
        let tokens = scanner.scan().unwrap();

        let expected_lexemes = vec![
            Lexeme::EqEq,
            Lexeme::Bang,
            Lexeme::BangEq,
            Lexeme::Less,
            Lexeme::LessEq,
            Lexeme::Greater,
            Lexeme::GreaterEq,
            Lexeme::Eof,
        ];

        assert_eq!(tokens.len(), expected_lexemes.len());
        for (token, expected) in tokens.iter().zip(expected_lexemes.iter()) {
            assert_eq!(&token.lexeme, expected);
        }
    }

    #[test]
    fn test_scanner_string_literal() {
        let mut scanner = Scanner::new("\"hello world\"", true);
        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].lexeme, Lexeme::String("hello world".into()));
        assert_eq!(tokens[1].lexeme, Lexeme::Eof);
    }

    #[test]
    fn test_scanner_unterminated_string() {
        let mut scanner = Scanner::new("\"unterminated", true);
        let result = scanner.scan();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error, 65);
    }

    #[test]
    fn test_scanner_numbers() {
        let mut scanner = Scanner::new("123 1.23 42.0", true);
        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].lexeme, Lexeme::Number("123".into(), 123.0));
        assert_eq!(tokens[1].lexeme, Lexeme::Number("1.23".into(), 1.23));
        assert_eq!(tokens[2].lexeme, Lexeme::Number("42.0".into(), 42.0));
        assert_eq!(tokens[3].lexeme, Lexeme::Eof);
    }

    #[test]
    fn test_scanner_identifiers() {
        let mut scanner = Scanner::new("variable _123private camelCase", true);
        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].lexeme, Lexeme::Identifier("variable".into()));
        assert_eq!(
            tokens[1].lexeme,
            Lexeme::Identifier("_123private".into())
        );
        assert_eq!(
            tokens[2].lexeme,
            Lexeme::Identifier("camelCase".into())
        );
        assert_eq!(tokens[3].lexeme, Lexeme::Eof);
    }

    #[test]
    fn test_scanner_keywords() {
        let mut scanner = Scanner::new("true false nil and or", true);
        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].lexeme, Lexeme::True);
        assert_eq!(tokens[1].lexeme, Lexeme::False);
        assert_eq!(tokens[2].lexeme, Lexeme::Nil);
        assert_eq!(tokens[3].lexeme, Lexeme::And);
        assert_eq!(tokens[4].lexeme, Lexeme::Or);
        assert_eq!(tokens[5].lexeme, Lexeme::Eof);
    }

    #[test]
    fn test_scanner_comments() {
        let mut scanner = Scanner::new("// this is a comment\n42", true);
        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].lexeme, Lexeme::Number("42".into(), 42.0));
        assert_eq!(tokens[1].lexeme, Lexeme::Eof);
    }

    #[test]
    fn test_scanner_whitespace() {
        let mut scanner = Scanner::new("  \t\n  42  \r\n  ", true);
        let tokens = scanner.scan().unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].lexeme, Lexeme::Number("42".into(), 42.0));
        assert_eq!(tokens[1].lexeme, Lexeme::Eof);
    }

    #[test]
    fn test_scanner_invalid_character() {
        let mut scanner = Scanner::new("@", true);
        let result = scanner.scan();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error, 65);
    }

    #[test]
    fn test_scanner_complex_program() {
        let source = r#"
            fun fibonacci(n) {
                if (n <= 1) return n;
                return fibonacci(n - 1) + fibonacci(n - 2);
            }

            print fibonacci(10);
        "#;

        let mut scanner = Scanner::new(source, true);
        let tokens = scanner.scan().unwrap();

        // Should contain fun, identifier, (, identifier, ), {, if, (, etc.
        assert!(tokens.len() > 20);
        assert!(tokens.iter().any(|t| matches!(t.lexeme, Lexeme::Fun)));
        assert!(tokens.iter().any(|t| matches!(t.lexeme, Lexeme::If)));
        assert!(tokens.iter().any(|t| matches!(t.lexeme, Lexeme::Return)));
        assert!(tokens.iter().any(|t| matches!(t.lexeme, Lexeme::Print)));
    }

    #[test]
    fn test_lexeme_from_helper() {
        assert_eq!(Lexeme::from("("), Lexeme::LeftParen);
        assert_eq!(Lexeme::from("true"), Lexeme::True);
    }

    #[test]
    fn test_keyword_token_helper() {
        assert_eq!(keyword_token("true"), Some(Lexeme::True));
        assert_eq!(keyword_token("false"), Some(Lexeme::False));
        assert_eq!(keyword_token("identifier"), None);
    }

    #[test]
    fn test_one_of_helper() {
        assert!(one_of('a', "abc"));
        assert!(one_of('\n', "\n\r"));
        assert!(!one_of('x', "abc"));
        assert!(!one_of('a', ""));
    }
}
