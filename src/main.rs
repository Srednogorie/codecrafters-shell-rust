mod commands;
mod utils;

use std::io::{self, Write};
use commands::{command_echo, command_exit, command_type};
use utils::check_unknown_command;

enum Commands {
    Echo(Vec<String>),
    Type(Vec<String>),
    Exit,
}
impl Commands {
    fn from_str(command: &str, args: &[String]) -> Option<Commands> {
        match command {
            "echo" => Some(Commands::Echo(args.to_vec())),
            "type" => Some(Commands::Type(args.to_vec())),
            "exit" => Some(Commands::Exit),
            _ => None,
        }
    }
    fn execute(&self) {
        match self {
            Commands::Echo(args) => command_echo(args),
            Commands::Type(args) => command_type(args),
            Commands::Exit => command_exit(),
        }
    }
}

fn take_input(input: &mut String) {
    input.clear();
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
}

fn main() {
    let mut input = String::new();
    loop {
        take_input(&mut input);
        let mut iter = input.split_whitespace();
        
        let command = match iter.next() {
            Some(cmd) => cmd,
            None => continue,  // empty input, no need for the is_empty check
        };

        let args: Vec<String> = iter.map(|s| s.to_string()).collect();

        match Commands::from_str(command, &args) {
            Some(cmd) => cmd.execute(),
            // None => println!("{}: command not found", input.trim()),
            None => check_unknown_command(command, args, true),
        }
    }
}
