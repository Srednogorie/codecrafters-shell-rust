mod commands;
mod custom_rustyline;
mod enums;
mod structs;
mod utils;

use enums::{Commands, SpecialTokens};
use rustyline::config::Config;
use rustyline::{CompletionType, Editor, Result};
use std::io::{self, Read, Write};
use std::ops::SubAssign;
use std::process::Stdio;
use structs::{ParseCommandTokens, RedirectInfo};
use utils::check_unknown_command;
use std::os::unix::io::{FromRawFd, IntoRawFd};

use crate::commands::{command_echo, command_type};
use crate::custom_rustyline::ShellCompleter;

fn parse_input(input: &str) -> Vec<Vec<String>> {
    let input = input.replace("''", "");
    let input = input.replace("\"\"", "");
    // read_line` includes the **newline character** `'\n'` at the end of the input
    // trim it or it's going to sneak into the command
    let input = input.trim();

    let mut tokens_set: Vec<Vec<String>> = Vec::new();

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
            '|' => {
                if inside_single_quote || inside_double_quote {
                    token.push(char);
                    continue;
                }
                tokens_set.push(std::mem::take(&mut tokens));
                inside_single_quote = false;
                inside_double_quote = false;
                escape_next = false;
            }
            _ => {
                token.push(char);
            }
        }
    }

    if !token.is_empty() {
        tokens.push(std::mem::take(&mut token));
        tokens_set.push(std::mem::take(&mut tokens));
    }
    tokens_set
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
    let config = Config::builder().completion_type(CompletionType::List).build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(ShellCompleter));

    loop {
        let input = rl.readline("$ ")?;

        let iters = parse_input(&input);

        if iters.len() == 1 {
            let iter = &iters[0];

            if iter.is_empty() {
                continue;
            }

            let parsed_command_tokens = parse_tokens(&iter);

            let (mut stdout_writer, mut stderr_writer): (Box<dyn Write>, Box<dyn Write>) =
                match &parsed_command_tokens.special_token {
                    Some(token) => {
                        let file = Box::new(token.open_file(parsed_command_tokens.special_token_arg.unwrap())?);
                        if token.is_stdout_redirect() {
                            (file, Box::new(io::stderr()))
                        } else {
                            (Box::new(io::stdout()), file)
                        }
                    }
                    None => (Box::new(io::stdout()), Box::new(io::stderr())),
                };

            match Commands::from_str(
                parsed_command_tokens.command.as_str(),
                parsed_command_tokens.command_args.as_slice(),
            ) {
                Ok(Some(cmd)) => {
                    if let Err(e) = cmd.execute(&mut *stdout_writer, &mut *stderr_writer) {
                        eprintln!("{}", e);
                    }
                }
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
        } else {
            let mut previous_stdout: Option<std::fs::File> = None;
            let mut children = Vec::new();
            
            let builtins: Vec<String> =
                vec!["echo", "type", "pwd", "cd"].into_iter().map(String::from).collect();

            for (i, cmd) in iters.iter().enumerate() {
                let cmd_str = cmd.first().unwrap().to_string();
                
                if builtins.contains(&cmd_str) {
                    match cmd_str.as_str() {
                        "echo" | "type" => {
                            if i == 0 {
                                let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
                                let _ = command_echo(&cmd[1..], &mut cursor);
                                
                                // Create an OS pipe - returns (OwnedFd, OwnedFd)
                                let (read_fd, write_fd) = nix::unistd::pipe().unwrap();
                                
                                // Convert OwnedFd to raw fd, then to File
                                let write_fd_raw = write_fd.into_raw_fd(); // Consumes ownership
                                let mut write_end = unsafe { std::fs::File::from_raw_fd(write_fd_raw) };
                                
                                write_end.write_all(cursor.get_ref()).unwrap();
                                drop(write_end); // Important! Close write end
                                
                                // Same for read end
                                let read_fd_raw = read_fd.into_raw_fd();
                                let read_end = unsafe { std::fs::File::from_raw_fd(read_fd_raw) };
                                previous_stdout = Some(read_end); // Store File, not Stdio::from(read_end)
                            } else if i == iters.len() - 1 {
                                if let Some(mut stdin_file) = previous_stdout.take() {
                                    let mut buf = Vec::new();
                                    stdin_file.read_to_end(&mut buf).unwrap();
                                    // let mut cursor = std::io::Cursor::new(buf);
                                    let _ = command_type(&cmd[1..], &mut std::io::stdout());
                                    // Note: echo doesn't use stdin, but if you had a builtin that did:
                                    // let _ = command_wc(&mut cursor, &mut std::io::stdout());
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                let mut command = std::process::Command::new(cmd.first().unwrap());

                if cmd.len() > 1 {
                    command.args(&cmd[1..]);
                }

                // stdin comes from previous command
                if let Some(stdout) = previous_stdout.take() {
                    command.stdin(Stdio::from(stdout)); // Convert File → Stdio here instead
                }

                // pipe stdout unless this is the last command
                if i < iters.len() - 1 {
                    command.stdout(std::process::Stdio::piped());
                } else {
                    command.stdout(std::process::Stdio::inherit());
                }

                let mut child = command.spawn().unwrap();
                previous_stdout = child.stdout.take().map(|stdout| {
                    unsafe { std::fs::File::from_raw_fd(stdout.into_raw_fd()) }
                });
                children.push(child);
            }

            for mut child in children {
                child.wait().unwrap();
            }
        }
    }
}
