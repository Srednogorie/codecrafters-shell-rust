mod commands;
mod enums;
mod structs;
mod utils;
mod custom_rustyline;


use enums::{Commands, SpecialTokens};
use std::io::{self, Write};
use structs::{ParseCommandTokens, RedirectInfo};
use utils::check_unknown_command;
use rustyline::{Editor, Result};

use crate::custom_rustyline::ShellCompleter;

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

fn parse_tokens(iter: &[String]) -> ParseCommandTokens<'_> {
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
            "2>" => {
                special_token = Some(SpecialTokens::StdErr);
                has_special_token = true;
            }
            ">>" => {
                special_token = Some(SpecialTokens::StdAppend);
                has_special_token = true;
            }
            "1>>" => {
                special_token = Some(SpecialTokens::StdAppendExtended);
                has_special_token = true;
            }
            "2>>" => {
                special_token = Some(SpecialTokens::ErrAppend);
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

    return ParseCommandTokens { command, command_args, special_token, special_token_arg };
}

fn main() -> Result<()> {
    let mut rl = Editor::new()?;
    rl.set_helper(Some(ShellCompleter));

    loop {
        let input = rl.readline("$ ")?;

        let iter = parse_input(&input);
        if iter.is_empty() {
            continue;
        }

        let parsed_command_tokens = parse_tokens(&iter);

        let (mut stdout_writer, mut stderr_writer): (Box<dyn Write>, Box<dyn Write>) = match &parsed_command_tokens
            .special_token
        {
            Some(token) => {
                let file = Box::new(token.open_file(parsed_command_tokens.special_token_arg.unwrap())?);
                if token.is_stdout_redirect() { (file, Box::new(io::stderr())) } else { (Box::new(io::stdout()), file) }
            }
            None => (Box::new(io::stdout()), Box::new(io::stderr())),
        };

        match Commands::from_str(parsed_command_tokens.command.as_str(), parsed_command_tokens.command_args.as_slice())
        {
            Ok(Some(cmd)) => {
                if let Err(e) = cmd.execute(&mut *stdout_writer, &mut *stderr_writer) {
                    eprintln!("{}", e);
                }
            },
            Ok(None) => {
                if let Err(e) = check_unknown_command(
                    parsed_command_tokens.command,
                    parsed_command_tokens.command_args,
                    true,
                    RedirectInfo {
                        special_token: parsed_command_tokens.special_token,
                        special_token_arg: parsed_command_tokens.special_token_arg,
                    },
                ) {
                    eprintln!("{}", e);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }    
}
