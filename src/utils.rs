use std::{os::unix::{fs::PermissionsExt, process::CommandExt}, process::Stdio};
use crate::SpecialTokens;

pub fn check_unknown_command(
    command: &str,
    args: Vec<String>,
    execute: bool,
    special_token: Option<SpecialTokens>,
    special_token_arg: Option<&str>
) {
    let key = "PATH";
    let mut found = false;
    match std::env::var_os(key) {
        Some(paths) => {
            for path in std::env::split_paths(&paths) {
                let path = format!("{}/{}", path.to_str().unwrap(), command);
                if std::fs::metadata(&path).is_ok() {
                    // Check if the file is executable
                    if std::fs::metadata(&path).unwrap().permissions().mode() & 0o111 != 0 {
                        found = true;
                        if execute {
                            match special_token {
                                Some(SpecialTokens::StdOut | SpecialTokens::StdOutExtended) => {
                                    std::process::Command::new(&path)
                                        .arg0(command)
                                        .args(args)
                                        .stdout(
                                            Stdio::from(
                                                std::fs::File::create(
                                                    &special_token_arg.unwrap()
                                                ).expect("Failed to create file")
                                            )
                                        )
                                        .status()
                                        .expect("Failed to execute command");
                                }
                                Some(SpecialTokens::StdErr) => {
                                    std::process::Command::new(&path)
                                        .arg0(command)
                                        .args(args)
                                        .stderr(
                                            Stdio::from(
                                                std::fs::File::create(
                                                    &special_token_arg.unwrap()
                                                ).expect("Failed to create file")
                                            )
                                        )
                                        .status()
                                        .expect("Failed to execute command");
                                }
                                Some(SpecialTokens::StdAppend | SpecialTokens::StdAppendExtended) => {
                                    std::process::Command::new(&path)
                                        .arg0(command)
                                        .args(args)
                                        .stdout(
                                            std::fs::OpenOptions::new()
                                                .create(true)
                                                .append(true)
                                                .open(special_token_arg.unwrap())
                                                .unwrap()
                                        )
                                        .status()
                                        .expect("Failed to execute command");
                                }
                                Some(SpecialTokens::ErrAppend) => {
                                    std::process::Command::new(&path)
                                        .arg0(command)
                                        .args(args)
                                        .stderr(
                                            std::fs::OpenOptions::new()
                                                .create(true)
                                                .append(true)
                                                .open(special_token_arg.unwrap())
                                                .unwrap()
                                        )
                                        .status()
                                        .expect("Failed to execute command");
                                }
                                None => {
                                    std::process::Command::new(&path)
                                        .arg0(command)
                                        .args(args)
                                        .status()
                                        .expect("Failed to execute command");
                                }
                            }
                        } else {
                            println!("{} is {}", command, path);
                        }
                        break;
                    }
                }
            }
        }
        None => {}
    }
    if !found {
        println!("{}: not found", command);
    }
}
