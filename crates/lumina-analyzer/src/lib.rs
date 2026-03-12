pub mod ast {
    pub use lumina_parser::ast::*;
}
pub mod types;
pub mod graph;
pub mod analyzer;

pub use analyzer::{Analyzer, AnalyzerError, AnalyzedProgram};
use lumina_parser::ast::Program;

pub fn analyze(program: Program, allow_imports: bool) -> Result<AnalyzedProgram, Vec<AnalyzerError>> {
    let mut analyzer = Analyzer::new();
    analyzer.allow_imports = allow_imports;
    analyzer.analyze(program)
}
