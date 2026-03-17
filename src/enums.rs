use crate::commands::{command_cd, command_echo, command_exit, command_pwd, command_type, command_history};
use crate::structs::History;
use std::fmt;
use std::fs::File;
use std::io::Write;

pub enum Commands {
    Echo(Vec<String>),
    Type(Vec<String>),
    Exit,
    Pwd,
    Cd(String),
    History(Vec<String>),
}
impl fmt::Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Commands::Echo(_) => "echo",
            Commands::Type(_) => "type",
            Commands::Exit => "exit",
            Commands::Pwd => "pwd",
            Commands::Cd(_) => "cd",
            Commands::History(_) => "history",
        };
        write!(f, "{}", name)
    }
}
impl Commands {
    pub fn from_str(command: &str, args: &[String]) -> Result<Option<Commands>, ShellError> {
        match command {
            "echo" => Ok(Some(Commands::Echo(args.to_vec()))),
            "type" => Ok(Some(Commands::Type(args.to_vec()))),
            "exit" => Ok(Some(Commands::Exit)),
            "pwd" => Ok(Some(Commands::Pwd)),
            "cd" => match args.first() {
                Some(a) => Ok(Some(Commands::Cd(a.clone()))),
                None => Err(ShellError::InvalidArguments("cd: missing argument".to_string())),
            },
            "history" => Ok(Some(Commands::History(args.to_vec()))),
            _ => Ok(None),
        }
    }
    pub fn execute(
        &self,
        stdout_writer: &mut dyn Write,
        stderr_writer: &mut dyn Write,
        history: &History,
    ) -> Result<(), std::io::Error> {
        match self {
            Commands::Echo(args) => command_echo(args, stdout_writer),
            Commands::Type(args) => command_type(args, stdout_writer),
            Commands::Pwd => command_pwd(stdout_writer),
            Commands::Exit => command_exit(),
            Commands::Cd(path) => command_cd(path.to_string(), stderr_writer),
            Commands::History(args) => command_history(args, &history, stdout_writer),
        }
    }
}

pub enum SpecialTokens {
    StdOut,
    StdOutExtended,
    StdErr,
    StdAppend,
    StdAppendExtended,
    ErrAppend,
    Pipe,
}
impl fmt::Display for SpecialTokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SpecialTokens::StdOut => ">",
            SpecialTokens::StdOutExtended => "1>",
            SpecialTokens::StdErr => "2>",
            SpecialTokens::StdAppend => ">>",
            Self::StdAppendExtended => "1>>",
            Self::ErrAppend => "2>>",
            SpecialTokens::Pipe => "|",
        };
        write!(f, "{}", name)
    }
}
impl SpecialTokens {
    pub fn is_stdout_redirect(&self) -> bool {
        matches!(
            self,
            SpecialTokens::StdOut
                | SpecialTokens::StdOutExtended
                | SpecialTokens::StdAppend
                | SpecialTokens::StdAppendExtended
        )
    }

    fn should_append(&self) -> bool {
        matches!(self, SpecialTokens::StdAppend | SpecialTokens::StdAppendExtended | SpecialTokens::ErrAppend)
    }

    pub fn open_file(&self, path: &str) -> std::io::Result<File> {
        if self.should_append() {
            std::fs::OpenOptions::new().create(true).append(true).open(path)
        } else {
            std::fs::File::create(path)
        }
    }
}

#[derive(Debug)]
pub enum ShellError {
    IoError(std::io::Error),
    CommandNotFound(String),
    InvalidArguments(String),
}
impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::IoError(err) => write!(f, "IO error: {}", err),
            ShellError::CommandNotFound(cmd) => write!(f, "{}: not found", cmd),
            ShellError::InvalidArguments(args) => write!(f, "Invalid arguments: {}", args),
        }
    }
}
impl std::error::Error for ShellError {}
// From<std::io::Error>` so that `?` on `io::Result` works automatically
impl From<std::io::Error> for ShellError {
    fn from(err: std::io::Error) -> Self {
        ShellError::IoError(err)
    }
}
