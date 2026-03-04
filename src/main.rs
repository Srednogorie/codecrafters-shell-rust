mod utils;
mod enums;
mod commands;

use std::io::{self, Write};
use utils::{check_unknown_command};
use enums::{Commands, SpecialTokens};


fn take_input(input: &mut String) {
    input.clear();
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
}

fn parse_input(input: &str) -> Vec<String> {
    let input = input.replace("''", "");
    let input = input.replace("\"\"", "");
    // read_line` includes the **newline character** `'\n'` at the end of the input
    // trim it or it's going to sneak into the command
    let input = input.trim();
    
    let mut tokens = Vec::new();
    
    let mut token = String::new();
    let mut inside_single_quote = false;
    let mut inside_double_quote = false;
    let mut escape_next = false;
    for char in input.chars() {
        if escape_next {
            token.push(char);
            escape_next = false;
            continue;
        }
        match char {
            ' ' => {
                if inside_single_quote || inside_double_quote {
                    token.push(char);
                } else {
                    if !token.is_empty() {
                        tokens.push(std::mem::take(&mut token))
                    }
                }
            }
            '\'' => {
                if inside_double_quote {
                    token.push(char);
                    continue;
                }
                if inside_single_quote {
                    inside_single_quote = false;
                } else {
                    inside_single_quote = true;
                }
            }
            '"' => {
                if inside_single_quote {
                    token.push(char);
                    continue;
                }
                if inside_double_quote {
                    inside_double_quote = false;
                } else {
                    inside_double_quote = true;
                }
            }
            '\\' => {
                if inside_single_quote {
                    token.push(char);
                    continue;
                }
                escape_next = true;
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

fn parse_tokens(iter: &[String]) -> (&String, Vec<String>, Option<SpecialTokens>, Option<&str>) {
    let (command, command_args_raw) = iter.split_first().unwrap();
    let mut special_token = None;
    let mut special_token_arg = None;
    let mut command_args = Vec::new();
    let mut has_special_token = false;
    for arg in command_args_raw {
        match arg.as_str() {
            ">" => {
                special_token = Some(SpecialTokens::StdOut);
                has_special_token = true;   
            }
            "1>" => {
                special_token = Some(SpecialTokens::StdOutExtended);
                has_special_token = true;   
            }
            _ => {
                if has_special_token {
                    special_token_arg = Some(arg.as_str());
                } else {
                    command_args.push(arg.to_string());
                }
            }
        }
    }
    
    return (command, command_args, special_token, special_token_arg)
}

fn main() {
    let mut input = String::new();
    loop {
        take_input(&mut input);
        let iter = parse_input(&input);
        if iter.is_empty() {
            continue;
        }
        
        let (command, command_args, special_token, special_token_arg) = parse_tokens(&iter);
        // After parsing...
        let mut output: Box<dyn Write> = if special_token.is_some() {
            // Redirect to file
            Box::new(
                std::fs::File::create(special_token_arg.as_ref().unwrap())
                    .expect("Failed to create file")
            )
        } else {
            // Normal stdout
            Box::new(std::io::stdout())
        };
        
        match Commands::from_str(command, command_args.as_slice()) {
            Some(cmd) => cmd.execute(&mut *output),
            None => {
                match Some(special_token) {
                    Some(special_token) => check_unknown_command(
                        command, command_args, true, special_token, special_token_arg
                    ),
                    None => check_unknown_command(command, command_args.to_vec(), true, None, None),
                }
            },
        }
    }
}
