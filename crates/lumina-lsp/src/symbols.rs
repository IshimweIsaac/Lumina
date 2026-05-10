use lumina_parser::ast::{Field, Program, Statement};
use tower_lsp::lsp_types::*;

pub fn get_document_symbols(prog: &Program) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for stmt in &prog.statements {
        match stmt {
            Statement::Entity(e) => {
                let range = span_to_range(&e.span);

                #[allow(deprecated)]
                let mut symbol = DocumentSymbol {
                    name: e.name.clone(),
                    detail: Some("Entity".to_string()),
                    kind: SymbolKind::CLASS,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: Some(Vec::new()),
                };

                let mut children = Vec::new();
                for field in &e.fields {
                    let (name, detail, span) = match field {
                        Field::Stored(s) => {
                            (s.name.clone(), format!("Stored {:?}", s.ty), s.span.clone())
                        }
                        Field::Derived(d) => {
                            (d.name.clone(), "Derived".to_string(), d.span.clone())
                        }
                        Field::Ref(r) => (
                            r.name.clone(),
                            format!("Ref {}", r.target_entity),
                            r.span.clone(),
                        ),
                    };
                    let field_range = span_to_range(&span);
                    #[allow(deprecated)]
                    children.push(DocumentSymbol {
                        name,
                        detail: Some(detail),
                        kind: SymbolKind::FIELD,
                        tags: None,
                        deprecated: None,
                        range: field_range,
                        selection_range: field_range,
                        children: None,
                    });
                }

                symbol.children = Some(children);
                symbols.push(symbol);
            }
            Statement::Rule(r) => {
                let range = span_to_range(&r.span);
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: r.name.clone(),
                    detail: Some("Rule".to_string()),
                    kind: SymbolKind::EVENT,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                });
            }
            Statement::Provider(p) => {
                let range = span_to_range(&p.span);
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: p.protocol.clone(),
                    detail: Some("Provider".to_string()),
                    kind: SymbolKind::INTERFACE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                });
            }
            Statement::Cluster(c) => {
                let range = span_to_range(&c.span);
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: c.node_id.clone(),
                    detail: Some("Cluster Configuration".to_string()),
                    kind: SymbolKind::MODULE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                });
            }
            Statement::Let(l) => {
                let range = span_to_range(&l.span);
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: l.name.clone(),
                    detail: Some("Let Binding".to_string()),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                });
            }
            _ => {}
        }
    }

    symbols
}

fn span_to_range(span: &lumina_lexer::token::Span) -> Range {
    Range {
        start: Position {
            line: span.line as u32,
            character: span.col as u32,
        },
        end: Position {
            line: span.line as u32, // approximation using start pos
            character: span.col as u32,
        },
    }
}
