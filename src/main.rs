#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let mut input = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let cmd = read(&mut input);

        if let Err(e) = eval(cmd) {
            println!("{}", e);
        }
    }
}

fn read(input: &mut String) -> &str {
    let stdin = io::stdin();

    stdin.read_line(input).unwrap();

    input.trim_end()
}

fn eval(cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
    Err(format!("{}: command not found", cmd).into())
}
