use rustyline::history::{FileHistory, History};

use crate::enums::Commands;
use crate::utils::*;
use std::io::{self, Write};
use std::path::Path;

pub fn command_echo(args: &[String], stdout_writer: &mut dyn Write) -> Result<(), std::io::Error> {
    writeln!(stdout_writer, "{}", args.join(" "))?;
    Ok(())
}

pub fn command_exit() -> Result<(), std::io::Error> {
    std::process::exit(0);
}

pub fn command_type(args: &[String], stdout_writer: &mut dyn Write) -> Result<(), std::io::Error> {
    let command = args.first().map(|s| s.as_str()).unwrap_or("");
    let command_enum = Commands::from_str(command, args);
    match command_enum {
        Ok(Some(cmd)) => writeln!(stdout_writer, "{} is a shell builtin", cmd)?,
        Ok(None) => match find_in_path(command) {
            Some(full_path) => {
                writeln!(stdout_writer, "{} is {}", command, full_path.display())?;
            }
            None => {
                eprintln!("{}: not found", command);
            }
        },
        Err(e) => eprintln!("{}", e),
    }
    Ok(())
}

pub fn command_pwd(stdout_writer: &mut dyn Write) -> Result<(), std::io::Error> {
    writeln!(stdout_writer, "{}", std::env::current_dir()?.display())?;
    Ok(())
}

fn command_cd_set_current_dir(
    std_path: &Path,
    path: &str,
    stderr_writer: &mut dyn Write,
) -> Result<(), std::io::Error> {
    if std::fs::metadata(std_path).is_ok() {
        std::env::set_current_dir(path)?;
    } else {
        writeln!(stderr_writer, "cd: {}: No such file or directory", path)?;
    }
    Ok(())
}

pub fn command_cd(path: String, stderr_writer: &mut dyn Write) -> Result<(), std::io::Error> {
    // On Unix, a path is absolute if it starts with the root, so is_absolute and has_root are equivalent.
    let std_path = Path::new(&path);
    if std_path.is_absolute() {
        command_cd_set_current_dir(std_path, &path, stderr_writer)?;
    } else {
        if std_path.starts_with("~") {
            let home_dir =
                dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;
            let home_str = home_dir
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "home directory path is not valid UTF-8"))?;
            command_cd_set_current_dir(home_dir.as_path(), home_str, stderr_writer)?;
        } else {
            let current_dir = std::env::current_dir()?;
            let new_dir = current_dir.join(std_path);
            let new_dir = new_dir.as_path();
            command_cd_set_current_dir(new_dir, &path, stderr_writer)?;
        }
    }
    Ok(())
}

pub fn command_history(
    args: &[String],
    history: &FileHistory,
    stdout_writer: &mut dyn Write,
) -> Result<(), std::io::Error> {
    let iter_count = history.len();
    let max_entries = args.first().and_then(|a| a.parse::<usize>().ok()).unwrap_or(iter_count);
    for (i, entry) in history.iter().enumerate().skip(iter_count - max_entries) {
        writeln!(stdout_writer, "    {}  {}", i + 1, entry)?;
    }
    Ok(())
}
