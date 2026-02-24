use std::os::unix::{fs::PermissionsExt, process::CommandExt};

pub fn check_unknown_command(command: &str, args: Vec<String>, execute: bool) {
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
                            std::process::Command::new(&path)
                            .arg0(command)
                            .args(args)
                            .status()  // TODO - start form here spawn versus status
                            .expect("Failed to execute command");
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
