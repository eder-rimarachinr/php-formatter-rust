mod config;
mod formatter;
mod parser;
mod protocol;
mod rules;

use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Instant;

use config::Config;
use formatter::Formatter;
use protocol::{Diagnostic, DiagnosticFix, Request, Response};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--json") {
        run_json_mode();
    } else {
        run_legacy_mode(&args);
    }
}

// ---------------------------------------------------------------------------
// JSON mode  (php_formatter --json)
// ---------------------------------------------------------------------------

fn run_json_mode() {
    let mut buf = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut buf) {
        println!("{}", Response::error(format!("stdin read error: {e}")).to_json());
        return;
    }

    let request = match Request::parse(&buf) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", Response::error(e).to_json());
            return;
        }
    };

    let response = dispatch(request);
    println!("{}", response.to_json());
}

fn dispatch(req: Request) -> Response {
    let source = match req.source {
        Some(s) => s,
        None => return Response::error("missing 'source' field".into()),
    };

    let config = req
        .file_path
        .as_deref()
        .map(|p| Config::discover(Path::new(p)))
        .unwrap_or_default();

    match req.command.as_str() {
        "format" => cmd_format(source, config),
        "check"  => cmd_check(source, config),
        other    => Response::error(format!("unknown command: {other}")),
    }
}

fn cmd_format(source: String, config: Config) -> Response {
    let t0 = Instant::now();
    let formatted = Formatter::new(config).format(&source);
    let timing_ms = t0.elapsed().as_millis();
    let changed = formatted != source;

    Response {
        ok: true,
        command: "format".into(),
        formatted: Some(formatted),
        changed,
        timing_ms,
        diagnostics: vec![],
        error: None,
    }
}

fn cmd_check(source: String, config: Config) -> Response {
    let t0 = Instant::now();
    let formatted = Formatter::new(config).format(&source);
    let timing_ms = t0.elapsed().as_millis();

    let orig_lines: Vec<&str> = source.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();

    let diagnostics: Vec<Diagnostic> = orig_lines
        .iter()
        .zip(fmt_lines.iter())
        .enumerate()
        .filter(|(_, (o, f))| o != f)
        .map(|(i, (orig, fmt))| Diagnostic {
            line: i,
            col: 0,
            end_line: i,
            end_col: orig.len(),
            message: "Line would be reformatted".into(),
            severity: "warning".into(),
            code: "FMT001".into(),
            fix: Some(DiagnosticFix { replacement: fmt.to_string() }),
        })
        .collect();

    Response {
        ok: true,
        command: "check".into(),
        formatted: None,
        changed: !diagnostics.is_empty(),
        timing_ms,
        diagnostics,
        error: None,
    }
}

// ---------------------------------------------------------------------------
// Legacy mode  (php_formatter [file])
// ---------------------------------------------------------------------------

fn run_legacy_mode(args: &[String]) {
    let (source, config) = if args.len() > 1 {
        let path = std::path::PathBuf::from(&args[1]);
        let cfg = Config::discover(&path);
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", path.display());
            std::process::exit(1);
        });
        (src, cfg)
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {e}");
            std::process::exit(1);
        });
        (buf, Config::default())
    };

    let output = Formatter::new(config).format(&source);
    io::stdout().write_all(output.as_bytes()).unwrap_or_else(|e| {
        eprintln!("Error writing output: {e}");
        std::process::exit(1);
    });
}
