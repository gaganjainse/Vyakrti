use axum::extract::ws::{Message, WebSocket};
use futures::stream::StreamExt;
use vyakriti::lexer::Lexer;
use vyakriti::parser::Parser;
use vyakriti::compiler::BytecodeCompiler;
use vyakriti::vm::VirtualMachine;

pub async fn handle_socket(mut socket: WebSocket) {
    let _ = socket
        .send(Message::Text("व्याकृति-यन्त्रः संयुक्तः — कूटं प्रेषयतु।".into()))
        .await;

    let mut vm = VirtualMachine::new();

    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                let trimmed = text.trim();
                let response = if trimmed == ":reset" {
                    vm = VirtualMachine::new();
                    "OK reset".to_string()
                } else if trimmed == ":globals" {
                    let globals = vm.globals.iter()
                        .map(|(k, v)| format!("{} = {:?}", k, v))
                        .collect::<Vec<_>>();
                    if globals.is_empty() { "Globals: (empty)".to_string() } else { format!("Globals: {}", globals.join(", ")) }
                } else if trimmed == ":cancel" {
                    "OK cancel requested; no long-running job is active".to_string()
                } else {
                    match execute_in_vm(&mut vm, &text) {
                        Ok(output) => output,
                        Err(e) => format!("Error: {}", e),
                    }
                };
                if socket.send(Message::Text(response)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

fn execute_in_vm(vm: &mut VirtualMachine, source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let spanned_tokens = lexer.tokenize();
    let mut parser = Parser::new(spanned_tokens);
    let ast = parser.parse_program().map_err(|e| format!("Parse error: {}", e))?;

    let mut cc = BytecodeCompiler::new();
    for node in ast {
        cc.compile(node).map_err(|e| format!("Compile error: {}", e))?;
    }
    cc.link_unresolved_references().map_err(|e| format!("Compile error: {}", e))?;

    let bytecode = cc.get_bytecode();
    vm.run(&bytecode, 0).map_err(|e| format!("VM error: {}", e))?;

    let mut output = String::from("OK");
    if let Some(val) = vm.globals.iter().last() {
        output = format!("{:?} = {:?}", val.0, val.1);
    }
    Ok(output)
}
