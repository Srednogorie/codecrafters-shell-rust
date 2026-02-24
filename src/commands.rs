use crate::utils::*;

pub fn command_echo(args: &[String]) {
    println!("{}", args.join(" "));
}

pub fn command_exit() {
    std::process::exit(0);
}

pub fn command_type(args: &[String]) {
    let command = args.first().map(|s| s.as_str()).unwrap_or("");
    match command {
        "echo" => println!("echo is a shell builtin"),
        "type" => println!("type is a shell builtin"),
        "exit" => println!("exit is a shell builtin"),
        // We are calling from type meaning we don't wan to execute so we don't need to pass args either
        _ => check_unknown_command(command, vec![], false),
    }
}