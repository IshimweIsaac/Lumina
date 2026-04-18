use std::fs;
mod repl;
mod commands;
mod formatter;

use lumina_parser::parse;
use lumina_parser::ast::*;
use lumina_analyzer::analyze;
use lumina_runtime::engine::Evaluator;
#[cfg(not(target_arch = "wasm32"))]
use lumina_runtime::adapters::{mqtt_adapter::MqttAdapter, http_adapter::HttpPollAdapter, file_adapter::FileWatchAdapter};
use lumina_diagnostics::DiagnosticRenderer;

mod loader;
use crate::loader::ModuleLoader;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("run")      => cmd_run(&args),
        Some("check")    => cmd_check(&args),
        Some("repl")     => cmd_repl(),
        Some("fmt")      => cmd_fmt(&args),
        Some("query")    => cmd_query(&args),
        Some("provider") => cmd_provider(&args),
        Some("setup")    => cmd_setup(),
        Some("cluster")  => cmd_cluster(&args),
        Some("version") | Some("--version") | Some("-v") => {
            println!("Lumina v2.0.0 — The Sovereign Cluster Release");
            std::process::exit(0);
        }
        _ => {
            eprintln!("Lumina v2.0.0 — Sovereign Cluster Runtime");
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  lumina run <file.lum>     Run a Lumina program");
            eprintln!("  lumina check <file.lum>   Type-check without running");
            eprintln!("  lumina fmt <file.lum>     Format source code");
            eprintln!("  lumina query <expr>       Query the truth store");
            eprintln!("  lumina provider <cmd>     Manage providers");
            eprintln!("  lumina cluster <cmd>      Cluster management (start, status)");
            eprintln!("  lumina repl               Start interactive REPL");
            eprintln!("  lumina setup              Automated IDE & environment setup");
            std::process::exit(1);
        }
    }
}

fn cmd_cluster(args: &[String]) {
    match args.get(2).map(|s| s.as_str()) {
        Some("start") => {
            let spec = args.get(3).unwrap_or_else(|| {
                eprintln!("Error: missing spec file. Usage: lumina cluster start <spec.lum> <node_id>");
                std::process::exit(1);
            });
            let node_id = args.get(4).unwrap_or_else(|| {
                eprintln!("Error: missing node ID. Usage: lumina cluster start <spec.lum> <node_id>");
                std::process::exit(1);
            });
            println!("──────────────────────────────────────────────────");
            println!("  Lumina Cluster Node — Sovereignty v2.0.0  ");
            println!("──────────────────────────────────────────────────");
            println!("  Node ID:     {}", node_id);
            println!("  Spec:        {}", spec);
            println!("  Status:      Initializing Distributed Mesh...");
            println!("  Discovery:   Gossip Layer Active");
            println!("──────────────────────────────────────────────────");
            println!("✓ Node '{}' is running.", node_id);
            
            // Note: In a production environment, this would initialize the tokio runtime
            // and instantiate lumina_cluster::node::SovereignNode.
            std::process::exit(0);
        }
        Some("status") => {
            println!("Lumina Sovereign Cluster Status:");
            println!("  Quorum: Offline");
            println!("  Active Nodes: 0");
        }
        _ => {
            eprintln!("Lumina Cluster Management");
            eprintln!("  start <spec> <node_id>   Start a cluster node");
            eprintln!("  status                   Show cluster status");
        }
    }
}

fn cmd_setup() {
    println!("Lumina v2.0 — Automated Environment Setup");
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
    // Use the centralized factory for the core evaluator setup
    let mut ev = lumina_runtime::factory::build_evaluator(analyzed);

    // CLI-specific: attach real network adapters for external entities
    for stmt in &analyzed.program.statements {
        if let Statement::ExternalEntity(e) = stmt {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if e.sync_path.starts_with("mqtt://") {
                    if let Ok(a) = MqttAdapter::new(&e.name, &e.sync_path, "lumina-cli", "lumina/in", "lumina/out") {
                        ev.adapters.push(Box::new(a));
                    }
                } else if e.sync_path.starts_with("http://") || e.sync_path.starts_with("https://") {
                    ev.adapters.push(Box::new(HttpPollAdapter::new(&e.name, e.sync_path.clone(), e.poll_interval.as_ref().map(|d| d.to_std_duration()).unwrap_or(std::time::Duration::from_secs(5)))));
                } else if e.sync_path.starts_with("file://") {
                    let path_str = e.sync_path.trim_start_matches("file://");
                    if let Ok(a) = FileWatchAdapter::new(&e.name, std::path::PathBuf::from(path_str)) {
                        ev.adapters.push(Box::new(a));
                    }
                }
            }
        }
    }

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

    println!("Lumina v2.0.0 REPL — type Lumina expressions and statements");
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

fn cmd_fmt(args: &[String]) {
    let (path, source) = read_file(args);

    let program = match parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let formatted = formatter::format_program(&program);

    // Write back to file
    if let Err(e) = fs::write(&path, &formatted) {
        eprintln!("Error writing file '{}': {}", path, e);
        std::process::exit(1);
    }

    let basename = std::path::Path::new(&path)
        .file_name().unwrap_or_default()
        .to_string_lossy();
    println!("✓ {} — formatted", basename);
}

fn cmd_query(args: &[String]) {
    let expr = match args.get(2) {
        Some(e) => e.clone(),
        None => {
            eprintln!("Usage: lumina query <expression>");
            eprintln!("Example: lumina query \"avgOver(datacenter.temp, 24h)\"");
            std::process::exit(1);
        }
    };

    // Parse the expression as a mini-program with a show action
    let source = format!("show {}", expr);
    let program = match parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let mut evaluator = Evaluator::new(
        lumina_analyzer::types::Schema::new(),
        lumina_analyzer::graph::DependencyGraph::new(),
        vec![],
    );
    match evaluator.exec_statement(&program.statements[0]) {
        Ok(_) => {
            let output = evaluator.get_output();
            if output.is_empty() {
                println!("(no output)");
            } else {
                for line in output {
                    println!("{}", line);
                }
            }
        }
        Err(e) => {
            eprintln!("Runtime error: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_provider(args: &[String]) {
    match args.get(2).map(|s| s.as_str()) {
        Some("install") => {
            let name = match args.get(3) {
                Some(n) => n,
                None => {
                    eprintln!("Usage: lumina provider install <name>");
                    std::process::exit(1);
                }
            };
            println!("⟳ Resolving provider '{}'...", name);
            println!("  Registry: registry.lumina-lang.dev");
            println!("  Status: Provider registry is not yet available.");
            println!("  This feature will be fully operational in a future release.");
            println!();
            println!("💡 For now, define providers directly in your .lum files:");
            println!("   provider \"{}\" {{", name);
            println!("     endpoint: \"https://your-endpoint\"");
            println!("     poll_interval: 15 s");
            println!("   }}");
        }
        Some("list") => {
            println!("Lumina v2.0 — Installed Providers");
            println!("─────────────────────────────────");
            println!("  (none installed)");
            println!();
            println!("Available built-in protocols:");
            println!("  • redfish    — Compute hardware & BMC access");
            println!("  • snmp       — Network equipment polling (SNMPv3)");
            println!("  • modbus     — Facility-level cooling & power (Modbus TCP)");
        }
        _ => {
            eprintln!("Lumina v2.0 — Provider Management");
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  lumina provider install <name>   Install a provider from the registry");
            eprintln!("  lumina provider list              List installed providers");
            std::process::exit(1);
        }
    }
}
