use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::trace;
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
}

#[derive(Subcommand)]
enum LoxCommands {
    Tokenize { filename: Option<String> },
    Parse { filename: Option<String> },
    Evaluate { filename: Option<String> },
    Run { filename: Option<String> },
}

fn main() -> Result<ExitCode> {
    env_logger::init();

    let lox = Lox::parse();

    match lox.commands {
        LoxCommands::Tokenize { filename } => {
            if let Some(file) = filename {
                let source = get_source(file)?;
                let scanner = Scanner::new(&source);
                scanner
                    .scan()
                    .map_err(|e| eprintln!("{e}"))
                    .iter()
                    .flatten()
                    .for_each(|t| println!("{t}"));
            } else {
                // TODO: usage error, filename required
            }
        }
        LoxCommands::Parse { filename } => {
            if let Some(file) = filename {
                let _source = get_source(file)?;
            } else {
                // TODO: usage error, filename required
            }
        }

        LoxCommands::Evaluate { filename } => {
            if let Some(file) = filename {
                let _source = get_source(file)?;
            } else {
                // TODO: usage error, filename required
            }
        }

        LoxCommands::Run { filename } => {
            if let Some(file) = filename {
                let _source = get_source(file)?;
            } else {
                let _ = repl();
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
    trace!("get_source({filename})");
    let source = fs::read_to_string(PathBuf::from(&filename)).with_context(|| filename)?;

    Ok(source)
}
