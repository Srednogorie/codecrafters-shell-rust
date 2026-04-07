use rustyline::history::{FileHistory, History};

use crate::enums::{Commands, HistoryArgs, HistoryFlags};
use crate::structs::BackgroundJob;
use crate::utils::*;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub fn command_echo(args: &[String], stdout_writer: &mut dyn Write) -> Result<(), std::io::Error> {
    writeln!(stdout_writer, "{}", args.join(" "))?;
    Ok(())
}

pub fn command_exit(history: &mut FileHistory) -> Result<(), std::io::Error> {
    if let Ok(var) = std::env::var("HISTFILE") {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&var)?;
        for line in history.iter() {
            writeln!(file, "{}", line)?;
        }
    }
    std::process::exit(0);
}

pub fn command_type(args: &[String], stdout_writer: &mut dyn Write) -> Result<(), std::io::Error> {
    let command = args.first().map(|s| s.as_str()).unwrap_or("");
    let command_enum = Commands::from_str(command, &args[1..]);
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
    args: &HistoryArgs,
    history: &mut FileHistory,
    stdout_writer: &mut dyn Write,
) -> Result<(), std::io::Error> {
    match args {
        HistoryArgs::Limit(max_entries) => {
            let iter_count = history.len();
            for (i, entry) in history.iter().enumerate().skip(iter_count - max_entries) {
                writeln!(stdout_writer, "    {}  {}", i + 1, entry)?;
            }
        }
        HistoryArgs::File(flag, file_path) => {
            match flag {
                HistoryFlags::Read => {
                    let _ = history.load(Path::new(file_path));
                    let existing_history: Vec<String> = history.iter().map(|s| s.to_string()).collect();
                    let _ = history.clear();
                    for entry in existing_history {
                        let _ = history.add(entry.as_str());
                    }
                }
                HistoryFlags::Write => {
                    let existing_history: Vec<String> = history.iter().map(|s| s.to_string()).collect();
                    let mut file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create_new(true)
                        .open(file_path)?;
                    for entry in existing_history {
                        writeln!(file, "{}", entry)?;
                    }
                }
                HistoryFlags::Append => {
                    let existing_history: Vec<String> = history.iter().map(|s| s.to_string()).collect();
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(file_path)?;
                    for entry in existing_history {
                        writeln!(file, "{}", entry)?;
                    }
                    let _ = history.clear();
                }
            }
        }
        HistoryArgs::PrintAll => {
            for (i, entry) in history.iter().enumerate() {
                writeln!(stdout_writer, "    {}  {}", i + 1, entry)?;
            }
        }
    }
    Ok(())
}

pub fn command_jobs(background_jobs: &mut Vec<BackgroundJob>) -> Result<(), std::io::Error> {
    let jobs_len = background_jobs.len();
    let mut i = 1;
    background_jobs.retain_mut(|job| {
        match job.child.try_wait() {
            Ok(None) => {
                if jobs_len == i {
                    println!("[{}]+  Running                 {} {} &", job.num, job.command, job.args.join(" "));
                } else if jobs_len - 1 == i {
                    println!("[{}]-  Running                 {} {} &", job.num, job.command, job.args.join(" "));
                } else {
                    println!("[{}]   Running                 {} {} &", job.num, job.command, job.args.join(" "));
                }
                i += 1;
                true
            }
            Ok(Some(_)) => {
                if jobs_len == i {
                    println!("[{}]+  Done                 {} {}", job.num, job.command, job.args.join(" "));
                } else if jobs_len - 1 == i {
                    println!("[{}]-  Done                 {} {}", job.num, job.command, job.args.join(" "));
                } else {
                    println!("[{}]   Done                 {} {}", job.num, job.command, job.args.join(" "));
                }
                i += 1;
                false
            }
            Err(_) => {
                false
            }
        }
    });
    Ok(())
}
