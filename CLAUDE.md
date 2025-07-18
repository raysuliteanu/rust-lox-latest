# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of the Lox programming language interpreter, following the Crafting Interpreters book. The project is organized as a single-binary Rust application with modular components for lexical analysis, parsing, and evaluation.

## Commands

### Build and Run

- `cargo build` - Build the project
- `cargo build --release` - Build with optimizations
- `cargo run -- <command> <file>` - Run the interpreter with commands

### Testing

- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test
- `cargo test -- --test-threads=1` - Run tests serially

### Lox Interpreter Commands

The interpreter supports multiple operational modes:

- `cargo run -- tokenize <file.lox>` - Tokenize a Lox source file
- `cargo run -- parse <file.lox>` - Parse a Lox source file and display AST
- `cargo run -- evaluate <file.lox>` - Evaluate a Lox source file
- `cargo run -- run` - Start interactive REPL mode
- `cargo run -- run <file.lox>` - Run a Lox source file (not fully implemented)

## Architecture

### Core Components

1. **Lexical Analysis (token.rs)**

    - `Scanner` - Tokenizes Lox source code into tokens
    - `Token` - Represents individual tokens with lexemes and span information
    - `Lexeme` - Enumeration of all possible token types in Lox

2. **Parsing (parser.rs)**

    - `Parser` - Recursive descent parser that builds AST from tokens
    - Implements the complete Lox grammar with proper precedence
    - Comprehensive error handling with custom error types

3. **AST Model (model.rs)**

    - `Ast` - Top-level AST node types (declarations, statements, expressions)
    - `AstExpr` - Expression AST nodes (binary, unary, terminal, etc.)
    - `AstStmt` - Statement AST nodes (print, return, if, while, etc.)
    - All AST types implement `Display` for pretty-printing

4. **Evaluation (eval.rs)**

    - `Eval` - Tree-walking interpreter for executing Lox code
    - `EvalValue` - Runtime value representation
    - Currently supports basic expressions and print statements

5. **Source Location (span.rs)**
    - `Span` - Tracks line numbers and character positions for error reporting

### Grammar Implementation

The parser implements the complete Lox grammar with proper operator precedence:

- Equality operators (==, !=)
- Comparison operators (>, >=, <, <=)
- Term operators (+, -)
- Factor operators (\*, /)
- Unary operators (!, -)
- Primary expressions (literals, identifiers, grouping)

### Current Implementation Status

**Completed:**

- Full lexical analysis with all Lox tokens
- Complete parsing with proper precedence and error handling
- AST construction and display
- Basic expression evaluation
- Print statement execution
- Comprehensive test coverage

**In Progress:**

- Binary expression evaluation (stubbed)
- Variable assignment and scoping
- Control flow (if/else, while, for)
- Function definitions and calls
- Class definitions and inheritance

## Development Notes

- The project uses Rust 2024 edition
- Dependencies include clap for CLI, anyhow for error handling, and thiserror for custom errors
- Extensive unit tests are included for all major components
- The codebase follows functional programming patterns with immutable data structures where possible
- Error messages are designed to match the Crafting Interpreters specification

## Testing Approach

Tests are embedded within each module using `#[cfg(test)]`. Key test areas:

- Token scanning with various input types
- Parser functionality for all grammar rules
- AST construction and display formatting
- Error handling for malformed input
- Edge cases and boundary conditions
