use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};
use std::fmt::format;
use std::io::Write;
use std::{fs, io};

use crate::utils::get_paths;

pub struct ShellCompleter;

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<String>)> {
        let mut builtins: Vec<String> =
            vec!["echo", "type", "exit", "pwd", "cd"].into_iter().map(String::from).collect();
        // Only look at the text up to the cursor
        let word = &line[..pos];

        // Find where the current word starts (after the last space, or 0)
        let start = word.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let is_first_word = start == 0;
        let prefix = &word[start..];
        
        if is_first_word {
            let paths = get_paths().unwrap();
            for path in std::env::split_paths(&paths) {
                if let Ok(entries) = fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let file_name = entry.file_name();
                        let Some(name) = file_name.into_string().ok() else {
                            continue; // skip non-UTF8 filenames
                        };
                        if !builtins.contains(&name) {
                            builtins.push(name);
                        }
                    }
                }
            }
    
            let mut candidates: Vec<String> =
                builtins.iter().filter(|cmd| cmd.starts_with(prefix)).map(|s| format!("{} ", s)).collect();
            candidates.sort();
    
            Ok((start, candidates))
        } else {
            let split_prefix: Vec<&str> = prefix.split("/").collect();
            let mut pre = split_prefix[..split_prefix.len() - 1].join("/");
            if pre.is_empty() {
                pre = "./".to_string();
            }
            let post = split_prefix.last().unwrap();
            let paths = fs::read_dir(&pre).unwrap();
            
            let candidates: Vec<String> = paths
                .flatten()
                .filter(|path| {
                    path.file_name().into_string().unwrap_or_default().starts_with(post)
                })
                .map(|path| {
                    let name = path.file_name().into_string().unwrap_or_default();
                    if pre == "./" {
                        if path.path().is_dir() {
                            format!("{}/", name)
                        } else {
                            format!("{} ", name)
                        }
                    } else {
                        if path.path().is_dir() {
                            format!("{}/{}/", pre, name)
                        } else {
                            format!("{}/{} ", pre, name)
                        }
                    }
                })
                .collect();
            if candidates.len() > 1 {
                print!("\x07");
            }
            
            Ok((start, candidates))
        }
    }
}

impl Hinter for ShellCompleter {
    type Hint = String;
}
impl Highlighter for ShellCompleter {}
impl Validator for ShellCompleter {}
impl Helper for ShellCompleter {}
