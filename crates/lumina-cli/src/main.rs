use std::fs;
use std::collections::HashMap;
mod repl;
mod commands;

use lumina_parser::parse;
use lumina_parser::ast::*;
use lumina_analyzer::analyze;
use lumina_runtime::engine::Evaluator;
use lumina_runtime::adapters::static_adapter::StaticAdapter;
#[cfg(not(target_arch = "wasm32"))]
use lumina_runtime::adapters::{mqtt_adapter::MqttAdapter, http_adapter::HttpPollAdapter, file_adapter::FileWatchAdapter};
use lumina_diagnostics::DiagnosticRenderer;

mod loader;
use crate::loader::ModuleLoader;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("run")   => cmd_run(&args),
        Some("check") => cmd_check(&args),
        Some("repl")  => cmd_repl(),
        Some("setup") => cmd_setup(),
        _ => {
            eprintln!("Lumina v1.7.0 — Declarative Reactive Language");
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  lumina run <file.lum>     Run a Lumina program");
            eprintln!("  lumina check <file.lum>   Type-check without running");
            eprintln!("  lumina repl               Start interactive REPL");
            eprintln!("  lumina setup              Automated IDE & environment setup");
            std::process::exit(1);
        }
    }
}

fn cmd_setup() {
    println!("Lumina v1.7 — Automated Environment Setup");
    println!("─────────────────────────────────────────");

    // 1. Detect VS Code
    println!("Checking for VS Code / VSCodium...");
    let id = "luminalang.lumina-lang";
    let mut installed = false;

    for cmd in &["code", "codium", "cursor"] {
        let status = std::process::Command::new(cmd)
            .arg("--install-extension")
            .arg(id)
            .arg("--force")
            .status();

        if let Ok(s) = status {
            if s.success() {
                println!("✓ Successfully installed Lumina extension for {cmd}!");
                installed = true;
            }
        }
    }

    if !installed {
        println!("! No supported IDE (VS Code, VSCodium, Cursor) detected in PATH.");
        println!("  You can manually install the extension from: https://marketplace.visualstudio.com/items?itemName={id}");
    }

    println!("\nSetup complete. Happy coding!");
}

fn read_file(args: &[String]) -> (String, String) {
    let path = args.get(2).unwrap_or_else(|| {
        eprintln!("Error: missing file argument");
        eprintln!("Usage: lumina run <file.lum>");
        std::process::exit(1);
    });
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{path}': {e}");
        std::process::exit(1);
    });
    (path.clone(), source)
}

fn build_evaluator(analyzed: &lumina_analyzer::AnalyzedProgram) -> Evaluator {
    let mut rules = Vec::new();
    let mut derived = HashMap::new();
    let mut adapters: Vec<Box<dyn lumina_runtime::adapter::LuminaAdapter>> = Vec::new();

    for stmt in &analyzed.program.statements {
        match stmt {
            Statement::Rule(r) => rules.push(r.clone()),
            Statement::Entity(e) => {
                for f in &e.fields {
                    if let Field::Derived(df) = f {
                        derived.insert((e.name.clone(), df.name.clone()), df.expr.clone());
                    }
                }
            }
            Statement::ExternalEntity(e) => {
                // Initialize adapters based on sync_path
                if e.sync_path.starts_with("static://") {
                    adapters.push(Box::new(StaticAdapter::new(&e.name)));
                } 
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if e.sync_path.starts_with("mqtt://") {
                        if let Ok(a) = MqttAdapter::new(&e.name, &e.sync_path, "lumina-cli", "lumina/in", "lumina/out") {
                            adapters.push(Box::new(a));
                        }
                    } else if e.sync_path.starts_with("http://") || e.sync_path.starts_with("https://") {
                        adapters.push(Box::new(HttpPollAdapter::new(&e.name, e.sync_path.clone(), e.poll_interval.as_ref().map(|d| d.to_std_duration()).unwrap_or(std::time::Duration::from_secs(5)))));
                    } else if e.sync_path.starts_with("file://") {
                        let path_str = e.sync_path.trim_start_matches("file://");
                        if let Ok(a) = FileWatchAdapter::new(&e.name, std::path::PathBuf::from(path_str)) {
                            adapters.push(Box::new(a));
                        }
                    }
                }

                for f in &e.fields {
                    if let Field::Derived(df) = f {
                        derived.insert((e.name.clone(), df.name.clone()), df.expr.clone());
                    }
                }
            }
            _ => {}
        }
    }
    let mut ev = Evaluator::new(analyzed.schema.clone(), analyzed.graph.clone(), rules);
    ev.derived_exprs = derived;
    ev.adapters = adapters;
    ev
}

fn cmd_run(args: &[String]) {
    let (path, source) = read_file(args);

    let program = match ModuleLoader::load(Path::new(&path)) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    let analyzed = analyze(program, &source, &path, true).unwrap_or_else(|errors| {
        eprintln!("{}", DiagnosticRenderer::render_all(&errors));
        std::process::exit(1);
    });

    let mut evaluator = build_evaluator(&analyzed);

    for stmt in &analyzed.program.statements {
        if let Err(e) = evaluator.exec_statement(stmt) {
            eprintln!("Runtime error [{}]: {}", e.code(), e.message());
            std::process::exit(1);
        }
    }

    // Initial state calculation
    if let Err(e) = evaluator.recalculate_all_rules() {
        eprintln!("Initialization error: {}", e.message());
        std::process::exit(1);
    }

    if evaluator.timers.for_timers.is_empty() 
        && evaluator.timers.every_timers.is_empty() 
        && evaluator.adapters.is_empty() 
    {
        return;
    }

    println!("Running Lumina [Ctrl+C to stop]...");
    loop {
        match evaluator.tick() {
            Ok(events) => {
                for event in events {
                    let color = match event.severity.as_str() {
                        "info"     => "\x1b[36m", // Cyan
                        "warning"  => "\x1b[33m", // Yellow
                        "critical" => "\x1b[31m", // Red
                        "resolved" => "\x1b[32m", // Green
                        _          => "\x1b[0m",  // Reset
                    };
                    println!("{}[{}] {}: {}\x1b[0m", color, event.severity.to_uppercase(), event.rule, event.message);
                }
            }
            Err(rollback) => {
                eprintln!("\x1b[31mRuntime error: {}\x1b[0m", rollback.diagnostic.message);
                eprintln!("Suggested fix: {}", rollback.diagnostic.suggested_fix);
                std::process::exit(1);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn cmd_check(args: &[String]) {
    let (path, source) = read_file(args);

    let program = match ModuleLoader::load(Path::new(&path)) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    match analyze(program, &source, &path, true) {
        Ok(_) => {
            let basename = std::path::Path::new(&path)
                .file_name().unwrap_or_default()
                .to_string_lossy();
            println!("✓ {} — no errors found", basename);
        }
        Err(errors) => {
            eprintln!("{}", DiagnosticRenderer::render_all(&errors));
            std::process::exit(1);
        }
    }
}

fn cmd_repl() {
    use crate::repl::{ReplSession, ReplResult};
    use crate::commands::run_command;
    use std::io::{self, BufRead, Write};

    println!("Lumina v1.7.0 REPL — type Lumina expressions and statements");
    println!("Type ':help' to see inspector commands\n");

    let mut session = ReplSession::new();
    let stdin = io::stdin();

    loop {
        // Show prompt based on brace depth
        let prompt = if session.brace_depth > 0 { "... " } else { "lumina> " };
        print!("{}", prompt);
        io::stdout().flush().ok();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 { break; }
        let line = line.trim_end_matches('\n').trim_end_matches('\r');

        // Inspector commands start with ":"
        if line.starts_with(':') {
            println!("{}", run_command(&mut session, line));
            continue;
        }

        // Skip blank lines
        if line.trim().is_empty() { continue; }

        match session.feed(line) {
            ReplResult::NeedMore => {} // show "..." next iteration
            ReplResult::Ok(out) => { if !out.is_empty() { println!("{}", out); } }
            ReplResult::Error(err) => { eprintln!("{}", err); }
        }
    }
}
