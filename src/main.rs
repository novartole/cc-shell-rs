use std::{
    env, error, fs,
    io::{self, Write},
    process,
};

use thiserror::Error;

enum Cmd<'input> {
    Exit(i32),
    Echo(&'input str),
    Type(&'input str),
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
        use AppError::{BadArg, CmdNotFound};

        match input.split_once(' ') {
            Some((cmd, args)) => {
                let cmd = match cmd {
                    "exit" => {
                        let arg = args.trim();

                        if arg.is_empty() {
                            return Err(BadArg("exit code is required"));
                        }

                        let code = arg
                            .parse()
                            .map_err(|_| BadArg("failed to parse exit code"))?;

                        Cmd::Exit(code)
                    }
                    "echo" => Cmd::Echo(args),
                    "type" => {
                        let cmd = args.trim();

                        Cmd::Type(cmd)
                    }
                    _ => return Err(CmdNotFound),
                };

                Ok(cmd)
            }
            None => Err(CmdNotFound),
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

    'repl: loop {
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
            Cmd::Type(cmd) => {
                if matches!(cmd, "exit" | "echo" | "type") {
                    println!("{} is a shell builtin", cmd);
                    continue;
                }

                if let Ok(val) = env::var("PATH") {
                    for dir in val.split(':') {
                        for entry in fs::read_dir(dir)? {
                            let entry = entry?;

                            if entry.file_type().as_ref().is_ok_and(fs::FileType::is_file)
                                && entry.file_name() == cmd
                            {
                                println!("{0} is {1}/{0}", cmd, dir);
                                continue 'repl;
                            }
                        }
                    }
                }

                println!("{} not found", cmd,)
            }
        }
    }
}
