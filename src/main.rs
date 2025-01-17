use std::{
    borrow::Cow,
    env, error, fs,
    io::{self, Write},
    path, process,
    str::from_utf8,
};

use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    BadArg(&'static str),
    #[error(transparent)]
    Io(#[from] io::Error),
}

enum Cmd<'input> {
    Exit(i32),
    Echo(&'input str),
    Type(&'input str),
    Pwd,
    Cd(&'input str),
    External {
        cmd: &'input str,
        args: Option<&'input str>,
    },
}

impl<'input> TryFrom<&'input str> for Cmd<'input> {
    type Error = AppError;

    fn try_from(input: &'input str) -> Result<Self, Self::Error> {
        use AppError::BadArg;

        let cmd = match input.split_once(' ') {
            Some((cmd, args)) => match cmd {
                "exit" => {
                    let arg = args.trim_start();

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
                    let cmd = args.trim_start();

                    Cmd::Type(cmd)
                }
                "cd" => {
                    let path = args.trim_start();

                    Cmd::Cd(path)
                }
                cmd => Cmd::External {
                    cmd,
                    args: Some(args),
                },
            },
            None => {
                if input == "pwd" {
                    Cmd::Pwd
                } else {
                    Cmd::External {
                        cmd: input,
                        args: None,
                    }
                }
            }
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

    'repl: loop {
        // promt
        print!("$ ");
        io::stdout().flush()?;

        // read
        buf.clear();
        io::stdin().read_line(&mut buf)?;

        // parse
        let input = buf.trim_end();
        let cmd = input.try_into()?;

        // eval
        match cmd {
            Cmd::Exit(code) => process::exit(code),
            Cmd::Echo(msg) => println!("{}", msg),
            Cmd::Type(cmd) => {
                if matches!(cmd, "exit" | "echo" | "type" | "pwd" | "cd") {
                    println!("{} is a shell builtin", cmd);
                    continue;
                }

                if let Ok(val) = env::var("PATH") {
                    for dir in val.split(':') {
                        for entry in fs::read_dir(dir)? {
                            if entry?.file_name() == cmd {
                                println!("{0} is {1}/{0}", cmd, dir);
                                continue 'repl;
                            }
                        }
                    }
                }

                println!("{} not found", cmd)
            }
            Cmd::Pwd => println!("{}", env::current_dir()?.display()),
            Cmd::Cd(path) => {
                let path = if path == "~" {
                    env::var("HOME")?.into()
                } else {
                    Cow::Borrowed(path)
                };

                if let Err(e) = env::set_current_dir(path.as_ref()) {
                    if let io::ErrorKind::NotFound = e.kind() {
                        println!("{}: No such file or directory", path);
                        continue;
                    }

                    return Err(e.into());
                }
            }
            Cmd::External {
                cmd,
                args: raw_args,
            } => {
                let path = path::Path::new(cmd);

                let cmd = match path.try_exists() {
                    Ok(true) => path.into(),
                    _ => {
                        let mut path = None;

                        if let Ok(val) = env::var("PATH") {
                            'outer: for dir in val.split(':') {
                                for entry in fs::read_dir(dir)? {
                                    let entry = entry?;

                                    if entry.file_name() == cmd {
                                        path = entry.path().into();
                                        break 'outer;
                                    }
                                }
                            }
                        }

                        match path {
                            Some(cmd) => cmd,
                            None => {
                                println!("{}: command not found", input);
                                continue 'repl;
                            }
                        }
                    }
                };

                let mut prc = process::Command::new(cmd);

                if let Some(args) = raw_args {
                    prc.args(args.split_whitespace().filter(|arg| !arg.is_empty()));
                }

                let output = prc.output()?.stdout;
                print!("{}", from_utf8(&output)?);
            }
        }
    }
}
