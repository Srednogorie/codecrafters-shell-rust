use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::{Helper, Result, Context};
use std::fs;

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
        let prefix = &word[start..];
        
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

        let candidates: Vec<String> = builtins
            .iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .map(|s| format!("{} ", s))
            .collect();

        Ok((start, candidates))
    }
}

impl Hinter for ShellCompleter { type Hint = String; }
impl Highlighter for ShellCompleter {}
impl Validator for ShellCompleter {}
impl Helper for ShellCompleter {}
