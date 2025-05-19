use clap::{Parser, Subcommand};
use std::{
    fs,
    io::{BufRead as _, Write as _, stdout},
    path::PathBuf,
    process::ExitCode,
};

use miette::{IntoDiagnostic, Result, WrapErr};
use token::Scanner;

mod parser;
mod token;

#[derive(Parser)]
struct Lox {
    #[command(subcommand)]
    commands: Option<LoxCommands>,
}

#[derive(Subcommand)]
enum LoxCommands {
    Tokenize { filename: PathBuf },
    Parse { filename: PathBuf },
    Evaluate { filename: PathBuf },
    Run { filename: PathBuf },
}

fn main() -> Result<ExitCode, miette::Error> {
    env_logger::init();

    let lox = Lox::parse();

    #[allow(unused_variables)]
    let result = match &lox.commands {
        Some(command) => match command {
            LoxCommands::Tokenize { filename } => {
                let source = get_source(filename)?;
                let mut scanner = Scanner::new(source.as_str());
                match scanner.scan() {
                    Ok(t) => {
                        t.iter().for_each(|t| println!("{t}"));
                        0
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        65
                    }
                }
            }
            LoxCommands::Parse { filename } => {
                let source = get_source(filename)?;
                let mut parser = parser::Parser::new(source.as_str());
                match parser.parse() {
                    Ok(r) => {
                        r.iter().for_each(|t| println!("{t}"));
                        0
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        65
                    }
                }
            }
            LoxCommands::Evaluate { filename } => {
                let source = get_source(filename)?;
                0
            }
            LoxCommands::Run { filename } => {
                let source = get_source(filename)?;
                0
            }
        },
        None => match repl() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
    };

    Ok(ExitCode::from(result))
}

pub fn repl() -> Result<u8, miette::Error> {
    let mut stdin = std::io::stdin().lock();
    loop {
        let mut expr = String::new();
        print!("> ");
        let _ = stdout().flush();
        let _ = stdin
            .read_line(&mut expr)
            .map_err(miette::Error::from_err)?;

        let source = expr.trim_end();

        if source == "q" || source == "quit" {
            break;
        }

        Scanner::new(source)
            .scan()?
            .iter()
            .for_each(|t| println!("{t}"));
    }

    Ok(0)
}

fn get_source(filename: &PathBuf) -> Result<String, miette::Report> {
    let source = fs::read_to_string(filename)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read {}", filename.display()))?;
    Ok(source)
}
