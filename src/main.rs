#[allow(unused_imports)]
use std::io::{self, Write};

fn take_input(input: &mut String) {
    input.clear();
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
}

fn main() {
    let mut input = String::new();
    loop {
        take_input(&mut input);
        
        if input.trim() == "exit" {
            break;
        }
        
        println!("{}: command not found", input.trim());
    }
}
