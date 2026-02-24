use crate::{Commands, utils::*};

pub fn command_echo(args: &[String]) {
    println!("{}", args.join(" "));
}

pub fn command_exit() {
    std::process::exit(0);
}

pub fn command_type(args: &[String]) {
    let command = args.first().map(|s| s.as_str()).unwrap_or("");
    let command_enum = Commands::from_str(command, args);
    match command_enum {
        Some(cmd) => println!("{} is a shell builtin", cmd),
        None => check_unknown_command(command, vec![], false),
    }
}

pub fn command_pwd() {
    println!("{}", std::env::current_dir().unwrap().display());
}
