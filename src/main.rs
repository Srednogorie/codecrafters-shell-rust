#[allow(unused_imports)]
use std::io::{self, Write};

fn take_input(input: &mut String) {
    input.clear();
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
}

fn command_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn command_type(args: &[&str]) {
    let command = args.first().unwrap_or(&"");
    match *command {
        "echo" => println!("echo is a shell builtin"),
        "type" => println!("type is a shell builtin"),
        "exit" => println!("exit is a shell builtin"),
        _ => println!("{}: not found", command),
    }
}

fn main() {
    let mut input = String::new();
    loop {
        take_input(&mut input);
        let parsed_input: Vec<&str> = input.split_whitespace().collect();
        let command = parsed_input.first().unwrap_or(&"");
        
        if command.is_empty() {
            continue;
        }

        let args = &parsed_input[1..];

        match *command {
            "echo" => command_echo(args),
            "type" => command_type(args),
            "exit" => break,
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
