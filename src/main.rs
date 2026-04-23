use std::process;

use captionrs::run_cli as run;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
