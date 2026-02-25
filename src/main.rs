mod utils;
mod enums;
mod commands;

use std::io::{self, Write};
use utils::check_unknown_command;
use enums::Commands;


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
