# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of the Lox programming language interpreter, following the Crafting Interpreters book. The project is organized as a Cargo workspace with two main crates: the `lox` interpreter binary and a `macros` procedural macro library.

## Project Structure

```
rust-lox-latest/
├── Cargo.toml           # Workspace configuration
├── CLAUDE.md           # This file
├── Grammar.md          # Lox language grammar specification
├── README.md           # Project documentation
├── lox/                # Main interpreter crate
│   ├── Cargo.toml      # Binary crate configuration
│   └── src/
│       ├── main.rs     # CLI entry point and command handling
│       ├── token.rs    # Lexical analysis (Scanner, Token, Lexeme)
│       ├── parser.rs   # Recursive descent parser
│       ├── model.rs    # AST node definitions
│       ├── eval.rs     # Tree-walking interpreter
│       ├── func.rs     # Built-in function implementations
│       ├── span.rs     # Source location tracking
│       └── util.rs     # Utility functions
├── macros/             # Procedural macro crate (WIP)
│   ├── Cargo.toml      # Proc-macro crate configuration
│   └── src/lib.rs      # Macro implementations (placeholder)
└── test/               # Test Lox source files
    ├── *.lox           # Various test programs

```

## Commands

### Build and Run

- `cargo build` - Build the workspace (both crates)
- `cargo build --release` - Build with optimizations
- `cargo run -p rust-lox-latest -- <command> <file>` - Run the interpreter

### Testing

- `cargo test` - Run all tests across workspace
- `cargo test <test_name>` - Run specific test
- `cargo test -- --test-threads=1` - Run tests serially

### Lox Interpreter Commands

The interpreter supports multiple operational modes:

- `cargo run -p rust-lox-latest -- tokenize <file.lox>` - Tokenize a Lox source file
- `cargo run -p rust-lox-latest -- parse <file.lox>` - Parse a Lox source file and display AST
  - `--no-expression-mode` - Disable expression mode parsing
  - `--pretty-print` - Enable pretty-printed AST output
- `cargo run -p rust-lox-latest -- evaluate <file.lox>` - Evaluate a Lox source file
  - `--no-expression-mode` - Disable expression mode parsing
- `cargo run -p rust-lox-latest -- run` - Start interactive REPL mode
- `cargo run -p rust-lox-latest -- run <file.lox>` - Run a Lox source file

## Architecture

### Core Components

1. **Main Entry Point (lox/src/main.rs)**
    - CLI command parsing using clap
    - Command dispatch for tokenize, parse, evaluate, and run modes
    - REPL implementation for interactive mode

2. **Lexical Analysis (lox/src/token.rs)**
    - `Scanner` - Tokenizes Lox source code into tokens
    - `Token` - Represents individual tokens with lexemes and span information
    - `Lexeme` - Enumeration of all possible token types in Lox

3. **Parsing (lox/src/parser.rs)**
    - `Parser` - Recursive descent parser that builds AST from tokens
    - Implements the complete Lox grammar with proper precedence
    - Comprehensive error handling with custom error types
    - Support for both expression-only and full program parsing modes

4. **AST Model (lox/src/model.rs)**
    - `Ast` - Top-level AST node types (declarations, statements, expressions)
    - `AstExpr` - Expression AST nodes (binary, unary, terminal, etc.)
    - `AstStmt` - Statement AST nodes (print, return, if, while, etc.)
    - All AST types implement `Display` for pretty-printing

5. **Evaluation (lox/src/eval.rs)**
    - `Eval` - Tree-walking interpreter for executing Lox code
    - `EvalValue` - Runtime value representation
    - Environment management for variable scoping
    - Support for expressions, statements, and control flow

6. **Built-in Functions (lox/src/func.rs)**
    - Native function implementations (e.g., `clock()`)
    - Function call interface for built-ins

7. **Source Location (lox/src/span.rs)**
    - `Span` - Tracks line numbers and character positions for error reporting

8. **Utilities (lox/src/util.rs)**
    - Helper functions and common utilities

9. **Procedural Macros (macros/src/lib.rs)**
    - Placeholder for future macro implementations
    - Currently contains unimplemented `func` macro

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
- Binary expression evaluation (stubbed)
- Variable assignment and scoping
- Control flow (if/else, while, for)

**In Progress:**

- Function definitions and calls
- Class definitions and inheritance

## Development Notes

- The project uses Rust 2024 edition
- Organized as a Cargo workspace with separate crates for modularity
- Main interpreter dependencies: clap (CLI), anyhow (error handling), thiserror (custom errors), env_logger and log (logging)
- Macro crate uses syn, quote, and proc-macro2 for procedural macro development
- Extensive unit tests are included for all major components using `#[cfg(test)]`
- The codebase follows functional programming patterns with immutable data structures where possible
- Error messages are designed to match the Crafting Interpreters specification
- Test Lox programs are located in the `test/` directory for integration testing

## Testing Approach

Tests are embedded within each module using `#[cfg(test)]`. Key test areas:

- Token scanning with various input types
- Parser functionality for all grammar rules
- AST construction and display formatting
- Error handling for malformed input
- Edge cases and boundary conditions
