use std::path::Path;
use crate::utils::*;
use crate::enums::{Commands};
use std::io::Write;

pub fn command_echo(args: &[String], stdout_writer: &mut dyn Write) {
    writeln!(stdout_writer, "{}", args.join(" ")).unwrap();
}

pub fn command_exit() {
    std::process::exit(0);
}

pub fn command_type(args: &[String], stdout_writer: &mut dyn Write) {
    let command = args.first().map(|s| s.as_str()).unwrap_or("");
    let command_enum = Commands::from_str(command, args);
    match command_enum {
        Some(cmd) => writeln!(stdout_writer, "{} is a shell builtin", cmd).unwrap(),
        None => check_unknown_command(command, vec![], false, None, None),
    }
}

pub fn command_pwd(stdout_writer: &mut dyn Write) {
    writeln!(stdout_writer, "{}", std::env::current_dir().unwrap().display()).unwrap();
}

fn command_cd_set_current_dir(std_path: &Path, path: &str, stderr_writer: &mut dyn Write) {
    if std::fs::metadata(std_path).is_ok() {
        std::env::set_current_dir(path).unwrap();
    } else {
        writeln!(stderr_writer, "cd: {}: No such file or directory", path).unwrap();
    }
}

pub fn command_cd(path: String, stderr_writer: &mut dyn Write) {
    // On Unix, a path is absolute if it starts with the root, so is_absolute and has_root are equivalent.
    let std_path = Path::new(&path);
    if std_path.is_absolute() {
        command_cd_set_current_dir(std_path, &path, stderr_writer);
    } else {
        if std_path.starts_with("~") {
            let home_dir = std::env::home_dir().unwrap();
            command_cd_set_current_dir(home_dir.as_path(), home_dir.to_str().unwrap(), stderr_writer);
        } else {
            let current_dir = std::env::current_dir().unwrap();
            let new_dir = current_dir.join(std_path);
            let new_dir = new_dir.as_path();
            command_cd_set_current_dir(new_dir, &path, stderr_writer);
        }
    }
}
