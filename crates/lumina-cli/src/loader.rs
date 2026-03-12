use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;

use lumina_parser::parse;
use lumina_parser::ast::{Program, Statement};
use lumina_parser::LuminaError;

pub struct ModuleLoader {
    /// Fully loaded and parsed programs, keyed by canonical path
    loaded: HashMap<PathBuf, Program>,
    /// Load order (topological - dependencies first)
    order: Vec<PathBuf>,
    /// Currently being loaded - used for cycle detection
    in_stack: HashSet<PathBuf>,
}

impl ModuleLoader {
    /// Entry point: load an entry .lum file and all its transitive imports.
    /// Returns a single merged Program ready for analysis and execution.
    pub fn load(entry: &Path) -> Result<Program, String> {
        let mut loader = Self {
            loaded: HashMap::new(),
            order: Vec::new(),
            in_stack: HashSet::new(),
        };

        let canonical = entry.canonicalize().map_err(|e| {
            file_not_found(entry, &e.to_string())
        })?;

        loader.load_recursive(&canonical)?;
        Ok(loader.merge())
    }

    fn load_recursive(&mut self, path: &PathBuf) -> Result<(), String> {
        // Already loaded - skip (DAG, not tree)
        if self.loaded.contains_key(path) { return Ok(()); }

        // Cycle detection
        if self.in_stack.contains(path) {
            return Err(circular_import(path));
        }

        self.in_stack.insert(path.clone());

        // Read and parse
        let source = fs::read_to_string(path).map_err(|e| {
            file_not_found(path, &e.to_string())
        })?;

        let program = parse(&source).map_err(|e| {
            parse_to_diagnostic(e, path)
        })?;

        // Recurse into imports before adding this file
        let dir = path.parent().unwrap_or(Path::new("."));
        for import in program.imports() {
            let dep_path = dir.join(&import.path);
            let dep_canonical = dep_path.canonicalize().map_err(|e| {
                file_not_found(&dep_path, &e.to_string())
            })?;
            self.load_recursive(&dep_canonical)?;
        }

        self.in_stack.remove(path);
        self.loaded.insert(path.clone(), program);
        self.order.push(path.clone());
        Ok(())
    }

    /// Merge all programs in topological order into one flat Program.
    /// Import statements are stripped from the merged output.
    fn merge(&self) -> Program {
        let mut stmts = Vec::new();
        for path in &self.order {
            if let Some(prog) = self.loaded.get(path) {
                for stmt in &prog.statements {
                    // Skip import statements - already resolved
                    if !matches!(stmt, Statement::Import(_)) {
                        stmts.push(stmt.clone());
                    }
                }
            }
        }
        Program {
            statements: stmts,
            span: lumina_lexer::token::Span::default(),
        }
    }
}

// Error constructors
fn file_not_found(path: &Path, reason: &str) -> String {
    format!("L017: file not found: {} - {}", path.display(), reason)
}

fn circular_import(path: &Path) -> String {
    format!("L016: circular import: {}", path.display())
}

fn parse_to_diagnostic(e: LuminaError, path: &Path) -> String {
    format!("P001: {} in {}", e, path.display())
}
