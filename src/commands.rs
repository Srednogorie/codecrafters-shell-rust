use crate::utils::*;
use crate::enums::Commands;

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

pub fn command_cd(path: String) {
    if path.starts_with("/") {
        if std::fs::metadata(path.clone()).is_ok() {
            std::env::set_current_dir(&path).unwrap();
        } else {
            eprintln!("cd: {}: No such file or directory", path);
        }
    } else {
        let current_dir = std::env::current_dir().unwrap();
        let new_dir = current_dir.join(&path);
        if let Err(e) = std::env::set_current_dir(&new_dir) {
            eprintln!("cd: {}: {}", path, e);
        }
    }
}
