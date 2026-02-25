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
    pub fn execute(&self) {
        match self {
            Commands::Echo(args) => command_echo(args),
            Commands::Type(args) => command_type(args),
            Commands::Pwd => command_pwd(),
            Commands::Exit => command_exit(),
            Commands::Cd(path) => command_cd(path.to_string()),
        }
    }
}