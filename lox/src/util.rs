use std::collections::HashSet;
use std::sync::Arc;

use crate::model::{Ast, AstExpr, AstStmt};

pub fn print_ast(ast: &[Ast]) {
    const INDENT: &str = "    ";
    fn print_ast_node(ast: &Ast, level: usize) {
        let indent = INDENT.repeat(level);
        match ast {
            Ast::Class => println!("{indent}Class"),
            Ast::Function { name, params, body } => {
                let params = params
                    .iter()
                    .map(|s| format!("{indent}{s}"))
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("{indent}Function {name}({params})");
                print_ast_node(body, level + 1);
            }
            Ast::Variable { name, initializer } => {
                println!("{indent}Variable({name:?}, {initializer:?}),");
            }
            Ast::Block(nodes) => {
                println!("{indent}Block(");
                for node in nodes {
                    print_ast_node(node, level + 1);
                }
                println!("{indent})");
            }
            Ast::Statement(stmt) => {
                println!("{indent}Statement(");
                print_ast_stmt(stmt, level + 1);
                println!("{indent})");
            }
            Ast::Expression(expr) => {
                println!("{indent}Expression(");
                print_ast_expr(expr, level + 1);
                println!("{indent})");
            }
        }
    }

    fn print_ast_stmt(stmt: &AstStmt, level: usize) {
        let indent = INDENT.repeat(level);
        match stmt {
            AstStmt::If {
                condition,
                then,
                or_else,
            } => {
                println!("{indent}If(");
                print_ast_expr(condition, level + 1);
                println!("{indent},");
                print_ast_node(then, level + 1);
                if let Some(else_block) = or_else {
                    println!("{indent},");
                    print_ast_node(else_block, level + 1);
                }
                println!("{indent})");
            }
            AstStmt::While(cond, body) => {
                println!("{indent}While(");
                print_ast_expr(cond, level + 1);
                println!("{indent},");
                print_ast_node(body, level + 1);
                println!("{indent})");
            }
            AstStmt::Return(expr) => {
                println!("{indent}Return({expr:?})");
            }
            AstStmt::Print(expr) => {
                println!("{indent}Print(");
                print_ast_expr(expr, level + 1);
                println!("{indent})");
            }
            AstStmt::Expression(expr) => {
                print_ast_expr(expr, level);
            }
        }
    }

    fn print_ast_expr(expr: &AstExpr, level: usize) {
        let indent = INDENT.repeat(level);
        match expr {
            AstExpr::Call {
                callee,
                args,
                site: _,
            } => {
                println!("{indent}Call {{");
                println!("{indent}    callee: {callee}");
                println!("{indent}    args: [");
                for arg in args.iter() {
                    print_ast_expr(arg, level + 2);
                }
                println!("{indent}    ]");
                println!("{indent}}}");
            }
            AstExpr::Assignment { id, expr } => {
                println!("{indent}Assignment {{ id: \"{id}\", expr: {expr:?} }}");
            }
            AstExpr::Logical { op, left, right } => {
                println!("{indent}Logical {{ op: {op:?}, left: {left:?}, right: {right:?} }}");
            }
            AstExpr::Terminal(token) => {
                println!("{indent}Terminal({token:?})");
            }
            AstExpr::Group(expr) => {
                println!("{indent}Group(");
                print_ast_expr(expr, level + 1);
                println!("{indent})");
            }
            AstExpr::Unary { op, exp } => {
                println!("{indent}Unary {{ op: {op:?}, exp: {exp:?} }}");
            }
            AstExpr::Binary { op, left, right } => {
                println!("{indent}Binary {{ op: {op:?}, left: {left:?}, right: {right:?} }}");
            }
        }
    }

    println!("AST:");
    for node in ast {
        print_ast_node(node, 0);
    }
}

/// A thread-safe string interning system using `Arc<str>` for shared string storage.
///
/// This provides memory-efficient string deduplication by storing unique strings
/// as `Arc<str>` and returning cloned references for identical strings.
#[derive(Debug, Clone, Default)]
pub struct StringInterner {
    strings: HashSet<Arc<str>>,
}

#[allow(dead_code)]
impl StringInterner {
    /// Creates a new empty string interner.
    pub fn new() -> Self {
        Self {
            strings: HashSet::new(),
        }
    }

    /// Interns a string slice, returning an `Arc<str>` reference.
    ///
    /// If the string has been interned before, returns a clone of the existing
    /// `Arc<str>`. Otherwise, creates a new `Arc<str>` and stores it.
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            Arc::clone(interned)
        } else {
            let interned: Arc<str> = s.into();
            self.strings.insert(Arc::clone(&interned));
            interned
        }
    }

    /// Interns a `String`, returning an `Arc<str>` reference.
    ///
    /// This is a convenience method for interning owned strings.
    pub fn intern_string(&mut self, s: String) -> Arc<str> {
        self.intern(&s)
    }

    /// Returns the number of unique strings currently interned.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns true if no strings are currently interned.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Clears all interned strings, freeing memory.
    pub fn clear(&mut self) {
        self.strings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_new_interner() {
        let interner = StringInterner::new();
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());
    }

    #[test]
    fn test_default_interner() {
        let interner = StringInterner::default();
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());
    }

    #[test]
    fn test_intern_single_string() {
        let mut interner = StringInterner::new();
        let result = interner.intern("hello");

        assert_eq!(result.as_ref(), "hello");
        assert_eq!(interner.len(), 1);
        assert!(!interner.is_empty());
    }

    #[test]
    fn test_intern_duplicate_strings() {
        let mut interner = StringInterner::new();
        let first = interner.intern("hello");
        let second = interner.intern("hello");

        // Should return the same Arc<str> reference
        assert_eq!(first, second);
        assert_eq!(Arc::as_ptr(&first), Arc::as_ptr(&second));
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn test_intern_multiple_different_strings() {
        let mut interner = StringInterner::new();
        let hello = interner.intern("hello");
        let world = interner.intern("world");
        let foo = interner.intern("foo");

        assert_eq!(hello.as_ref(), "hello");
        assert_eq!(world.as_ref(), "world");
        assert_eq!(foo.as_ref(), "foo");
        assert_eq!(interner.len(), 3);
        assert!(!interner.is_empty());
    }

    #[test]
    fn test_intern_string_method() {
        let mut interner = StringInterner::new();
        let owned_string = "hello".to_string();
        let result = interner.intern_string(owned_string);

        assert_eq!(result.as_ref(), "hello");
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn test_intern_string_vs_intern_consistency() {
        let mut interner = StringInterner::new();
        let from_str = interner.intern("hello");
        let from_string = interner.intern_string("hello".to_string());

        // Both methods should return the same interned reference
        assert_eq!(from_str, from_string);
        assert_eq!(Arc::as_ptr(&from_str), Arc::as_ptr(&from_string));
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn test_clear_interner() {
        let mut interner = StringInterner::new();
        interner.intern("hello");
        interner.intern("world");

        assert_eq!(interner.len(), 2);
        assert!(!interner.is_empty());

        interner.clear();

        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());
    }

    #[test]
    fn test_intern_empty_string() {
        let mut interner = StringInterner::new();
        let empty = interner.intern("");

        assert_eq!(empty.as_ref(), "");
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn test_intern_unicode_strings() {
        let mut interner = StringInterner::new();
        let emoji = interner.intern("🦀");
        let chinese = interner.intern("你好");
        let accented = interner.intern("café");

        assert_eq!(emoji.as_ref(), "🦀");
        assert_eq!(chinese.as_ref(), "你好");
        assert_eq!(accented.as_ref(), "café");
        assert_eq!(interner.len(), 3);
    }

    #[test]
    fn test_clone_interner() {
        let mut interner1 = StringInterner::new();
        interner1.intern("hello");
        interner1.intern("world");

        let interner2 = interner1.clone();

        // Both should have the same length
        assert_eq!(interner1.len(), interner2.len());
        assert_eq!(interner1.len(), 2);

        // But they should be independent instances
        let mut interner3 = interner2.clone();
        interner3.intern("new");

        assert_eq!(interner1.len(), 2);
        assert_eq!(interner2.len(), 2);
        assert_eq!(interner3.len(), 3);
    }

    #[test]
    fn test_debug_format() {
        let mut interner = StringInterner::new();
        interner.intern("test");

        let debug_str = format!("{:?}", interner);
        assert!(debug_str.contains("StringInterner"));
    }

    #[test]
    fn test_intern_long_string() {
        let mut interner = StringInterner::new();
        let long_string = "a".repeat(1000);
        let result = interner.intern(&long_string);

        assert_eq!(result.len(), 1000);
        assert_eq!(interner.len(), 1);

        // Interning the same long string should reuse the Arc
        let result2 = interner.intern(&long_string);
        assert_eq!(Arc::as_ptr(&result), Arc::as_ptr(&result2));
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn test_intern_whitespace_strings() {
        let mut interner = StringInterner::new();
        let space = interner.intern(" ");
        let tab = interner.intern("\t");
        let newline = interner.intern("\n");
        let multiple_spaces = interner.intern("   ");

        assert_eq!(space.as_ref(), " ");
        assert_eq!(tab.as_ref(), "\t");
        assert_eq!(newline.as_ref(), "\n");
        assert_eq!(multiple_spaces.as_ref(), "   ");
        assert_eq!(interner.len(), 4);
    }
}
