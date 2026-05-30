mod formatter;
mod parser;
mod rules;

use std::io::{self, Read, Write};
use std::path::PathBuf;
use formatter::Formatter;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let source = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| {
                eprintln!("Error reading {}: {}", path.display(), e);
                std::process::exit(1);
            })
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            });
        buf
    };

    let formatter = Formatter::new(4);
    let output = formatter.format(&source);

    io::stdout()
        .write_all(output.as_bytes())
        .unwrap_or_else(|e| {
            eprintln!("Error writing output: {}", e);
            std::process::exit(1);
        });
}
