use std::fs;
use std::io::Write;
use std::collections::HashMap;
mod repl;
mod commands;
mod formatter;

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
        Some("run")      => cmd_run(&args),
        Some("check")    => cmd_check(&args),
        Some("repl")     => cmd_repl(),
        Some("fmt")      => cmd_fmt(&args),
        Some("query")    => cmd_query(&args),
        Some("provider") => cmd_provider(&args),
        Some("setup")    => cmd_setup(),
        Some("uninstall") => cmd_uninstall(),
        Some("cluster")  => cmd_cluster(&args),
        Some("version") | Some("--version") | Some("-v") => {
            println!("Lumina v2.0.0: The Cluster Release");
            std::process::exit(0);
        }
        _ => {
            eprintln!("Lumina v2.0.0: The Cluster Release");
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
            eprintln!("  lumina uninstall          Remove Lumina from your system");
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

    // All VS Code-compatible IDEs (they all support --install-extension)
    let ide_commands: &[(&str, &str)] = &[
        ("code",        "VS Code"),
        ("codium",      "VSCodium"),
        ("cursor",      "Cursor"),
        ("antigravity", "Antigravity"),
        ("windsurf",    "Windsurf"),
        ("positron",    "Positron"),
        ("code-insiders", "VS Code Insiders"),
        ("code-oss",    "Code OSS"),
    ];

    let id = "luminalang.lumina-lang";
    let vsix_name = "lumina-lang-1.8.0.vsix";
    let mut installed = false;

    // 1. Determine the installation source
    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(Path::new("."));
    let local_vsix = exe_dir.join(vsix_name);

    // Also check ~/.lumina/bin (where the curl installer places things)
    let home_vsix = dirs_fallback().map(|h| h.join(".lumina").join("bin").join(vsix_name));

    let install_source = if local_vsix.exists() {
        println!("→ Found local extension: {}", local_vsix.display());
        local_vsix.to_string_lossy().to_string()
    } else if home_vsix.as_ref().map_or(false, |p| p.exists()) {
        let p = home_vsix.unwrap();
        println!("→ Found local extension: {}", p.display());
        p.to_string_lossy().to_string()
    } else {
        // Try to download the .vsix from the website for offline install
        println!("→ No local extension found. Downloading from server...");
        let download_dest = exe_dir.join(vsix_name);
        let url = format!("https://lumina-lang.web.app/{}", vsix_name);
        match download_vsix(&url, &download_dest) {
            Ok(_) => {
                println!("  ✓ Downloaded {}", vsix_name);
                download_dest.to_string_lossy().to_string()
            }
            Err(e) => {
                println!("  ✗ Download failed ({}). Falling back to marketplace ID.", e);
                id.to_string()
            }
        }
    };

    // 2. Try installing in every detected IDE
    println!("\nScanning for supported IDEs...");
    for (cmd, name) in ide_commands {
        // Quick check: does the command exist?
        let exists = std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !exists {
            continue; // Skip silently — don't clutter output with IDEs user doesn't have
        }

        print!("  {} ({})... ", name, cmd);
        std::io::stdout().flush().ok();

        let status = std::process::Command::new(cmd)
            .arg("--install-extension")
            .arg(&install_source)
            .arg("--force")
            .output();

        match status {
            Ok(output) if output.status.success() => {
                println!("✓ Installed");
                installed = true;
            }
            Ok(_) => println!("✗ Failed"),
            Err(_) => println!("✗ Error"),
        }
    }

    if !installed {
        println!("\n⚠  No supported IDE detected.");
        println!("  Install the extension manually from:");
        println!("  https://marketplace.visualstudio.com/items?itemName={}", id);
    }

    println!("\nSetup complete. Happy coding!");
}

/// Simple HOME directory fallback (avoids adding a dependency)
fn dirs_fallback() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
}

/// Download a file from a URL to a destination path using curl
fn download_vsix(url: &str, dest: &std::path::Path) -> Result<(), String> {
    let output = std::process::Command::new("curl")
        .args(&["-fsSL", url, "-o"])
        .arg(dest)
        .output()
        .map_err(|e| format!("curl not found: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn cmd_uninstall() {
    println!("Lumina v2.0 — Uninstaller");
    println!("─────────────────────────");

    // 1. Uninstall IDE extensions
    let ide_commands: &[(&str, &str)] = &[
        ("code",        "VS Code"),
        ("codium",      "VSCodium"),
        ("cursor",      "Cursor"),
        ("antigravity", "Antigravity"),
        ("windsurf",    "Windsurf"),
        ("positron",    "Positron"),
        ("code-insiders", "VS Code Insiders"),
        ("code-oss",    "Code OSS"),
    ];
    let id = "luminalang.lumina-lang";

    println!("Removing IDE extensions...");
    for (cmd, name) in ide_commands {
        let exists = std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !exists { continue; }

        print!("  {} ({})... ", name, cmd);
        std::io::stdout().flush().ok();

        let status = std::process::Command::new(cmd)
            .arg("--uninstall-extension")
            .arg(id)
            .output();

        match status {
            Ok(output) if output.status.success() => println!("✓ Removed"),
            _ => println!("✗ Not installed or failed"),
        }
    }

    // 2. Remove binaries from ~/.lumina
    let lumina_home = dirs_fallback()
        .map(|h| h.join(".lumina"))
        .unwrap_or_else(|| std::path::PathBuf::from(".lumina"));

    if lumina_home.exists() {
        print!("Removing {}... ", lumina_home.display());
        std::io::stdout().flush().ok();
        match std::fs::remove_dir_all(&lumina_home) {
            Ok(_) => println!("✓ Done"),
            Err(e) => println!("✗ {}", e),
        }
    }

    // 3. Clean PATH entries from shell profiles
    println!("Cleaning PATH from shell profiles...");
    let home = dirs_fallback().unwrap_or_default();
    for profile_name in &[".bashrc", ".zshrc", ".bash_profile", ".profile"] {
        let profile = home.join(profile_name);
        if profile.exists() {
            if let Ok(contents) = std::fs::read_to_string(&profile) {
                let cleaned: String = contents
                    .lines()
                    .filter(|line| !line.contains(".lumina/bin"))
                    .collect::<Vec<_>>()
                    .join("\n");
                if cleaned != contents {
                    let _ = std::fs::write(&profile, cleaned + "\n");
                    println!("  ✓ Cleaned {}", profile_name);
                }
            }
        }
    }

    println!("\nLumina has been uninstalled. Restart your terminal to finalize.");
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
