use super::repl::ReplSession;
use std::fs;

pub fn run_command(session: &mut ReplSession, cmd: &str) -> String {
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    match parts[0] {
        ":state" => state_cmd(session),
        ":schema" => schema_cmd(session),
        ":clear" => clear_cmd(session),
        ":help" => help_cmd(),
        ":load" => {
            let path = parts.get(1).unwrap_or(&"").trim();
            load_cmd(session, path)
        }
        ":save" => {
            let path = parts.get(1).unwrap_or(&"").trim();
            save_cmd(session, path)
        }
        ":quit" | ":q" => std::process::exit(0),
        other => format!("Unknown command: {}. Type :help for commands.", other),
    }
}

fn state_cmd(s: &mut ReplSession) -> String {
    let state = s.evaluator.export_state();
    serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".into())
}

fn schema_cmd(s: &mut ReplSession) -> String {
    // Print entity names and field types from the evaluator's entity registry
    s.evaluator.describe_schema()
}

fn clear_cmd(s: &mut ReplSession) -> String {
    s.clear();
    "Session cleared.".into()
}

fn load_cmd(s: &mut ReplSession, path: &str) -> String {
    if path.is_empty() { return "Usage: :load <file.lum>".into(); }
    match fs::read_to_string(path) {
        Err(e) => format!("Cannot read {}: {}", path, e),
        Ok(src) => match s.feed(&src) {
            super::repl::ReplResult::Ok(_) => format!("Loaded {}", path),
            super::repl::ReplResult::Error(e) => e,
            super::repl::ReplResult::NeedMore => "Incomplete construct in file.".into(),
        }
    }
}

fn save_cmd(s: &ReplSession, path: &str) -> String {
    if path.is_empty() { return "Usage: :save <file.lum>".into(); }
    match fs::write(path, &s.full_history) {
        Ok(()) => format!("Saved session to {}", path),
        Err(e) => format!("Cannot write {}: {}", path, e),
    }
}

fn help_cmd() -> String {
    ":state - print current state as JSON\n\
    :schema - list declared entities and fields\n\
    :load <file> - execute a .lum file into this session\n\
    :save <file> - save session source to file\n\
    :clear - reset the session\n\
    :help - show this list\n\
    :quit - exit the REPL".into()
}
