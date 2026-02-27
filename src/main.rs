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

fn parse_input(input: &str) -> Vec<String> {
    let input = input.replace("''", "");
    // read_line` includes the **newline character** `'\n'` at the end of the input
    // trim it or it's going to sneak into the command
    let input = input.trim();
    
    let mut tokens = Vec::new();
    
    let mut token = String::new();
    let mut inside_single_quote = false;
    for char in input.chars() {
        match char {
            ' ' => {
                if inside_single_quote {
                    token.push(char);
                } else {
                    if !token.is_empty() {
                        tokens.push(std::mem::take(&mut token))
                    }
                }
            }
            '\'' => {
                if inside_single_quote {
                    inside_single_quote = false;
                } else {
                    inside_single_quote = true;
                }
            }
            _ => {
                token.push(char);
            }
        }
    }
    
    if !token.is_empty() {
        tokens.push(std::mem::take(&mut token));
    }
    tokens
}

fn main() {
    let mut input = String::new();
    loop {
        take_input(&mut input);
        let iter = parse_input(&input);
        
        let (command, args) = iter.split_first().unwrap();
        
        match Commands::from_str(command, args) {
            Some(cmd) => cmd.execute(),
            None => check_unknown_command(command, args.to_vec(), true),
        }
    }
}
