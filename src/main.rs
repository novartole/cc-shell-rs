use std::{
    error,
    io::{self, Write},
    process,
};

use thiserror::Error;

enum Cmd<'input> {
    Exit(i32),
    Echo(&'input str),
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

impl<'input> TryFrom<&'input str> for Cmd<'input> {
    type Error = AppError;

    fn try_from(input: &'input str) -> Result<Self, Self::Error> {
        use AppError::BadArg;

        let cmd = if let Some(arg) = input.strip_prefix("exit ").map(str::trim) {
            let code = if arg.is_empty() {
                return Err(BadArg("exit code is required"));
            } else {
                arg.parse()
                    .map_err(|_| BadArg("failed to parse exit code"))?
            };

            Cmd::Exit(code)
        } else if let Some(msg) = input.strip_prefix("echo ") {
            Cmd::Echo(msg)
        } else {
            return Err(AppError::CmdNotFound);
        };

        Ok(cmd)
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
            Cmd::Echo(msg) => println!("{}", msg),
        }
    }
}
