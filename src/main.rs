#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;


fn take_input(input: &mut String) {
    input.clear();
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
}

fn command_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn check_unknown_command(command: &str) {
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
                        println!("{} is {}", command, &path);
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

fn command_type(args: &[&str]) {
    let command = args.first().unwrap_or(&"");
    match *command {
        "echo" => println!("echo is a shell builtin"),
        "type" => println!("type is a shell builtin"),
        "exit" => println!("exit is a shell builtin"),
        _ => check_unknown_command(command),
    }
}

fn main() {
    let mut input = String::new();
    loop {
        take_input(&mut input);
        let parsed_input: Vec<&str> = input.split_whitespace().collect();
        let command = parsed_input.first().unwrap_or(&"");
        
        if command.is_empty() {
            continue;
        }

        let args = &parsed_input[1..];

        match *command {
            "echo" => command_echo(args),
            "type" => command_type(args),
            "exit" => break,
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
