use Part1_Interp::lexer;
use std::fs::File;
use std::io::prelude::*;

pub fn run_path(path: String) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    //DEBUG LINE
    println!("contents {}", contents);
    //END OF DEBUG LINE
    run(contents);
    Ok(())
}

// TODO! pub fn RunInteractive()
// TODO! Tackle the new method logic of Lexer. It should first scan then save to a lexer struct.

pub fn run(source: String) {
    let mut lexer = lexer::Lexer::new(source, Vec::new());
    let tokens = lexer.scan();

    println!("{:#?}", tokens);
}
