use std::{
    error,
    io::{self, Write},
    process,
};

use thiserror::Error;

enum Cmd {
    Exit(i32),
}

#[derive(Debug, Error)]
enum AppError {
    #[error("command not found")]
    CmdNotFound,
    #[error("{0}")]
    BadArg(&'static str),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl TryFrom<&str> for Cmd {
    type Error = AppError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        use AppError::BadArg;

        let mut parts = input.split_whitespace();

        match parts.next().unwrap() {
            "exit" => {
                let code = parts
                    .next()
                    .ok_or(BadArg("exit code is required"))?
                    .parse()
                    .map_err(|_| BadArg("failed to parse exit code"))?;

                let cmd = Cmd::Exit(code);

                Ok(cmd)
            }
            _ => Err(AppError::CmdNotFound),
        }
    }
}

fn main() {
    if let Err(e) = run() {
        println!("fatal error: {}", e);
    }
}

fn run() -> Result<(), Box<dyn error::Error>> {
    let mut buf = String::new();

    loop {
        // promt
        print!("$ ");
        io::stdout().flush()?;

        // read
        buf.clear();
        io::stdin().read_line(&mut buf)?;

        // parse
        let input = buf.trim_end();
        let cmd = match input.try_into() {
            Ok(cmd) => cmd,
            Err(e) => {
                if let AppError::CmdNotFound = e {
                    println!("{}: {}", input, e);
                    continue;
                }

                return Err(e.into());
            }
        };

        // eval
        match cmd {
            Cmd::Exit(code) => process::exit(code),
        }
    }
}
