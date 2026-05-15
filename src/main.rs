mod cli;
mod orchestrator;

fn main() {
    let value = cli::command_parser();
    let path = value.unwrap();
    if let cli::Command::RunPath(s) = path {
        println!("value: {}", s);
        let _ = orchestrator::run_path(s);
    }
}
