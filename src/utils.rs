use crate::{enums::ShellError, structs::RedirectInfo};
use std::{
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::Stdio,
};

const PERMISSIONS_EXECUTABLE: u32 = 0o111;
const PATH_KEY: &str = "PATH";

fn find_command_in_path(command: &str) -> Option<PathBuf> {
    let paths = std::env::var_os(PATH_KEY)?;

    for path in std::env::split_paths(&paths) {
        let full = path.join(command);
        if let Ok(meta) = std::fs::metadata(&full) {
            if meta.permissions().mode() & PERMISSIONS_EXECUTABLE != 0 {
                return Some(full);
            }
        }
    }

    None
}

fn execute_external_command(
    full_path: &PathBuf, command: &str, args: Vec<String>, redirect_info: RedirectInfo
) -> std::io::Result<()> {
    if let Some(token) = redirect_info.special_token {
        let file = token.open_file(redirect_info.special_token_arg.unwrap())?;
        let mut cmd = std::process::Command::new(&full_path);
        cmd.arg0(command).args(args);
        if token.is_stdout_redirect() {
            cmd.stdout(Stdio::from(file));
        } else {
            cmd.stderr(Stdio::from(file));
        }
        cmd.status()?;
    } else {
        std::process::Command::new(&full_path).arg0(command).args(args).status()?;
    }
    Ok(())
}

pub fn check_unknown_command(
    command: &str, args: Vec<String>, execute: bool, redirect_info: RedirectInfo<'_>
) -> Result<(), ShellError> {
    match find_command_in_path(command) {
        Some(full_path) => {
            if execute {
                execute_external_command(&full_path, command, args, redirect_info)?;
            } else {
                println!("{} is {}", command, full_path.display());
            }
        }
        None => {
            return Err(ShellError::CommandNotFound(command.to_string()));
        }
    }
    Ok(())
}
