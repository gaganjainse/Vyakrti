use crate::payload::{CompileResponse, Diagnostic};
use serde_json::json;
use vyakriti::lexer::Lexer;
use vyakriti::parser::Parser;
use vyakriti::semantic::TypeChecker;
use vyakriti::macro_expander::MacroExpander;
use vyakriti::derive_processor::DeriveProcessor;
use vyakriti::monomorphizer::ASTMonomorphizer;
use vyakriti::borrow_checker::BorrowChecker;
use vyakriti::exhaustiveness::ExhaustivenessAnalyzer;
use vyakriti::optimizer::ASTOptimizer;
use vyakriti::compiler::BytecodeCompiler;
use vyakriti::vm::{VirtualMachine, Value, OpCode};
use vyakriti::ast::Expression;

pub fn compile_source(source: &str) -> CompileResponse {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut output: Vec<String> = Vec::new();

    // Phase 1: Lexical analysis
    let mut lexer = Lexer::new(source);
    let spanned_tokens = lexer.tokenize();
    let tokens: Vec<_> = spanned_tokens.iter().map(|st| st.token.clone()).collect();

    let raw_lines: Vec<&str> = source.lines().collect();
    output.push(format!("पठितम् {} पङ्क्तयः, {} वर्णाः।", raw_lines.len(), source.len()));

    // Token display strings
    let token_displays: Vec<String> = spanned_tokens.iter().map(|st| {
        format!("{:?} [{}:{}]", st.token, st.line, st.col)
    }).collect();

    if tokens.is_empty() {
        diagnostics.push(Diagnostic {
            line: 1, column: 1,
            message: "Empty source file or no tokens recognized.".to_string(),
            sanskrit_message: Some("रिक्तं मूलपत्रम्".to_string()),
            severity: "warning".to_string(),
        });
    }

    // Phase 2: Parse
    let ast_value = match Parser::new(spanned_tokens.clone()).parse_program() {
        Ok(ast) => {
            // Build JSON AST for frontend display
            let json_ast = build_json_ast(&ast);

            // Phase 2b: Semantic type check (Kāraka-driven)
            let mut tc = TypeChecker::new();
            let checked_ast = match tc.check_program(ast) {
                Ok(ca) => ca,
                Err(errors) => {
                    for err in errors {
                        let (line, column) = infer_location(source, &err);
                        diagnostics.push(Diagnostic {
                            line, column,
                            message: format!("Type error: {}", err),
                            sanskrit_message: None,
                            severity: "error".to_string(),
                        });
                    }
                    // Continue with unchecked AST for partial diagnostics
                    let mut lex = Lexer::new(source);
                    let toks = lex.tokenize();
                    let mut par = Parser::new(toks);
                    par.parse_program().unwrap_or_default()
                }
            };

            // Phase 3: Macro expansion
            let mut macro_engine = MacroExpander::new();
            let macro_ast = macro_engine.expand_program(checked_ast);

            // Phase 4: Derive processing
            let mut derive_engine = DeriveProcessor::new();
            let derived_ast = derive_engine.expand_attributes(macro_ast);

            // Phase 5: Monomorphization
            let mut monomorphizer = ASTMonomorphizer::new();
            let mono_ast = monomorphizer.process_program(derived_ast);

            // Phase 6: Optimizer
            let opt = ASTOptimizer::new();
            let optimized = opt.optimize_program(mono_ast);

            // Phase 7: Borrow check
            let mut bc = BorrowChecker::new();
            if let Err(e) = bc.check_program(&optimized) {
                let (line, column) = infer_location(source, &e);
                diagnostics.push(Diagnostic {
                    line, column,
                    message: format!("Borrow check error: {}", e),
                    sanskrit_message: None,
                    severity: "error".to_string(),
                });
            }

            // Phase 8: Exhaustiveness
            let mut ea = ExhaustivenessAnalyzer::new();
            if let Err(e) = ea.analyze_program(&optimized) {
                let (line, column) = infer_location(source, &e);
                diagnostics.push(Diagnostic {
                    line, column,
                    message: format!("Exhaustiveness check error: {}", e),
                    sanskrit_message: None,
                    severity: "error".to_string(),
                });
            }

            // Phase 9: Compile
            let mut cc = BytecodeCompiler::new();
            for node in optimized {
                if let Err(e) = cc.compile(node) {
                    let (line, column) = infer_location(source, &e.to_string());
                    diagnostics.push(Diagnostic {
                        line, column,
                        message: format!("Compile error: {}", e),
                        sanskrit_message: None,
                        severity: "error".to_string(),
                    });
                    break;
                }
            }
            match cc.link_unresolved_references() {
                Ok(()) => {
                    let bytecode = cc.get_bytecode();
                    let bc_display = format_bytecode(&bytecode);

                    // Phase 10: Run VM
                    if diagnostics.iter().all(|d| d.severity != "error") {
                        let mut vm = VirtualMachine::new();
                        match vm.run(&bytecode, 0) {
                            Ok(()) => {
                                output.push("निर्वाहः सफलः समाप्तः।".to_string());
                                let top = vm.globals.iter().map(|(k, v)| format!("{} = {:?}", k, v)).collect::<Vec<_>>();
                                if !top.is_empty() {
                                    output.push(format!("Globals: {}", top.join(", ")));
                                }
                            }
                            Err(e) => {
                                let (line, column) = infer_location(source, &e.to_string());
                                diagnostics.push(Diagnostic {
                                    line, column,
                                    message: format!("VM runtime error: {}", e),
                                    sanskrit_message: None,
                                    severity: "error".to_string(),
                                });
                            }
                        }
                    }

                    (json_ast, token_displays, bc_display)
                }
                Err(e) => {
                    let (line, column) = infer_location(source, &e.to_string());
                    diagnostics.push(Diagnostic {
                        line, column,
                        message: format!("Link error: {}", e),
                        sanskrit_message: None,
                        severity: "error".to_string(),
                    });
                    (json_ast, token_displays, String::new())
                }
            }
        }
        Err(e) => {
            let (line, col) = extract_location(&e);
            diagnostics.push(Diagnostic {
                line, column: col,
                message: format!("Parse error: {}", e),
                sanskrit_message: None,
                severity: "error".to_string(),
            });
            let fallback = json!({ "type": "Program", "script_mode": "mixed", "body": [] });
            (fallback, token_displays, String::new())
        }
    };

    let (ast, token_list, bytecode_str) = ast_value;

    CompileResponse {
        ast,
        tokens: token_list,
        bytecode: bytecode_str,
        diagnostics,
        output,
    }
}

fn extract_location(e: &str) -> (usize, usize) {
    // Parse "error at 3:7: ..." format
    if let Some(rest) = e.strip_prefix("error at ") {
        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() == 2 {
            if let (Ok(line), Some(col_str)) = (parts[0].parse::<usize>(), parts[1].split(':').next()) {
                if let Ok(col) = col_str.parse::<usize>() {
                    return (line, col);
                }
            }
        }
    }
    (1, 1)
}

fn infer_location(source: &str, message: &str) -> (usize, usize) {
    if let Some(symbol) = message.split('\'').nth(1) {
        for (line_idx, line) in source.lines().enumerate() {
            if let Some(col_idx) = line.find(symbol) {
                return (line_idx + 1, col_idx + 1);
            }
        }
    }
    (1, 1)
}

fn build_json_ast(nodes: &[vyakriti::ast::ASTNode]) -> serde_json::Value {
    let body: Vec<serde_json::Value> = nodes.iter().map(|node| {
        match node {
            vyakriti::ast::ASTNode::VarDecl { name, data_type, value, .. } => {
                json!({
                    "type": "VariableDeclaration",
                    "name": name,
                    "dataType": data_type,
                    "value": expr_to_json(value)
                })
            }
            vyakriti::ast::ASTNode::FuncDecl { name, parameters, return_type, .. } => {
                json!({
                    "type": "FunctionDeclaration",
                    "name": name,
                    "returnType": return_type,
                    "parameters": parameters.iter().map(|(n, t)| json!({"name": n, "type": t})).collect::<Vec<_>>()
                })
            }
            vyakriti::ast::ASTNode::IfStmt { condition, .. } => {
                json!({
                    "type": "IfStatement",
                    "condition": expr_to_json(condition)
                })
            }
            vyakriti::ast::ASTNode::WhileStmt { condition, .. } => {
                json!({
                    "type": "WhileStatement",
                    "condition": expr_to_json(condition)
                })
            }
            vyakriti::ast::ASTNode::EnumDecl { name, variants, .. } => {
                json!({
                    "type": "EnumDeclaration",
                    "name": name,
                    "variants": variants.iter().map(|(v, _)| v).collect::<Vec<_>>()
                })
            }
            vyakriti::ast::ASTNode::StatementExpr(expr) => {
                json!({
                    "type": "ExpressionStatement",
                    "expression": expr_to_json(expr)
                })
            }
            vyakriti::ast::ASTNode::StructDecl { name, attributes, fields, .. } => {
                json!({
                    "type": "StructDeclaration",
                    "name": name,
                    "attributes": attributes,
                    "fields": fields.iter().map(|(n, t)| json!({"name": n, "type": t})).collect::<Vec<_>>()
                })
            }
            _ => json!({ "type": "Unknown" })
        }
    }).collect();

    json!({ "type": "Program", "script_mode": "mixed", "body": body })
}

fn expr_to_json(expr: &Expression) -> serde_json::Value {
    match expr {
        Expression::Literal(val) => match val {
            Value::Int(n) => json!({ "type": "Literal", "value": n, "kind": "int" }),
            Value::Float(f) => json!({ "type": "Literal", "value": f, "kind": "float" }),
            Value::Bool(b) => json!({ "type": "Literal", "value": b, "kind": "bool" }),
            Value::Str(s) => json!({ "type": "Literal", "value": s, "kind": "string" }),
            _ => json!({ "type": "Literal", "value": null, "kind": "null" }),
        },
        Expression::IntLiteral(n) => json!({ "type": "Literal", "value": n, "kind": "int" }),
        Expression::Variable(name) => json!({ "type": "Identifier", "name": name }),
        Expression::Binary { left, op, right } => json!({
            "type": "BinaryExpression",
            "operator": op,
            "left": expr_to_json(left),
            "right": expr_to_json(right)
        }),
        Expression::Call { name, args } => json!({
            "type": "CallExpression",
            "callee": name,
            "arguments": args.iter().map(expr_to_json).collect::<Vec<_>>()
        }),
        _ => json!({ "type": "Unknown" }),
    }
}

fn format_bytecode(bytecode: &[u8]) -> String {
    let mut lines = Vec::new();
    let mut i = 0;
    while i < bytecode.len() {
        let op = bytecode[i]; i += 1;
        if op == OpCode::PushConst as u8 {
            let tag = bytecode[i]; i += 1;
            match tag {
                0 => {
                    let mut b = [0; 8]; b.copy_from_slice(&bytecode[i..i+8]); i += 8;
                    lines.push(format!("{:04x}  PushConst int({})", i-2, i64::from_be_bytes(b)));
                }
                1 => { lines.push(format!("{:04x}  PushConst bool({})", i-2, bytecode[i] == 1)); i += 1; }
                2 => {
                    let len = u16::from_be_bytes([bytecode[i], bytecode[i+1]]); i += 2;
                    let s = String::from_utf8_lossy(&bytecode[i..i+len as usize]); i += len as usize;
                    lines.push(format!("{:04x}  PushConst str(\"{}\")", i-4-len as usize, s));
                }
                3 => {
                    let mut b = [0; 8]; b.copy_from_slice(&bytecode[i..i+8]); i += 8;
                    lines.push(format!("{:04x}  PushConst float({})", i-2, f64::from_be_bytes(b)));
                }
                4 => {
                    let len = u16::from_be_bytes([bytecode[i], bytecode[i+1]]); i += 2;
                    let s = String::from_utf8_lossy(&bytecode[i..i+len as usize]); i += len as usize;
                    lines.push(format!("{:04x}  PushConst enum({})", i-4-len as usize, s));
                }
                _ => lines.push(format!("{:04x}  PushConst tag={}", i-2, tag)),
            }
        } else if op == OpCode::LoadVar as u8 {
            let len = u16::from_be_bytes([bytecode[i], bytecode[i+1]]); i += 2;
            let s = String::from_utf8_lossy(&bytecode[i..i+len as usize]); i += len as usize;
            lines.push(format!("{:04x}  LoadVar \"{}\"", i-4-len as usize, s));
        } else if op == OpCode::StoreVar as u8 {
            let len = u16::from_be_bytes([bytecode[i], bytecode[i+1]]); i += 2;
            let s = String::from_utf8_lossy(&bytecode[i..i+len as usize]); i += len as usize;
            lines.push(format!("{:04x}  StoreVar \"{}\"", i-4-len as usize, s));
        } else if op == OpCode::Dup as u8 {
            lines.push(format!("{:04x}  Dup", i-1));
        } else if op == OpCode::Drop as u8 {
            lines.push(format!("{:04x}  Drop", i-1));
        } else if op == OpCode::GetVariant as u8 {
            lines.push(format!("{:04x}  GetVariant", i-1));
        } else if op == OpCode::Jump as u8 {
            let addr = u32::from_be_bytes([bytecode[i], bytecode[i+1], bytecode[i+2], bytecode[i+3]]); i += 4;
            lines.push(format!("{:04x}  Jump -> 0x{:04x}", i-5, addr));
        } else if op == OpCode::JumpIfFalse as u8 {
            let addr = u32::from_be_bytes([bytecode[i], bytecode[i+1], bytecode[i+2], bytecode[i+3]]); i += 4;
            lines.push(format!("{:04x}  JumpIfFalse -> 0x{:04x}", i-5, addr));
        } else if op == OpCode::Call as u8 {
            let addr = u32::from_be_bytes([bytecode[i], bytecode[i+1], bytecode[i+2], bytecode[i+3]]); i += 4;
            lines.push(format!("{:04x}  Call -> 0x{:04x}", i-5, addr));
        } else if op == OpCode::Return as u8 {
            lines.push(format!("{:04x}  Return", i-1));
        } else if op == OpCode::CallBuiltin as u8 {
            let len = u16::from_be_bytes([bytecode[i], bytecode[i+1]]); i += 2;
            let s = String::from_utf8_lossy(&bytecode[i..i+len as usize]); i += len as usize;
            let nargs = bytecode[i]; i += 1;
            lines.push(format!("{:04x}  CallBuiltin \"{}\" ({} args)", i-4-len as usize, s, nargs));
        } else if op == OpCode::BindParams as u8 {
            let start = i - 1;
            let count = bytecode[i] as usize; i += 1;
            let mut names = Vec::new();
            for _ in 0..count {
                let len = u16::from_be_bytes([bytecode[i], bytecode[i+1]]); i += 2;
                let s = String::from_utf8_lossy(&bytecode[i..i+len as usize]).to_string();
                i += len as usize;
                names.push(s);
            }
            lines.push(format!("{:04x}  BindParams ({}) {}", start, count, names.join(", ")));
        } else if op == OpCode::Add as u8 { lines.push(format!("{:04x}  Add", i-1));
        } else if op == OpCode::Sub as u8 { lines.push(format!("{:04x}  Sub", i-1));
        } else if op == OpCode::Mul as u8 { lines.push(format!("{:04x}  Mul", i-1));
        } else if op == OpCode::Div as u8 { lines.push(format!("{:04x}  Div", i-1));
        } else if op == OpCode::Equal as u8 { lines.push(format!("{:04x}  Equal", i-1));
        } else if op == OpCode::NotEqual as u8 { lines.push(format!("{:04x}  NotEqual", i-1));
        } else if op == OpCode::LessThan as u8 { lines.push(format!("{:04x}  LessThan", i-1));
        } else if op == OpCode::GreaterThan as u8 { lines.push(format!("{:04x}  GreaterThan", i-1));
        } else if op == OpCode::LessEqual as u8 { lines.push(format!("{:04x}  LessEqual", i-1));
        } else if op == OpCode::GreaterEqual as u8 { lines.push(format!("{:04x}  GreaterEqual", i-1));
        } else if op == OpCode::FAdd as u8 { lines.push(format!("{:04x}  FAdd", i-1));
        } else if op == OpCode::FSub as u8 { lines.push(format!("{:04x}  FSub", i-1));
        } else if op == OpCode::FMul as u8 { lines.push(format!("{:04x}  FMul", i-1));
        } else if op == OpCode::FDiv as u8 { lines.push(format!("{:04x}  FDiv", i-1));
        } else { lines.push(format!("{:04x}  Unknown 0x{:02x}", i-1, op)); }
    }
    lines.join("\n")
}
