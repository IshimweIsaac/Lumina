pub mod ast {
    pub use lumina_parser::ast::*;
}
pub mod types;
pub mod graph;
pub mod analyzer;

pub use analyzer::{Analyzer, AnalyzerError, AnalyzedProgram};
use lumina_parser::ast::Program;
use lumina_diagnostics::{Diagnostic, SourceLocation, extract_line};

pub fn analyze(program: Program, source: &str, filename: &str, allow_imports: bool) -> Result<AnalyzedProgram, Vec<Diagnostic>> {
    let mut analyzer = Analyzer::new();
    analyzer.allow_imports = allow_imports;
    match analyzer.analyze(program) {
        Ok(analyzed) => Ok(analyzed),
        Err(raw_errors) => {
            let diags = raw_errors.into_iter().map(|e| {
                Diagnostic::new(
                    e.code.to_string(),
                    e.message.to_string(),
                    SourceLocation::from_span(e.span.line, e.span.col, e.span.end.saturating_sub(e.span.start).max(1), filename),
                    extract_line(source, e.span.line),
                    help_for_code(&e.code),
                )
            }).collect();
            Err(diags)
        }
    }
}

fn help_for_code(code: &str) -> Option<String> {
    match code {
        "L001" => Some("try choosing a unique name for this entity to resolve the naming conflict".into()),
        "L002" => Some("check spelling or ensure the entity is declared before it's used in this rule".into()),
        "L003" => Some("you can only manually 'update' stored fields, not derived ones (:=)".into()),
        "L004" => Some("avoid circular dependencies by making one of the fields in the chain a stored field (field: Type)".into()),
        "L005" => Some("ensure entity names are unique across your entire program".into()),
        "L006" => Some("ensure all field names within the same entity are unique".into()),
        "L010" => Some("double-check the field name spelling in your entity definition".into()),
        "L041" => Some("try shifting the time-dependent logic into a rule action instead of a := field".into()),
        "R004" => Some("verify your index value is within the bounds of the list".into()),
        _ => None,
    }
}
