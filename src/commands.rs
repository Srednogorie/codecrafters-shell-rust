use std::path::Path;
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

fn command_cd_set_current_dir(std_path: &Path, path: &str) {
    if std::fs::metadata(std_path).is_ok() {
        std::env::set_current_dir(path).unwrap();
    } else {
        eprintln!("cd: {}: No such file or directory", path);
    }
}

pub fn command_cd(path: String) {
    // On Unix, a path is absolute if it starts with the root, so is_absolute and has_root are equivalent.
    let std_path = Path::new(&path);
    if std_path.is_absolute() {
        command_cd_set_current_dir(std_path, &path);
    } else {
        if std_path.starts_with("~") {
            let home_dir = std::env::home_dir().unwrap();
            command_cd_set_current_dir(home_dir.as_path(), home_dir.to_str().unwrap());
        } else {
            let current_dir = std::env::current_dir().unwrap();
            let new_dir = current_dir.join(std_path);
            let new_dir = new_dir.as_path();
            command_cd_set_current_dir(new_dir, &path);
        }
    }
}
