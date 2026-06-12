use crate::compiler::compile_source;
use crate::payload::Diagnostic;
use serde::{Deserialize, Serialize};
use vyakriti::ast::{ASTNode, Expression};
use vyakriti::lexer::Lexer;
use vyakriti::parser::Parser;

#[derive(Debug, Deserialize)]
pub struct SourceRequest {
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct HoverRequest {
    pub word: String,
}

#[derive(Debug, Deserialize)]
pub struct DefinitionRequest {
    pub source: String,
    pub symbol: String,
}

#[derive(Debug, Serialize)]
pub struct ParseResponse {
    pub ast: serde_json::Value,
    pub tokens: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Serialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct HoverResponse {
    pub contents: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SymbolItem {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct FormatResponse {
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct DefinitionResponse {
    pub line: usize,
    pub column: usize,
    pub found: bool,
}

const KEYWORDS: &[(&str, &str, &str)] = &[
    ("मान", "keyword", "variable declaration"),
    ("कार्य", "keyword", "function declaration"),
    ("प्रतिफल", "keyword", "return value"),
    ("यदि", "keyword", "conditional branch"),
    ("तर्हि", "keyword", "then branch"),
    ("अन्यथा", "keyword", "else branch"),
    ("यावत्", "keyword", "while loop"),
    ("तावत्", "keyword", "while body"),
    ("मुद्रण", "builtin", "print a value"),
    ("दैर्घ्यम्", "builtin", "string length"),
    ("प्रकारः", "builtin", "type name"),
    ("योजन", "builtin", "string concatenation"),
    ("अस्ति", "builtin", "string contains"),
    ("संख्या", "builtin", "parse integer"),
    ("रूपान्तर", "builtin", "convert value to string"),
    ("रूपभेदः", "keyword", "enum declaration"),
    ("समीक्षा", "keyword", "match expression"),
    ("वस्तु_विन्यासः", "keyword", "struct declaration"),
    ("गुणधर्म", "keyword", "trait declaration"),
    ("अनुष्ठान", "keyword", "implementation block"),
];

pub fn parse(source: &str) -> ParseResponse {
    let response = compile_source(source);
    ParseResponse {
        ast: response.ast,
        tokens: response.tokens,
        diagnostics: response.diagnostics,
    }
}

pub fn diagnostics(source: &str) -> Vec<Diagnostic> {
    compile_source(source).diagnostics
}

pub fn completions() -> Vec<CompletionItem> {
    KEYWORDS.iter().map(|(label, kind, detail)| CompletionItem {
        label: (*label).to_string(),
        kind: (*kind).to_string(),
        detail: (*detail).to_string(),
    }).collect()
}

pub fn hover(word: &str) -> HoverResponse {
    let contents = KEYWORDS.iter()
        .find(|(label, _, _)| *label == word)
        .map(|(label, kind, detail)| format!("{} ({}) — {}", label, kind, detail));
    HoverResponse { contents }
}

pub fn symbols(source: &str) -> Vec<SymbolItem> {
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer.tokenize());
    let Ok(ast) = parser.parse_program() else { return Vec::new(); };
    let mut output = Vec::new();
    for node in ast {
        collect_node_symbols(&node, &mut output);
    }
    output
}

fn collect_node_symbols(node: &ASTNode, output: &mut Vec<SymbolItem>) {
    match node {
        ASTNode::VarDecl { name, .. } => output.push(SymbolItem { name: name.clone(), kind: "variable".into() }),
        ASTNode::FuncDecl { name, .. } => output.push(SymbolItem { name: name.clone(), kind: "function".into() }),
        ASTNode::StructDecl { name, .. } => output.push(SymbolItem { name: name.clone(), kind: "struct".into() }),
        ASTNode::EnumDecl { name, variants, .. } => {
            output.push(SymbolItem { name: name.clone(), kind: "enum".into() });
            for (variant, _) in variants {
                output.push(SymbolItem { name: variant.clone(), kind: "variant".into() });
            }
        }
        ASTNode::TraitDecl { name, .. } => output.push(SymbolItem { name: name.clone(), kind: "trait".into() }),
        ASTNode::IfStmt { then_branch, else_branch, .. } => {
            for stmt in then_branch { collect_node_symbols(stmt, output); }
            if let Some(branch) = else_branch {
                for stmt in branch { collect_node_symbols(stmt, output); }
            }
        }
        ASTNode::WhileStmt { body, .. } | ASTNode::Block(body) => {
            for stmt in body { collect_node_symbols(stmt, output); }
        }
        ASTNode::StatementExpr(expr) | ASTNode::ReturnStmt(expr) => collect_expr_symbols(expr, output),
        _ => {}
    }
}

fn collect_expr_symbols(expr: &Expression, output: &mut Vec<SymbolItem>) {
    if let Expression::Call { name, .. } = expr {
        output.push(SymbolItem { name: name.clone(), kind: "call".into() });
    }
}

pub fn format_source(source: &str) -> FormatResponse {
    let lines = source.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    FormatResponse { source: lines }
}

pub fn definition(source: &str, symbol: &str) -> DefinitionResponse {
    for (line_idx, line) in source.lines().enumerate() {
        if line.contains(&format!("मान {}", symbol)) || line.contains(&format!("कार्य {}", symbol)) {
            let column = line.find(symbol).map(|c| c + 1).unwrap_or(1);
            return DefinitionResponse { line: line_idx + 1, column, found: true };
        }
    }
    DefinitionResponse { line: 1, column: 1, found: false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completions_include_canonical_devanagari_keywords() {
        let items = completions();
        assert!(items.iter().any(|item| item.label == "कार्य"));
        assert!(items.iter().any(|item| item.label == "मुद्रण"));
        assert!(!items.iter().any(|item| item.label == "function"));
    }

    #[test]
    fn symbols_reports_declarations() {
        let symbols = symbols("मान x : अङ्क = 1 ।\nकार्य योगः() -> अङ्क { प्रतिफल 1 । }");
        assert!(symbols.iter().any(|item| item.name == "x" && item.kind == "variable"));
        assert!(symbols.iter().any(|item| item.name == "योगः" && item.kind == "function"));
    }

    #[test]
    fn definition_finds_simple_declaration() {
        let location = definition("मान उत्तरम् : अङ्क = 42 ।", "उत्तरम्");
        assert!(location.found);
        assert_eq!(location.line, 1);
    }
}
