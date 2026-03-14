mod commands;
mod custom_rustyline;
mod enums;
mod structs;
mod utils;

use enums::{Commands, SpecialTokens};
use rustyline::config::Config;
use rustyline::{CompletionType, Editor, Result};
use std::io::{self, Write};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::process::Stdio;

use crate::custom_rustyline::ShellCompleter;
use crate::structs::{PipelineStage, Redirect};
use crate::utils::execute_external;

fn tokens_to_stage(tokens: Vec<String>) -> PipelineStage {
    let (command, command_args_raw) = tokens.split_first().unwrap();
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
    // instead of unconditionally building Redirect and unwrapping:
    if let (Some(token), Some(target)) = (special_token, special_token_arg) {
        PipelineStage {
            command: command.to_string(),
            args: command_args,
            redirect: Some(Redirect { token, target: target.to_string() }),
        }
    } else {
        PipelineStage { command: command.to_string(), args: command_args, redirect: None }
    }
}

fn parse_input(input: &str) -> Vec<PipelineStage> {
    let input = input.replace("''", "");
    let input = input.replace("\"\"", "");
    // read_line` includes the **newline character** `'\n'` at the end of the input
    // trim it or it's going to sneak into the command
    let input = input.trim();

    let mut tokens_set: Vec<PipelineStage> = Vec::new();

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
                // flush current token before splitting
                if !token.is_empty() {
                    tokens.push(std::mem::take(&mut token));
                }

                tokens_set.push(tokens_to_stage(tokens.clone()));
                tokens.clear();

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
        tokens_set.push(tokens_to_stage(tokens.clone()));
    }
    tokens_set
}

fn execute_pipeline(stages: Vec<PipelineStage>) -> Result<()> {
    let mut previous_stdout: Option<std::fs::File> = None;
    let mut children = Vec::new();

    for (i, stage) in stages.iter().enumerate() {
        let stdin = match previous_stdout.take() {
            Some(file) => Stdio::from(file),
            None => Stdio::inherit(),
        };

        let stdout = if i < stages.len() - 1 { Stdio::piped() } else { Stdio::inherit() };

        match Commands::from_str(stage.command.as_str(), stage.args.as_slice()) {
            Ok(Some(cmd)) => {
                let mut stdout_cursor = std::io::Cursor::new(Vec::<u8>::new());
                let mut stderr_cursor = std::io::Cursor::new(Vec::<u8>::new());
                let _ = cmd.execute(&mut stdout_cursor, &mut stderr_cursor);

                if i < stages.len() - 1 {
                    // not the last stage — feed output into a pipe
                    let (read_fd, write_fd) = nix::unistd::pipe().unwrap();
                    let mut write_end = unsafe { std::fs::File::from_raw_fd(write_fd.into_raw_fd()) };
                    write_end.write_all(stdout_cursor.get_ref()).unwrap();
                    drop(write_end);
                    previous_stdout = Some(unsafe { std::fs::File::from_raw_fd(read_fd.into_raw_fd()) });
                } else {
                    match &stage.redirect {
                        Some(redirect) if redirect.token.is_stdout_redirect() => {
                            let mut file = redirect.token.open_file(&redirect.target)?;
                            file.write_all(stdout_cursor.get_ref()).unwrap();
                            io::stderr().write_all(stderr_cursor.get_ref()).unwrap();
                        }
                        Some(redirect) => {
                            // stderr redirect
                            let mut file = redirect.token.open_file(&redirect.target)?;
                            file.write_all(stderr_cursor.get_ref()).unwrap();
                            io::stdout().write_all(stdout_cursor.get_ref()).unwrap();
                        }
                        None => {
                            io::stdout().write_all(stdout_cursor.get_ref()).unwrap();
                            io::stderr().write_all(stderr_cursor.get_ref()).unwrap();
                        }
                    }
                }
            }
            Ok(None) => {
                let result = execute_external(stage, stdin, stdout, Stdio::inherit());
                match result {
                    Ok(mut child) => {
                        if i < stages.len() - 1 {
                            previous_stdout =
                                child.stdout.take().map(|s| unsafe { std::fs::File::from_raw_fd(s.into_raw_fd()) });
                        }
                        children.push(child);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    for mut child in children {
        child.wait().unwrap();
    }
    Ok(())
}

fn main() -> Result<()> {
    let config = Config::builder().completion_type(CompletionType::List).build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(ShellCompleter));
    loop {
        let input = rl.readline("$ ")?;
        let stages = parse_input(&input);
        if stages.is_empty() {
            continue;
        }
        if let Err(e) = execute_pipeline(stages) {
            eprintln!("{}", e);
        }
    }
}
