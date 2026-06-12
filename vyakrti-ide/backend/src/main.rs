mod compiler;
mod lsp;
mod payload;
mod workspace;
mod ws;

use axum::{
    routing::{get, post},
    Router, Json,
    http::{Method, header},
    extract::ws::WebSocketUpgrade,
};
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};

async fn compile_handler(Json(payload): Json<payload::CompileRequest>) -> Json<payload::CompileResponse> {
    let response = compiler::compile_source(&payload.source);
    Json(response)
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl axum::response::IntoResponse {
    ws.on_upgrade(ws::handle_socket)
}

async fn health_handler() -> &'static str {
    "व्याकृति-पृष्ठभागः सज्जः"
}

async fn workspace_list_handler() -> Json<serde_json::Value> {
    match workspace::list_files().await {
        Ok(files) => Json(serde_json::json!({ "files": files })),
        Err(error) => Json(serde_json::json!({ "error": error })),
    }
}

async fn workspace_read_handler(Json(payload): Json<workspace::FilePathRequest>) -> Json<serde_json::Value> {
    match workspace::read_file(&payload.path).await {
        Ok(file) => Json(serde_json::json!(file)),
        Err(error) => Json(serde_json::json!({ "error": error })),
    }
}

async fn workspace_write_handler(Json(payload): Json<workspace::WriteFileRequest>) -> Json<serde_json::Value> {
    match workspace::write_file(&payload.path, &payload.content).await {
        Ok(result) => Json(serde_json::json!(result)),
        Err(error) => Json(serde_json::json!({ "error": error })),
    }
}

async fn workspace_search_handler(Json(payload): Json<workspace::SearchRequest>) -> Json<serde_json::Value> {
    match workspace::search(&payload.query).await {
        Ok(matches) => Json(serde_json::json!({ "matches": matches })),
        Err(error) => Json(serde_json::json!({ "error": error })),
    }
}

async fn lsp_parse_handler(Json(payload): Json<lsp::SourceRequest>) -> Json<lsp::ParseResponse> {
    Json(lsp::parse(&payload.source))
}

async fn lsp_diagnostics_handler(Json(payload): Json<lsp::SourceRequest>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "diagnostics": lsp::diagnostics(&payload.source) }))
}

async fn lsp_completions_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "items": lsp::completions() }))
}

async fn lsp_hover_handler(Json(payload): Json<lsp::HoverRequest>) -> Json<lsp::HoverResponse> {
    Json(lsp::hover(&payload.word))
}

async fn lsp_symbols_handler(Json(payload): Json<lsp::SourceRequest>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "symbols": lsp::symbols(&payload.source) }))
}

async fn lsp_format_handler(Json(payload): Json<lsp::SourceRequest>) -> Json<lsp::FormatResponse> {
    Json(lsp::format_source(&payload.source))
}

async fn lsp_definition_handler(Json(payload): Json<lsp::DefinitionRequest>) -> Json<lsp::DefinitionResponse> {
    Json(lsp::definition(&payload.source, &payload.symbol))
}

#[derive(Deserialize)]
pub struct ExplainRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct ExplainResponse {
    pub explanation: String,
    pub sanskrit_hint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

async fn explain_handler(Json(payload): Json<ExplainRequest>) -> Json<ExplainResponse> {
    let msg = payload.message.to_lowercase();

    let (explanation, sanskrit_hint, suggestion) = if msg.contains("borrow check") {
        (
            "The borrow checker found a violation of ownership rules. Each value in Vyākṛti has a single owner, and you cannot read or write a value that has been moved or mutably borrowed elsewhere.".to_string(),
            "स्वामित्व-नियमः भङ्गः — एकस्य मूल्यस्य एकः स्वामी भवति।".to_string(),
            Some("Use `ref` (सन्दर्भः) to borrow instead of moving. Ensure no mutable borrow is active when reading.".to_string()),
        )
    } else if msg.contains("exhaustiveness") || msg.contains("match") || msg.contains("समीक्षा") {
        (
            "A match expression does not cover all variants of the enum. Vyākṛti requires exhaustive pattern matching on enums.".to_string(),
            "समीक्षा-अपूर्णता — सर्वे रूपभेदाः आच्छादिताः न सन्ति।".to_string(),
            Some("Add missing arms for all enum variants, or add a wildcard `_` pattern.".to_string()),
        )
    } else if msg.contains("parse error") || msg.contains("।") {
        (
            "The parser expected a statement terminator '।' (danda) to mark the end of every statement.".to_string(),
            "वाक्य-समाप्तिः — प्रत्येकं वाक्यम् '।' इत्यनेन समाप्यते।".to_string(),
            Some("Add `।` (danda) at the end of the statement.".to_string()),
        )
    } else if msg.contains("undefined") || msg.contains("not found") || msg.contains("link") {
        (
            "A referenced function or variable could not be resolved. The name may be misspelled or the symbol was not defined in the current scope.".to_string(),
            "अपरिचितः नाम — नाम दोषयुक्तं वा अनुपस्थितं वा।".to_string(),
            Some("Check spelling and ensure the identifier is defined before use.".to_string()),
        )
    } else if msg.contains("stack underflow") || msg.contains("vm error") {
        (
            "The VM encountered a runtime error, usually from operating on the wrong type of value or stack corruption.".to_string(),
            "रनटाइम-दोषः — मूल्य-प्रकारः अयुक्तः वा राशिः दूषितः वा।".to_string(),
            Some("Check that all operations receive the expected types (int + int, bool in conditions, etc.).".to_string()),
        )
    } else if msg.contains("type") || msg.contains("प्रकार") {
        (
            "A type mismatch was detected. Vyākṛti expects type annotations to match the actual values used.".to_string(),
            "प्रकार-असङ्गतिः — प्रकार-निर्देशः मूल्येन सह न मिलति।".to_string(),
            Some("Verify the declared type matches the assigned value's actual type.".to_string()),
        )
    } else {
        (
            format!("Vyākṛti compiler diagnostic: {}", payload.message),
            "व्याकृति-संकलक-निदानम्।".to_string(),
            None,
        )
    };

    Json(ExplainResponse { explanation, sanskrit_hint, suggestion })
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/compile", post(compile_handler))
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .route("/explain", post(explain_handler))
        .route("/workspace/list", get(workspace_list_handler))
        .route("/workspace/read", post(workspace_read_handler))
        .route("/workspace/write", post(workspace_write_handler))
        .route("/workspace/search", post(workspace_search_handler))
        .route("/lsp/parse", post(lsp_parse_handler))
        .route("/lsp/diagnostics", post(lsp_diagnostics_handler))
        .route("/lsp/completions", get(lsp_completions_handler))
        .route("/lsp/hover", post(lsp_hover_handler))
        .route("/lsp/symbols", post(lsp_symbols_handler))
        .route("/lsp/format", post(lsp_format_handler))
        .route("/lsp/definition", post(lsp_definition_handler))
        .layer(cors);

    let bind_addr = std::env::var("VYAKRTI_BACKEND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap();
    println!("Vyākṛti Backend listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
