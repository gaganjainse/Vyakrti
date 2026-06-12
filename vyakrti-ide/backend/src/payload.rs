use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CompileRequest {
    pub source: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub sanskrit_message: Option<String>,
    pub severity: String, // "error", "warning", "info"
}

#[derive(Debug, Serialize)]
pub struct CompileResponse {
    pub ast: serde_json::Value,
    pub tokens: Vec<String>,
    pub bytecode: String,
    pub diagnostics: Vec<Diagnostic>,
    pub output: Vec<String>,
}