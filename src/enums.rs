use rustyline::history::FileHistory;

use crate::commands::{command_cd, command_echo, command_exit, command_history, command_jobs, command_pwd, command_type};
use std::{fmt};
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub enum HistoryFlags {
    Read,
    Write,
    Append,
}
impl fmt::Display for HistoryFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            HistoryFlags::Read => "-r",
            HistoryFlags::Write => "-w",
            HistoryFlags::Append => "-a",
        };
        write!(f, "{}", name)
    }
}
impl HistoryFlags {
    pub fn from_str(flag: &str) -> Result<HistoryFlags, ShellError> {
        match flag {
            "-r" => Ok(HistoryFlags::Read),
            "-w" => Ok(HistoryFlags::Write),
            "-a" => Ok(HistoryFlags::Append),
            _ => Err(ShellError::InvalidFlag(flag.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum HistoryArgs {
    Limit(usize),
    File(HistoryFlags, String),
    PrintAll,
}

#[derive(Debug)]
pub enum Commands {
    Echo(Vec<String>),
    Type(Vec<String>),
    Exit,
    Pwd,
    Cd(String),
    History(HistoryArgs),
    Jobs,
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
            Commands::Jobs => "jobs",
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
            "history" => match args.first() {
                Some(value) => match HistoryFlags::from_str(value) {
                    Ok(flag) => Ok(Some(Commands::History(HistoryArgs::File(flag, args[1].clone())))),
                    _ => Ok(Some(Commands::History(HistoryArgs::Limit(
                        value.parse::<usize>().map_err(
                            |_| ShellError::InvalidArguments(format!("history: invalid argument: {}", value))
                        )?
                    )))),
                },
                None => Ok(Some(Commands::History(HistoryArgs::PrintAll))),
            },
            "jobs" => Ok(Some(Commands::Jobs)),
            _ => Ok(None),
        }
    }
    pub fn execute(
        &self,
        stdout_writer: &mut dyn Write,
        stderr_writer: &mut dyn Write,
        history: &mut FileHistory,
    ) -> Result<(), std::io::Error> {
        match self {
            Commands::Echo(args) => command_echo(args, stdout_writer),
            Commands::Type(args) => command_type(args, stdout_writer),
            Commands::Pwd => command_pwd(stdout_writer),
            // TODO: Refactor possibly here
            Commands::Exit => command_exit(history),
            Commands::Cd(path) => command_cd(path.to_string(), stderr_writer),
            Commands::History(args) => command_history(args, history, stdout_writer),
            Commands::Jobs => command_jobs(),
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
    InvalidFlag(String),
}
impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::IoError(err) => write!(f, "IO error: {}", err),
            ShellError::CommandNotFound(cmd) => write!(f, "{}: not found", cmd),
            ShellError::InvalidArguments(args) => write!(f, "Invalid arguments: {}", args),
            ShellError::InvalidFlag(flag) => write!(f, "Invalid flag: {}", flag),
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
