use crate::structs::PipelineStage;
use std::{
    ffi::OsString,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::{Child, Stdio},
};

const PERMISSIONS_EXECUTABLE: u32 = 0o111;
const PATH_KEY: &str = "PATH";

pub fn get_paths() -> Option<OsString> {
    std::env::var_os(PATH_KEY)
}

pub fn find_in_path(command: &str) -> Option<PathBuf> {
    let paths = get_paths()?;

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

pub fn execute_external(stage: &PipelineStage, stdin: Stdio, stdout: Stdio, stderr: Stdio) -> std::io::Result<Child> {
    let full_path = find_in_path(&stage.command)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("{}: not found", stage.command)))?;

    let mut cmd = std::process::Command::new(&full_path);
    cmd.arg0(&stage.command).args(&stage.args);
    cmd.stdin(stdin);

    if let Some(redirect) = &stage.redirect {
        let file = redirect.token.open_file(&redirect.target)?;
        if redirect.token.is_stdout_redirect() {
            cmd.stdout(Stdio::from(file));
            cmd.stderr(stderr);
        } else {
            cmd.stdout(stdout);
            cmd.stderr(Stdio::from(file));
        }
    } else {
        cmd.stdout(stdout);
        cmd.stderr(stderr);
    }

    cmd.spawn()
}
