use std::io::Write;
use std::fmt;
use crate::commands::{command_echo, command_exit, command_type, command_pwd, command_cd};


pub enum Commands {
    Echo(Vec<String>),
    Type(Vec<String>),
    Exit,
    Pwd,
    Cd(String),
}
impl fmt::Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Commands::Echo(_) => "echo",
            Commands::Type(_) => "type",
            Commands::Exit => "exit",
            Commands::Pwd => "pwd",
            Commands::Cd(_) => "cd",
        };
        write!(f, "{}", name)
    }
}
impl Commands {
    pub fn from_str(command: &str, args: &[String]) -> Option<Commands> {
        match command {
            "echo" => Some(Commands::Echo(args.to_vec())),
            "type" => Some(Commands::Type(args.to_vec())),
            "exit" => Some(Commands::Exit),
            "pwd" => Some(Commands::Pwd),
            "cd" => Some(Commands::Cd(args[0].clone())),
            _ => None,
        }
    }
    pub fn execute(&self, writer: &mut dyn Write) {
        match self {
            Commands::Echo(args) => command_echo(args, writer),
            Commands::Type(args) => command_type(args, writer),
            Commands::Pwd => command_pwd(writer),
            Commands::Exit => command_exit(),
            Commands::Cd(path) => command_cd(path.to_string(), writer),
        }
    }
}

pub enum SpecialTokens {
    StdOut,
    StdOutExtended,
    // StdErr,
    // StdInOut,
    // StdAppend,
}
impl fmt::Display for SpecialTokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SpecialTokens::StdOut => ">",
            SpecialTokens::StdOutExtended => "1>",
            // SpecialTokens::StdErr => "2>",
            // SpecialTokens::StdInOut => "&>",
            // SpecialTokens::StdAppend => ">>",
        };
        write!(f, "{}", name)
    }
}
impl SpecialTokens {
    pub fn from_str(command: &str) -> Option<SpecialTokens> {
        match command {
            ">" => Some(SpecialTokens::StdOut),
            "1>" => Some(SpecialTokens::StdOutExtended),
            // "2>" => Some(SpecialTokens::StdErr),
            // "&>" => Some(SpecialTokens::StdInOut),
            // ">>" => Some(SpecialTokens::StdAppend),
            _ => None,
        }
    }
}