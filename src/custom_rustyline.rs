use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::{Helper, Result, Context};

pub struct ShellCompleter;

const BUILTINS: &[&str] = &["echo", "type", "exit", "pwd", "cd"];

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<String>)> {
        // Only look at the text up to the cursor
        let word = &line[..pos];

        // Find where the current word starts (after the last space, or 0)
        let start = word.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let prefix = &word[start..];

        let candidates: Vec<String> = BUILTINS
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
