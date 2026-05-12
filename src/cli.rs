use std::env;

#[derive(Debug)]
pub enum Command {
    RunInteractive,
    RunPath(String),
}

//TODO Add later an enum and methods for each variant of error possible.
pub fn help() {
    println!("usage:
        Run file:
        RoX /path/to/file.rx

        Interactive mode:
        RoX
        ")
}

pub fn command_parser() -> Result<Command, fn()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("Entering Interactive Mode");
            Ok(Command::RunInteractive)
        },
        2 => {
            Ok(Command::RunPath(args[1].clone()))
        },
        _ => {
            Err(help)
        }
    }
}
