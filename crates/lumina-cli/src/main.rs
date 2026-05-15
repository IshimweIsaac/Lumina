use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

mod commands;
mod formatter;
mod repl;


use lumina_analyzer::analyze;
use lumina_diagnostics::DiagnosticRenderer;
use lumina_parser::ast::*;
use lumina_parser::parse;
use lumina_runtime::engine::Evaluator;

#[cfg(all(not(target_arch = "wasm32"), not(windows)))]
use lumina_runtime::adapters::mqtt_adapter::MqttAdapter;
use lumina_runtime::adapters::static_adapter::StaticAdapter;
#[cfg(not(target_arch = "wasm32"))]
use lumina_runtime::adapters::{file_adapter::FileWatchAdapter, http_adapter::HttpPollAdapter};


mod loader;
use crate::loader::ModuleLoader;
use std::path::Path;

/// Canonical version string derived from Cargo.toml — single source of truth.
const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("run") => cmd_run(&args),
        Some("check") => cmd_check(&args),
        Some("repl") => cmd_repl(),
        Some("fmt") => cmd_fmt(&args),
        Some("query") => cmd_query(&args),
        Some("provider") => cmd_provider(&args),
        Some("setup") => cmd_setup(),
        Some("update") => cmd_update(&args),
        Some("uninstall") => cmd_uninstall(),
        Some("cluster") => cmd_cluster(&args),
        Some("get") => {
            if args.get(2) == Some(&"documentation".to_string()) {
                let content = include_str!("master_knowledge.md");
                match fs::write("master_knowledge.md", content) {
                    Ok(_) => println!("Successfully created 'master_knowledge.md' in the current directory. Your AI can now ingest it."),
                    Err(e) => {
                        eprintln!("Error creating documentation file: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            eprintln!("Unknown command: Lumina get {}", args.get(2).unwrap_or(&"".to_string()));
            std::process::exit(1);
        }

        Some("version") | Some("--version") | Some("-v") => {
            println!("Lumina {}: The Architect Release", VERSION);
            std::process::exit(0);
        }
        _ => {
            eprintln!("Lumina {}: The Architect Release", VERSION);
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  lumina run <file.lum>       Run a Lumina program (--trace for debug)");
            eprintln!("  lumina check <file.lum>     Type-check without running");
            eprintln!("  lumina get documentation    Output master documentation for AI agents");
            eprintln!("  lumina fmt <file.lum>       Format source code");
            eprintln!("  lumina query <expr>         Query the truth store");
            eprintln!("  lumina provider <cmd>       Manage providers");
            eprintln!("  lumina cluster <cmd>        Cluster management (start, status, join)");
            eprintln!("  lumina repl                 Start interactive REPL");
            eprintln!("  lumina setup                Automated IDE & environment setup");
            eprintln!("  lumina update               Update Lumina to the latest version");
            eprintln!("  lumina uninstall            Remove Lumina from your system");
            std::process::exit(1);
        }
    }
}


fn cmd_cluster(args: &[String]) {
    match args.get(2).map(|s| s.as_str()) {
        Some("start") => {
            let spec = args.get(3).unwrap_or_else(|| {
                eprintln!(
                    "Error: missing spec file. Usage: lumina cluster start <spec.lum> [node_id]"
                );
                std::process::exit(1);
            });
            let source = fs::read_to_string(spec).unwrap_or_else(|e| {
                eprintln!("Error reading spec '{}': {}", spec, e);
                std::process::exit(1);
            });
            let program = match parse(&source) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            };

            // Extract cluster declaration from the program
            let cluster_decl = program
                .statements
                .iter()
                .find_map(|s| {
                    if let Statement::Cluster(c) = s {
                        Some(c)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    eprintln!("Error: no 'cluster {{ }}' block found in {}", spec);
                    std::process::exit(1);
                });

            // Allow overriding node_id from command line
            let mut config = lumina_cluster::ClusterConfig::from_decl(cluster_decl);
            if let Some(override_id) = args.get(4) {
                config.node_id = override_id.clone();
            }

            println!("──────────────────────────────────────────────────");
            println!("  Lumina Cluster Node — Sovereignty {}  ", VERSION);
            println!("──────────────────────────────────────────────────");
            println!("  Node ID:     {}", config.node_id);
            println!("  Peers:       {:?}", config.peers);
            println!("  Quorum:      {}", config.quorum);
            println!(
                "  Election:    {:.0}ms timeout",
                config.election_timeout.as_millis()
            );
            println!("──────────────────────────────────────────────────");

            // Initialize the cluster node
            let mut node = lumina_cluster::ClusterNode::new(config);
            node.initialize();

            let status = node.status();
            println!("  Role:        {}", status.role);
            println!("  Term:        {}", status.term);
            println!("  State:       {}", status.state);
            if let Some(ref leader) = status.leader_id {
                println!("  Leader:      {}", leader);
            }
            println!("  Mesh Nodes:  {}", status.mesh_nodes);
            println!("──────────────────────────────────────────────────");
            println!("✓ Node '{}' is running. [Ctrl+C to stop]", status.node_id);
            loop {
                node.tick(std::time::Instant::now());
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        Some("status") => {
            println!("Lumina Sovereign Cluster Status ({})", VERSION);
            println!("──────────────────────────────────────────────────");

            // In a real deployment, this would query a running node via IPC.
            // For now, report the offline status with useful diagnostics.
            println!("  Quorum:        Offline (no running nodes detected)");
            println!("  Active Nodes:  0");
            println!("  Leader:        None");
            println!("──────────────────────────────────────────────────");
            println!("  To start a node:");
            println!("    lumina cluster start <spec.lum> [node_id]");
        }
        Some("join") => {
            let address = args.get(3).unwrap_or_else(|| {
                eprintln!("Usage: lumina cluster join <address>");
                std::process::exit(1);
            });
            println!("Joining cluster at {}...", address);
            println!("  Status: Cluster join requires a running node.");
            println!("  Start a node first: lumina cluster start <spec.lum> <node_id>");
        }
        _ => {
            eprintln!("Lumina Cluster Management ({})", VERSION);
            eprintln!("  start <spec> [node_id]   Start a cluster node");
            eprintln!("  status                   Show cluster status");
            eprintln!("  join <address>            Join an existing cluster");
        }
    }
}

fn cmd_setup() {
    // Silent header — this runs automatically during install

    // All VS Code-compatible IDEs (they all support --install-extension)
    let ide_commands: &[(&str, &str)] = &[
        ("code", "VS Code"),
        ("codium", "VSCodium"),
        ("cursor", "Cursor"),
        ("antigravity", "Antigravity"),
        ("windsurf", "Windsurf"),
        ("positron", "Positron"),
        ("code-insiders", "VS Code Insiders"),
        ("code-oss", "Code OSS"),
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
        let url = format!(
            "https://woijupkxzzakmkneyxwk.supabase.co/storage/v1/object/public/Lumina/{}",
            vsix_name
        );
        match download_vsix(&url, &download_dest) {
            Ok(_) => {
                println!("  ✓ Downloaded {}", vsix_name);
                download_dest.to_string_lossy().to_string()
            }
            Err(e) => {
                println!(
                    "  ✗ Download failed ({}). Falling back to marketplace ID.",
                    e
                );
                id.to_string()
            }
        }
    };

    // 2. Try installing in every detected IDE
    println!("\nScanning for supported IDEs...");
    for (cmd, name) in ide_commands {
        // Quick check: does the command exist?
        let exists = if cfg!(windows) {
            std::process::Command::new("where")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        } else {
            std::process::Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };

        if !exists {
            continue; // Skip silently — don't clutter output with IDEs user doesn't have
        }

        print!("  {} ({})... ", name, cmd);
        std::io::stdout().flush().ok();

        let mut command = if cfg!(windows) {
            let mut c = std::process::Command::new("cmd");
            c.args(&["/C", cmd]);
            c
        } else {
            std::process::Command::new(cmd)
        };

        let status = command
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

    // 3. Neovim Zero-Config Setup
    if let Some(home) = dirs_fallback() {
        let nvim_dir = home.join(".config").join("nvim");
        if nvim_dir.exists() {
            print!("  Neovim (nvim)... ");
            std::io::stdout().flush().ok();

            let plugin_dir = nvim_dir.join("plugin");
            let _ = std::fs::create_dir_all(&plugin_dir);

            let lua_script = r#"-- Auto-generated by Lumina Installer
if vim.fn.executable('lumina-lsp') == 1 then
  vim.filetype.add({
    extension = { lum = 'lumina' }
  })
  vim.api.nvim_create_autocmd("FileType", {
    pattern = "lumina",
    callback = function()
      vim.lsp.start({
        name = 'lumina-lsp',
        cmd = {'lumina-lsp'},
        root_dir = vim.fn.getcwd(),
      })
    end,
  })
end"#;
            let script_path = plugin_dir.join("lumina.lua");
            match std::fs::write(&script_path, lua_script) {
                Ok(_) => {
                    println!("✓ Installed (Zero-Config)");
                    installed = true;
                }
                Err(_) => println!("✗ Failed to write plugin"),
            }
        }
    }

    if !installed {
        println!("\n⚠  No supported IDE detected.");
        println!("  Install the extension manually from:");
        println!(
            "  https://marketplace.visualstudio.com/items?itemName={}",
            id
        );
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
        ("code", "VS Code"),
        ("codium", "VSCodium"),
        ("cursor", "Cursor"),
        ("antigravity", "Antigravity"),
        ("windsurf", "Windsurf"),
        ("positron", "Positron"),
        ("code-insiders", "VS Code Insiders"),
        ("code-oss", "Code OSS"),
    ];
    let id = "luminalang.lumina-lang";

    println!("Removing IDE extensions...");
    for (cmd, name) in ide_commands {
        let exists = if cfg!(windows) {
            std::process::Command::new("where")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        } else {
            std::process::Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };
        if !exists {
            continue;
        }

        print!("  {} ({})... ", name, cmd);
        std::io::stdout().flush().ok();

        let mut command = if cfg!(windows) {
            let mut c = std::process::Command::new("cmd");
            c.args(&["/C", cmd]);
            c
        } else {
            std::process::Command::new(cmd)
        };

        let status = command.arg("--uninstall-extension").arg(id).output();

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
                    .filter(|line| {
                        !line.contains(".lumina/bin")
                            && !line.contains(".lumina/env")
                            && line.trim() != "# Lumina"
                    })
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

fn build_evaluator(
    analyzed: &lumina_analyzer::AnalyzedProgram,
    target_node_id: Option<&str>,
) -> Evaluator {
    // Use the centralized factory for the core evaluator setup
    let mut ev = lumina_runtime::factory::build_evaluator(analyzed);

    let mut cluster_decl = None;

    // CLI-specific: attach real network adapters and handle cluster setup
    for stmt in &analyzed.program.statements {
        match stmt {
            Statement::Cluster(c) => {
                if let Some(tid) = target_node_id {
                    if c.node_id == tid {
                        cluster_decl = Some(c.clone());
                    }
                } else if cluster_decl.is_none() {
                    cluster_decl = Some(c.clone());
                }
            }
            Statement::ExternalEntity(e) => {
                // Initialize real network adapters based on sync_path
                #[cfg(all(not(target_arch = "wasm32"), not(windows)))]
                {
                    if e.sync_path.starts_with("mqtt://") {
                        if let Ok(a) = MqttAdapter::new(
                            &e.name,
                            &e.sync_path,
                            "lumina-cli",
                            "lumina/in",
                            "lumina/out",
                        ) {
                            ev.adapters.push(Box::new(a));
                        }
                    }
                }
                #[cfg(all(not(target_arch = "wasm32"), windows))]
                {
                    if e.sync_path.starts_with("mqtt://") {
                        eprintln!(
                            "Warning: MQTT support is currently unavailable in this Windows build."
                        );
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if e.sync_path.starts_with("http://") || e.sync_path.starts_with("https://") {
                        ev.adapters.push(Box::new(HttpPollAdapter::new(
                            &e.name,
                            e.sync_path.clone(),
                            e.poll_interval
                                .as_ref()
                                .map(|d| d.to_std_duration())
                                .unwrap_or(std::time::Duration::from_secs(5)),
                        )));
                    } else if e.sync_path.starts_with("file://") {
                        let path_str = e.sync_path.trim_start_matches("file://");
                        if let Ok(a) =
                            FileWatchAdapter::new(&e.name, std::path::PathBuf::from(path_str))
                        {
                            ev.adapters.push(Box::new(a));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(decl) = cluster_decl {
        let config = lumina_cluster::ClusterConfig::from_decl(&decl);
        let mut node = lumina_cluster::ClusterNode::new(config);
        node.initialize();
        ev.cluster_node = Some(Arc::new(Mutex::new(node)));
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

    // Parse flags: --node-id <id>, --trace
    let mut node_id = None;
    let mut trace_mode = false;
    for i in 0..args.len() {
        if args[i] == "--node-id" && i + 1 < args.len() {
            node_id = Some(args[i + 1].as_str());
        }
        if args[i] == "--trace" {
            trace_mode = true;
        }
    }

    let mut evaluator = build_evaluator(&analyzed, node_id);
    evaluator.trace_mode = trace_mode;

    // Validate that all external entities have adapters
    let warnings = evaluator.validate_adapters();
    for warning in warnings {
        eprintln!("\x1b[33mWarning: {}\x1b[0m", warning);
    }

    evaluator.is_initializing = true;
    for stmt in &analyzed.program.statements {
        if let Err(e) = evaluator.exec_statement(stmt) {
            eprintln!("Runtime error [{}]: {}", e.code(), e.message());
            std::process::exit(1);
        }
    }
    evaluator.is_initializing = false;

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
                        "info" => "\x1b[36m",     // Cyan
                        "warning" => "\x1b[33m",  // Yellow
                        "critical" => "\x1b[31m", // Red
                        "resolved" => "\x1b[32m", // Green
                        _ => "\x1b[0m",           // Reset
                    };
                    println!(
                        "{}[{}] {}: {}\x1b[0m",
                        color,
                        event.severity.to_uppercase(),
                        event.rule,
                        event.message
                    );
                }
            }
            Err(rollback) => {
                eprintln!(
                    "\x1b[31mRuntime error: {}\x1b[0m",
                    rollback.diagnostic.message
                );
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
                .file_name()
                .unwrap_or_default()
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
    use crate::commands::run_command;
    use crate::repl::{ReplResult, ReplSession};
    use std::io::{self, BufRead, Write};

    println!("Lumina {} REPL — type Lumina expressions and statements", VERSION);
    println!("Type ':help' to see inspector commands\n");

    let mut session = ReplSession::new();
    let stdin = io::stdin();

    loop {
        // Show prompt based on brace depth
        let prompt = if session.brace_depth > 0 {
            "... "
        } else {
            "lumina> "
        };
        print!("{}", prompt);
        io::stdout().flush().ok();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim_end_matches('\n').trim_end_matches('\r');

        // Inspector commands start with ":"
        if line.starts_with(':') {
            println!("{}", run_command(&mut session, line));
            continue;
        }

        // Skip blank lines
        if line.trim().is_empty() {
            continue;
        }

        match session.feed(line) {
            ReplResult::NeedMore => {} // show "..." next iteration
            ReplResult::Ok(out) => {
                if !out.is_empty() {
                    println!("{}", out);
                }
            }
            ReplResult::Error(err) => {
                eprintln!("{}", err);
            }
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
        .file_name()
        .unwrap_or_default()
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

// ── Self-Update ────────────────────────────────────────────────────────────

fn cmd_update(args: &[String]) {
    let force = args.iter().any(|a| a == "--force");
    let check_only = args.iter().any(|a| a == "--check");

    println!("Lumina {} — Update", VERSION);
    println!("─────────────────────────");

    // 1. Query GitHub Releases API for the latest version
    print!("  Checking for updates... ");
    std::io::stdout().flush().ok();

    let api_url = "https://api.github.com/repos/IshimweIsaac/Lumina/releases/latest";
    let json = match curl_get_string(api_url) {
        Ok(s) => s,
        Err(e) => {
            println!("✗");
            eprintln!("  Failed to check for updates: {}", e);
            eprintln!("  Check your internet connection or try again later.");
            std::process::exit(1);
        }
    };

    let latest_tag = match parse_latest_tag(&json) {
        Some(t) => t,
        None => {
            println!("✗");
            eprintln!("  Could not determine the latest version from the GitHub API response.");
            eprintln!("  This may mean there are no published releases yet.");
            std::process::exit(1);
        }
    };

    // 2. Compare versions
    if latest_tag == VERSION && !force {
        println!("✓");
        println!("  Already up to date ({}).", VERSION);
        return;
    }

    if check_only {
        if latest_tag == VERSION {
            println!("✓");
            println!("  Already up to date ({}).", VERSION);
        } else {
            println!("→ {}", latest_tag);
            println!("  Update available: {} → {}", VERSION, latest_tag);
            println!("  Run 'lumina update' to install it.");
        }
        return;
    }

    if latest_tag != VERSION {
        println!("→ {}", latest_tag);
        println!("  Updating: {} → {}", VERSION, latest_tag);
    } else {
        println!("↻");
        println!("  Force-reinstalling {}...", VERSION);
    }

    // 3. Detect platform
    let platform = detect_platform_update();
    if platform.is_empty() {
        eprintln!("  Unsupported platform. Please download manually from:");
        eprintln!("  https://github.com/IshimweIsaac/Lumina/releases");
        std::process::exit(1);
    }

    // 4. Determine install directory
    let bin_dir = get_lumina_bin_dir();

    // 5. Download and replace binaries
    let base_url = "https://woijupkxzzakmkneyxwk.supabase.co/storage/v1/object/public/Lumina";

    let bins: &[(&str, &str)] = if cfg!(windows) {
        &[
            (&Box::leak(format!("lumina-{}.exe", platform).into_boxed_str()) as &str, "lumina.exe"),
            (&Box::leak(format!("lumina-{}-lsp.exe", platform).into_boxed_str()) as &str, "lumina-lsp.exe"),
        ]
    } else {
        // Use a leaked string so the borrow lives long enough.
        // This is fine — cmd_update runs once then the process exits.
        let cli_name: &'static str = Box::leak(format!("lumina-{}", platform).into_boxed_str());
        let lsp_name: &'static str = Box::leak(format!("lumina-{}-lsp", platform).into_boxed_str());
        &[
            (cli_name, "lumina"),
            (lsp_name, "lumina-lsp"),
        ]
    };

    for (remote_name, local_name) in bins {
        let url = format!("{}/{}", base_url, remote_name);
        let dest = bin_dir.join(local_name);
        let tmp = bin_dir.join(format!(".{}.tmp", local_name));

        print!("  Downloading {}... ", local_name);
        std::io::stdout().flush().ok();

        // Download binary to temp file
        if let Err(e) = curl_download(&url, &tmp) {
            println!("✗");
            eprintln!("  Failed to download {}: {}", remote_name, e);
            let _ = fs::remove_file(&tmp);
            std::process::exit(1);
        }

        // Download and verify checksum
        let checksum_url = format!("{}/{}.sha256", base_url, remote_name);
        match curl_get_string(&checksum_url) {
            Ok(checksum_content) => {
                let expected = checksum_content.split_whitespace().next().unwrap_or("");
                if !expected.is_empty() {
                    match compute_sha256(&tmp) {
                        Ok(actual) if actual == expected => { /* checksum OK */ }
                        Ok(actual) => {
                            println!("✗");
                            eprintln!("  Checksum verification failed for {}", local_name);
                            eprintln!("    Expected: {}", expected);
                            eprintln!("    Actual:   {}", actual);
                            let _ = fs::remove_file(&tmp);
                            std::process::exit(1);
                        }
                        Err(e) => {
                            println!("✗");
                            eprintln!("  Could not compute checksum for {}: {}", local_name, e);
                            let _ = fs::remove_file(&tmp);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(_) => {
                // Checksum file not available — warn but continue
                print!("(no checksum) ");
            }
        }

        // Atomic replace: on Windows, rename current to .old first
        #[cfg(windows)]
        {
            let old = bin_dir.join(format!(".{}.old", local_name));
            let _ = fs::remove_file(&old); // clean up any previous .old
            if dest.exists() {
                let _ = fs::rename(&dest, &old);
            }
        }

        if let Err(e) = fs::rename(&tmp, &dest) {
            println!("✗");
            eprintln!("  Failed to install {}: {}", local_name, e);
            let _ = fs::remove_file(&tmp);
            std::process::exit(1);
        }

        // Unix: chmod +x
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
        }

        // macOS: remove quarantine
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("xattr")
                .args(&["-d", "com.apple.quarantine"])
                .arg(&dest)
                .output();
        }

        println!("✓");
    }

    // Clean up Windows .old files
    #[cfg(windows)]
    for (_, local_name) in bins {
        let old = bin_dir.join(format!(".{}.old", local_name));
        let _ = fs::remove_file(&old);
    }

    println!();
    println!("  ✓ Lumina updated to {}.", latest_tag);
    println!("  Run 'lumina --version' to verify.");
}

/// Detect the platform string matching the release binary naming convention.
fn detect_platform_update() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("linux", "x86_64") => "linux-x64".to_string(),
        ("linux", "aarch64") => "linux-arm64".to_string(),
        ("macos", "x86_64") => "macos-x64".to_string(),
        ("macos", "aarch64") => "macos-arm64".to_string(),
        ("windows", "x86_64") => "windows-x64".to_string(),
        _ => String::new(),
    }
}

/// Get the Lumina bin directory (~/.lumina/bin), falling back to the directory
/// containing the currently running executable.
fn get_lumina_bin_dir() -> std::path::PathBuf {
    // Prefer LUMINA_HOME if set
    if let Ok(home) = std::env::var("LUMINA_HOME") {
        let p = std::path::PathBuf::from(home).join("bin");
        if p.exists() {
            return p;
        }
    }

    // Standard location: ~/.lumina/bin
    if let Some(home) = dirs_fallback() {
        let p = home.join(".lumina").join("bin");
        if p.exists() {
            return p;
        }
    }

    // Fallback: directory of the current executable
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Run curl and return stdout as a String.
fn curl_get_string(url: &str) -> Result<String, String> {
    let output = std::process::Command::new("curl")
        .args(&["-fsSL", "--max-time", "15", "-H", "User-Agent: lumina-cli", url])
        .output()
        .map_err(|e| format!("curl not found: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| format!("invalid UTF-8 in response: {}", e))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Download a file via curl.
fn curl_download(url: &str, dest: &std::path::Path) -> Result<(), String> {
    let output = std::process::Command::new("curl")
        .args(&["-fsSL", "--max-time", "120", "-H", "User-Agent: lumina-cli"])
        .arg(url)
        .arg("-o")
        .arg(dest)
        .output()
        .map_err(|e| format!("curl not found: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Parse the "tag_name" field from a GitHub API JSON response.
fn parse_latest_tag(json: &str) -> Option<String> {
    // Use serde_json since it's already a dependency
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v.get("tag_name")?.as_str().map(|s| s.to_string())
}

/// Compute SHA256 hex digest of a file.
fn compute_sha256(path: &std::path::Path) -> Result<String, String> {
    // Use sha256sum / shasum command (same approach as install.sh)
    let output = if cfg!(target_os = "macos") {
        std::process::Command::new("shasum")
            .args(&["-a", "256"])
            .arg(path)
            .output()
    } else {
        std::process::Command::new("sha256sum")
            .arg(path)
            .output()
    };

    match output {
        Ok(o) if o.status.success() => {
            let out = String::from_utf8_lossy(&o.stdout);
            Ok(out.split_whitespace().next().unwrap_or("").to_string())
        }
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => Err(format!("could not run checksum command: {}", e)),
    }
}
