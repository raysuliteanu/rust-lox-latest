use clap::Parser;
use parser::Ast;
use std::{
    fs,
    io::{BufRead as _, Write as _, stdout},
    path::PathBuf,
    process::ExitCode,
};

use miette::{IntoDiagnostic, Result, WrapErr};

mod parser;
mod token;

#[derive(Parser)]
struct Lox {
    filename: Option<String>,
}

fn main() -> miette::Result<ExitCode> {
    env_logger::init();

    let lox = Lox::parse();

    let result = if let Some(filename) = lox.filename {
        let source = get_source(filename)?;
        parser::Parser::new(source.as_str()).parse()
    } else {
        repl()
    };

    let rc = match result {
        Ok(t) => {
            t.iter().for_each(|t| println!("{t}"));
            0
        }
        Err(e) => {
            eprintln!("{e:?}");
            e.code()
                .unwrap_or(Box::new("1"))
                .to_string()
                .parse::<u8>()
                .into_diagnostic()?
        }
    };

    Ok(ExitCode::from(rc))
}

pub fn repl() -> Result<Vec<Ast>, miette::Error> {
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

        let r = parser::Parser::new(source).parse();
        match r {
            Ok(v) => v.iter().for_each(|t| println!("{t}")),
            Err(e) => return Err(e),
        }
    }

    Ok(vec![])
}

fn get_source(filename: String) -> Result<String, miette::Report> {
    let source = fs::read_to_string(PathBuf::from(&filename))
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read {filename}"))?;
    Ok(source)
}
