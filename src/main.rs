use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use parser::Ast;
use std::{
    fs,
    io::{BufRead as _, Read, Write as _, stdin, stdout},
    path::PathBuf,
    process::ExitCode,
};

use crate::token::Scanner;

mod parser;
mod span;
mod token;

#[derive(Parser)]
struct Lox {
    #[command(subcommand)]
    commands: LoxCommands,
    filename: Option<String>,
}

#[derive(Subcommand)]
enum LoxCommands {
    Tokenize,
    Parse,
    Evaluate,
    Run,
}

fn main() -> Result<ExitCode> {
    env_logger::init();

    let lox = Lox::parse();

    match lox.commands {
        LoxCommands::Tokenize => {
            if let Some(file) = lox.filename {
                let source = get_source(file)?;
                let _ = Scanner::scan(&source)?;
            } else {
                // TODO: usage error, filename required
            }
        }
        LoxCommands::Parse => {
            if let Some(file) = lox.filename {
                let source = get_source(file)?;
            } else {
                // TODO: usage error, filename required
            }
        }

        LoxCommands::Evaluate => {
            if let Some(file) = lox.filename {
                let source = get_source(file)?;
            } else {
                // TODO: usage error, filename required
            }
        }

        LoxCommands::Run => {
            if let Some(file) = lox.filename {
                let source = get_source(file)?;
            } else {
                repl();
            }
        }
    }

    let rc = 0;

    Ok(ExitCode::from(rc))
}

pub fn repl() -> anyhow::Result<Vec<Ast>> {
    let mut stdin = std::io::stdin().lock();
    loop {
        let mut expr = String::new();
        print!("> ");
        let _ = stdout().flush();
        let _ = stdin.read_line(&mut expr)?;

        let source = expr.trim_end();

        if source == "q" || source == "quit" {
            break;
        }
    }

    Ok(vec![])
}

fn get_source(filename: String) -> anyhow::Result<String> {
    let source = fs::read_to_string(PathBuf::from(&filename)).with_context(|| filename)?;

    Ok(source)
}
