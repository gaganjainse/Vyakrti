#![allow(clippy::type_complexity)]
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod semantic;
pub mod macro_expander;
pub mod derive_processor;
pub mod monomorphizer;
pub mod optimizer;
pub mod borrow_checker;
pub mod exhaustiveness;
pub mod compiler;
pub mod jit_memory;
pub mod jit_compiler;
pub mod vm;
pub mod disassembler;

use lexer::Lexer;
use parser::Parser;
use semantic::TypeChecker;
use macro_expander::MacroExpander;
use derive_processor::DeriveProcessor;
use monomorphizer::ASTMonomorphizer;
use optimizer::ASTOptimizer;
use borrow_checker::BorrowChecker;
use exhaustiveness::ExhaustivenessAnalyzer;
use compiler::BytecodeCompiler;
use vm::VirtualMachine;

fn run_pipeline(source: &str) -> Result<(), String> {
    println!("[Phase 1] Lexical Scan Pass...");
    let mut lexer = Lexer::new(source);
    let spanned_tokens = lexer.tokenize();
    let tokens: Vec<_> = spanned_tokens.iter().map(|st| st.token.clone()).collect();
    println!("       Tokens generated: {}", tokens.len());

    println!("[Phase 2] Syntax Parse Pass...");
    let mut parser = Parser::new(spanned_tokens);
    let raw_ast = parser.parse_program().map_err(|e| format!("Parse error: {}", e))?;

    println!("[Phase 2b] Kāraka-Driven Semantic Type Check Pass...");
    let mut type_checker = TypeChecker::new();
    let checked_ast = type_checker.check_program(raw_ast).map_err(|errors| {
        format!("Semantic errors:\n  {}", errors.join("\n  "))
    })?;
    println!("       Type check passed. {} scope(s), {} symbol(s).",
        type_checker.table.current_depth,
        type_checker.table.symbol_count()
    );

    println!("[Phase 3] Macro Template Expansion Pass...");
    let mut macro_engine = MacroExpander::new();
    let macro_ast = macro_engine.expand_program(checked_ast);

    println!("[Phase 4] Attribute Auto-Derivation Pass...");
    let mut derive_engine = DeriveProcessor::new();
    let derived_ast = derive_engine.expand_attributes(macro_ast);

    println!("[Phase 5] Generics Monomorphization Pass...");
    let mut monomorphizer = ASTMonomorphizer::new();
    let monomorphized_ast = monomorphizer.process_program(derived_ast);

    println!("[Phase 6] Ahead-of-Time Optimization Pass...");
    let optimizer = ASTOptimizer::new();
    let optimized_ast = optimizer.optimize_program(monomorphized_ast);
    println!("       AST nodes after optimization: {}", optimized_ast.len());

    println!("[Phase 7] Memory Safety & Reference Lifetime Verification Pass...");
    let mut safety_analyzer = BorrowChecker::new();
    safety_analyzer.check_program(&optimized_ast)?;
    println!("       Borrow check passed.");

    println!("[Phase 8] Algebraic Data Type Branch Exhaustiveness Pass...");
    let mut exhaustiveness_pass = ExhaustivenessAnalyzer::new();
    exhaustiveness_pass.analyze_program(&optimized_ast).map_err(|e| e.to_string())?;
    println!("       Exhaustiveness check passed.");

    println!("[Phase 9] Bytecode Serialization Generation Pass...");
    let mut bytecode_compiler = BytecodeCompiler::new();
    for node in optimized_ast { bytecode_compiler.compile(node).map_err(|e| format!("{}", e))?; }
    bytecode_compiler.link_unresolved_references().map_err(|e| format!("{}", e))?;
    let bytecode = bytecode_compiler.get_bytecode();
    println!("       Bytecode generated: {} bytes", bytecode.len());

    crate::disassembler::Disassembler::disassemble(&bytecode);

    println!("[Phase 10] Virtual Machine Runtime Loop Execution...");
    let mut vm = VirtualMachine::new();
    vm.run(&bytecode, 0).map_err(|e| format!("VM error: {}", e))?;
    println!("       Execution complete.");
    println!("\n--- व्याकृतिः:toolchain integration verification pipeline ---");
    vm.print_top_stack_value();
    Ok(())
}

fn main() {
    let source_code_payload = r#"
        मान मूल्यम् : अङ्क = ६० * ६० ।
    "#;

    if let Err(e) = run_pipeline(source_code_payload) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Token, Lexer};
    use crate::vm::Value;
    use crate::ast::{ASTNode, Expression};
    use crate::compiler::is_builtin;
    use crate::vm::OpCode;

    // --- Kāraka / Semantic Tests ---

    #[test]
    fn test_extract_karaka_devanagari() {
        let (base, k) = semantic::extract_karaka("रामम्_कर्म");
        assert_eq!(base, "रामम्");
        assert_eq!(k, semantic::Karaka::Karma);
    }

    #[test]
    fn test_extract_karaka_no_suffix() {
        let (base, k) = semantic::extract_karaka("x");
        assert_eq!(base, "x");
        assert_eq!(k, semantic::Karaka::Default);
    }

    #[test]
    fn test_type_checker_accepts_valid_program() {
        let mut lex = Lexer::new("मान x : अङ्क = 42 ।");
        let st = lex.tokenize();
        let mut p = Parser::new(st);
        let ast = p.parse_program().unwrap();
        let mut tc = TypeChecker::new();
        assert!(tc.check_program(ast).is_ok());
    }

    #[test]
    fn test_type_checker_rejects_type_mismatch() {
        let mut lex = Lexer::new("मान x : अङ्क = सत ।");
        let st = lex.tokenize();
        let mut p = Parser::new(st);
        let ast = p.parse_program().unwrap();
        let mut tc = TypeChecker::new();
        let result = tc.check_program(ast);
        assert!(result.is_err(), "type mismatch should produce an error");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("type mismatch")));
    }

    #[test]
    fn test_type_checker_rejects_undefined_variable() {
        let mut lex = Lexer::new("मान x : अङ्क = y ।");
        let st = lex.tokenize();
        let mut p = Parser::new(st);
        let ast = p.parse_program().unwrap();
        let mut tc = TypeChecker::new();
        let result = tc.check_program(ast);
        assert!(result.is_err(), "undefined variable should produce an error");
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("undefined variable")));
    }

    #[test]
    fn test_karaka_detects_all_suffixes() {
        let cases = [
            ("_कर्तृ", semantic::Karaka::Kartr),
            ("_कर्म", semantic::Karaka::Karma),
            ("_करण", semantic::Karaka::Karana),
            ("_सम्प्रदान", semantic::Karaka::Sampradana),
            ("_अपादान", semantic::Karaka::Apadana),
            ("_सम्बन्ध", semantic::Karaka::Sambandha),
            ("_अधिकरण", semantic::Karaka::Adhikarana),
        ];
        for (suffix, expected) in &cases {
            let name = format!("val{}", suffix);
            let (base, k) = semantic::extract_karaka(&name);
            assert_eq!(base, "val", "failed for suffix {}", suffix);
            assert_eq!(k, *expected, "failed for suffix {}", suffix);
        }
    }

    #[test]
    fn test_symbol_table_scope_management() {
        let mut st = semantic::SymbolTable::new();
        st.insert("x".into(), semantic::Symbol {
            name: "x".into(), base_name: "x".into(),
            karaka: semantic::Karaka::Default,
            resolved_type: semantic::ResolvedType::Known(semantic::GanaType::int()),
            depth: 0, is_function: false, param_count: 0, param_types: vec![],
            access_level: semantic::AccessModifier::Private,
        });
        assert!(st.lookup("x").is_some());
        st.push_scope();
        st.insert("y".into(), semantic::Symbol {
            name: "y".into(), base_name: "y".into(),
            karaka: semantic::Karaka::Default,
            resolved_type: semantic::ResolvedType::Known(semantic::GanaType::bool()),
            depth: 1, is_function: false, param_count: 0, param_types: vec![],
            access_level: semantic::AccessModifier::Private,
        });
        assert!(st.lookup("x").is_some(), "outer scope visible from inner");
        assert!(st.lookup("y").is_some(), "inner scope visible");
        st.pop_scope();
        assert!(st.lookup("y").is_none(), "inner scope gone after pop");
        assert!(st.lookup("x").is_some(), "outer scope still visible");
    }

    #[test]
    fn test_type_checker_functions_typecheck() {
        let src = r#"
            कार्य योगः(क : अङ्क, ख : अङ्क) {
                मान फलम् : अङ्क = क + ख ।
            }
            योगः(3, 4) ।
        "#;
        let mut lex = Lexer::new(src);
        let st = lex.tokenize();
        let mut p = Parser::new(st);
        let ast = p.parse_program().unwrap();
        let mut tc = TypeChecker::new();
        assert!(tc.check_program(ast).is_ok(), "valid function should typecheck");
    }

    #[test]
    fn test_std_pipeline_with_semantic_pass() {
        let source = "मान x : अङ्क = 5 ।";
        let mut lex = Lexer::new(source);
        let st = lex.tokenize();
        let mut p = Parser::new(st);
        let ast = p.parse_program().unwrap();
        let mut tc = TypeChecker::new();
        let checked = tc.check_program(ast).expect("semantic pass should succeed");
        assert_eq!(checked.len(), 1, "should have 1 node after semantic pass");
    }

    #[test]
    fn test_lexer_basic_tokens() {
        let mut lex = Lexer::new("मान x : अङ्क = 5 ।");
        let tokens = lex.tokenize();
        assert!(!tokens.is_empty(), "should produce tokens");
        assert_eq!(tokens[0].token, Token::VarDeclKW);
        assert!(tokens.iter().any(|t| matches!(&t.token, Token::Identifier(id) if id == "x")));
        assert!(tokens.iter().any(|t| t.token == Token::IntLiteral(5)));
        assert!(tokens.iter().any(|t| t.token == Token::Danda));
    }

    #[test]
    fn test_lexer_less_equal_greater_equal() {
        let mut lex = Lexer::new("x <= 5 >= 3");
        let tokens = lex.tokenize();
        let toks: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert!(toks.contains(&&Token::LessEqual), "<= should emit LessEqual");
        assert!(toks.contains(&&Token::GreaterEqual), ">= should emit GreaterEqual");
    }

    #[test]
    fn test_lexer_devanagari_keywords() {
        let mut lex = Lexer::new("यदि तर्हि अन्यथा प्रतिफल कार्य मान");
        let tokens = lex.tokenize();
        let toks: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert!(toks.contains(&&Token::If));
        assert!(toks.contains(&&Token::Tarhi));
        assert!(toks.contains(&&Token::Anyatha));
        assert!(toks.contains(&&Token::Pratiphala));
        assert!(toks.contains(&&Token::FuncDeclKW));
        assert!(toks.contains(&&Token::VarDeclKW));
    }

    #[test]
    fn test_lexer_cha_va() {
        let mut lex = Lexer::new("च वा");
        let tokens = lex.tokenize();
        let toks: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert!(toks.contains(&&Token::Cha));
        assert!(toks.contains(&&Token::Va));
    }

    #[test]
    fn test_lexer_string_escape() {
        let mut lex = Lexer::new(r#""hello\nworld""#);
        let tokens = lex.tokenize();
        if let Token::Literal(Value::Str(s)) = &tokens[0].token {
            assert_eq!(s, "hello\nworld");
        } else {
            panic!("expected Str literal");
        }
    }

    #[test]
    fn test_lexer_devanagari_digit_normalization() {
        let mut lex = Lexer::new("१२३");
        let tokens = lex.tokenize();
        assert_eq!(tokens[0].token, Token::IntLiteral(123));
    }

    #[test]
    fn test_parser_simple_program() {
        let mut lex = Lexer::new("मान x : अङ्क = 5 ।");
        let spanned = lex.tokenize();
        let mut parser = Parser::new(spanned);
        let ast = parser.parse_program().expect("should parse");
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            ASTNode::VarDecl { name, data_type, value, .. } => {
                assert_eq!(name, "x");
                assert_eq!(data_type, &Some("अङ्क".into()));
                assert_eq!(value, &Expression::IntLiteral(5));
            }
            _ => panic!("expected VarDecl"),
        }
    }

    #[test]
    fn test_parser_binary_expression() {
        let mut lex = Lexer::new("मान y : अङ्क = 10 + 20 ।");
        let spanned = lex.tokenize();
        let mut parser = Parser::new(spanned);
        let ast = parser.parse_program().expect("should parse");
        match &ast[0] {
            ASTNode::VarDecl { value, .. } => {
                match value {
                    Expression::Binary { left, op, right } => {
                        assert_eq!(op, "+");
                        assert_eq!(**left, Expression::IntLiteral(10));
                        assert_eq!(**right, Expression::IntLiteral(20));
                    }
                    _ => panic!("expected Binary expression"),
                }
            }
            _ => panic!("expected VarDecl"),
        }
    }

    #[test]
    fn test_parser_function_return_type_and_parenthesized_if() {
        let src = r#"
            कार्य योगः(क : अङ्क, ख : अङ्क) -> अङ्क {
                प्रतिफल क + ख ।
            }
            यदि (सत) तर्हि { योगः(1, 2) । }
        "#;
        let mut lex = Lexer::new(src);
        let spanned = lex.tokenize();
        let mut parser = Parser::new(spanned);
        let ast = parser.parse_program().expect("should parse function and parens");
        match &ast[0] {
            ASTNode::FuncDecl { return_type, .. } => assert_eq!(return_type, "अङ्क"),
            _ => panic!("expected function declaration"),
        }
        assert!(matches!(ast[1], ASTNode::IfStmt { .. }));
    }

    #[test]
    fn test_borrow_checker_no_false_positive() {
        let mut bc = BorrowChecker::new();
        let nodes = vec![
            ASTNode::VarDecl {
                name: "x".into(),
                data_type: Some("अङ्क".into()),
                value: Expression::IntLiteral(5),
                access_level: crate::ast::AccessModifier::Private,
            },
            ASTNode::StatementExpr(
                Expression::Binary {
                    left: Box::new(Expression::Variable("x".into())),
                    op: "+".into(),
                    right: Box::new(Expression::Variable("x".into())),
                }
            ),
        ];
        assert!(bc.check_program(&nodes).is_ok(), "reading x twice should be allowed");
    }

    #[test]
    fn test_optimizer_constant_folding() {
        let opt = ASTOptimizer::new();
        let nodes = vec![
            ASTNode::VarDecl {
                name: "z".into(),
                data_type: None,
                value: Expression::Binary {
                    left: Box::new(Expression::IntLiteral(100)),
                    op: "*".into(),
                    right: Box::new(Expression::IntLiteral(200)),
                },
                access_level: crate::ast::AccessModifier::Private,
            },
        ];
        let result = opt.optimize_program(nodes);
        if let ASTNode::VarDecl { value, .. } = &result[0] {
            assert_eq!(value, &Expression::Literal(Value::Int(20000)));
        } else {
            panic!("expected VarDecl with folded literal");
        }
    }

    #[test]
    fn test_vm_simple_execution() {
        let mut vm = VirtualMachine::new();
        let mut bc = BytecodeCompiler::new();
        bc.compile(ASTNode::VarDecl {
            name: "result".into(),
            data_type: None,
            value: Expression::IntLiteral(42),
            access_level: crate::ast::AccessModifier::Private,
        }).expect("compile var");
        bc.link_unresolved_references().expect("link should succeed");
        let bytecode = bc.get_bytecode();
        vm.run(&bytecode, 0).expect("VM should run");
        assert_eq!(vm.globals.get("result"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_vm_arithmetic() {
        let mut vm = VirtualMachine::new();
        let mut bc = BytecodeCompiler::new();
        bc.compile(ASTNode::StatementExpr(
            Expression::Binary {
                left: Box::new(Expression::IntLiteral(6)),
                op: "*".into(),
                right: Box::new(Expression::IntLiteral(7)),
            }
        )).expect("compile expr");
        bc.link_unresolved_references().expect("link");
        let bytecode = bc.get_bytecode();
        vm.run(&bytecode, 0).expect("VM should run");
    }

    #[test]
    fn test_vm_function_parameters_return_value_and_local_scope() {
        let mut compiler = BytecodeCompiler::new();
        compiler.compile(ASTNode::FuncDecl {
            name: "योगः".into(),
            parameters: vec![("क".into(), "अङ्क".into()), ("ख".into(), "अङ्क".into())],
            return_type: "अङ्क".into(),
            body: vec![
                ASTNode::VarDecl {
                    name: "अन्तः".into(),
                    data_type: Some("अङ्क".into()),
                    value: Expression::Binary {
                        left: Box::new(Expression::Variable("क".into())),
                        op: "+".into(),
                        right: Box::new(Expression::Variable("ख".into())),
                    },
                    access_level: crate::ast::AccessModifier::Private,
                },
                ASTNode::ReturnStmt(Expression::Variable("अन्तः".into())),
            ],
            access_level: crate::ast::AccessModifier::Private,
        }).expect("compile function");
        compiler.compile(ASTNode::StatementExpr(Expression::Call {
            name: "योगः".into(),
            args: vec![Expression::IntLiteral(3), Expression::IntLiteral(4)],
        })).expect("compile call");
        compiler.link_unresolved_references().expect("link");

        let mut vm = VirtualMachine::new();
        vm.run(&compiler.get_bytecode(), 0).expect("VM should run function");
        assert_eq!(vm.stack_top(), Some(&Value::Int(7)));
        assert!(vm.globals.get("क").is_none(), "parameter must stay local");
        assert!(vm.globals.get("अन्तः").is_none(), "function local must not leak into globals");
    }

    #[test]
    fn test_selfhost_corpus_parses_and_golden_runs() {
        let corpus = [
            include_str!("../selfhost/tokens.vya"),
            include_str!("../selfhost/lexer.vya"),
            include_str!("../selfhost/parser.vya"),
            include_str!("../selfhost/diagnostics.vya"),
        ];
        for source in corpus {
            let mut lex = Lexer::new(source);
            let mut parser = Parser::new(lex.tokenize());
            let ast = parser.parse_program().expect("selfhost source should parse");
            let mut tc = TypeChecker::new();
            tc.check_program(ast).expect("selfhost source should typecheck");
        }

        run_pipeline(include_str!("../selfhost/golden_smoke.vya")).expect("selfhost golden smoke should run");
    }

    #[test]
    fn test_lexer_source_location() {
        let mut lex = Lexer::new("मान\nx = 5\n।");
        let tokens = lex.tokenize();
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].col, 1);
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].col, 1);
    }

    #[test]
    fn test_parser_error_with_location() {
        let mut lex = Lexer::new("मान x : अङ्क = 5"); // missing ।
        let spanned = lex.tokenize();
        let mut parser = Parser::new(spanned);
        let result = parser.parse_program();
        assert!(result.is_err(), "should fail due to missing ।");
        let err = result.unwrap_err();
        assert!(err.contains("error:"), "error should contain 'error:', got: {}", err);
    }

    #[test]
    fn test_builtin_is_builtin() {
        assert!(is_builtin("मुद्रण"));
        assert!(is_builtin("दैर्घ्यम्"));
        assert!(is_builtin("प्रकारः"));
        assert!(!is_builtin("print"));
        assert!(!is_builtin("len"));
        assert!(!is_builtin("user_fn"));
    }

    #[test]
    fn test_builtin_emit_callbuiltin() {
        let mut cc = BytecodeCompiler::new();
        cc.compile_expr(Expression::Call {
            name: "मुद्रण".into(),
            args: vec![Expression::Literal(Value::Str("hello".into()))],
        }).expect("compile builtin");
        let bc = cc.get_bytecode();
        assert!(!bc.is_empty());
    }

    #[test]
    fn test_builtin_vm_len() {
        let mut vm = VirtualMachine::new();
        let mut cc = BytecodeCompiler::new();
        cc.compile_expr(Expression::Call {
            name: "दैर्घ्यम्".into(),
            args: vec![Expression::Literal(Value::Str("व्याकृति".into()))],
        }).expect("compile builtin");
        cc.link_unresolved_references().expect("link");
        let bc = cc.get_bytecode();
        vm.run(&bc, 0).expect("VM should run");
    }

    #[test]
    fn test_builtin_skipped_by_linker() {
        let mut cc = BytecodeCompiler::new();
        cc.compile(ASTNode::StatementExpr(Expression::Call {
            name: "मुद्रण".into(),
            args: vec![Expression::IntLiteral(42)],
        })).expect("compile builtin statement");
        assert!(cc.link_unresolved_references().is_ok());
    }

    #[test]
    fn test_vm_comparison_ops() {
        let mut vm = VirtualMachine::new();
        let mut bc = Vec::new();
        let push_int = |bc: &mut Vec<u8>, n: i64| {
            bc.push(OpCode::PushConst as u8);
            bc.push(0); bc.extend_from_slice(&n.to_be_bytes());
        };
        push_int(&mut bc, 10); push_int(&mut bc, 20);
        bc.push(OpCode::LessThan as u8);
        push_int(&mut bc, 20); push_int(&mut bc, 10);
        bc.push(OpCode::GreaterThan as u8);
        push_int(&mut bc, 10); push_int(&mut bc, 10);
        bc.push(OpCode::LessEqual as u8);
        push_int(&mut bc, 10); push_int(&mut bc, 10);
        bc.push(OpCode::GreaterEqual as u8);
        push_int(&mut bc, 10); push_int(&mut bc, 20);
        bc.push(OpCode::NotEqual as u8);
        vm.run(&bc, 0).expect("comparison VM should run");
    }

    #[test]
    fn test_vm_match_expr() {
        let mut cc = BytecodeCompiler::new();
        cc.compile_expr(Expression::MatchExpr {
            eval_target: Box::new(Expression::Literal(Value::Enum("रक्त".into(), Box::new(Value::Int(42))))),
            arms: vec![
                ("रक्त".into(), vec!["x".into()], Expression::Literal(Value::Int(1))),
                ("नील".into(), vec![], Expression::Literal(Value::Int(2))),
            ],
        }).expect("compile match");
        cc.link_unresolved_references().expect("link");
        let bc = cc.get_bytecode();
        let mut vm = VirtualMachine::new();
        vm.run(&bc, 0).expect("match VM should run");
    }

    #[test]
    fn test_vm_field_access() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("x".into(), Value::Int(42));
        let s = Value::Struct(fields);
        let mut cc = BytecodeCompiler::new();
        cc.compile_expr(Expression::FieldAccess {
            object: Box::new(Expression::Literal(s)),
            field: "x".into(),
        }).expect("compile field access");
        cc.link_unresolved_references().expect("link");
        let bc = cc.get_bytecode();
        let mut vm = VirtualMachine::new();
        vm.run(&bc, 0).expect("field access VM should run");
    }

    #[test]
    fn test_new_builtins() {
        let mut vm = VirtualMachine::new();
        let mut cc = BytecodeCompiler::new();
        cc.compile_expr(Expression::Call {
            name: "योजन".into(),
            args: vec![
                Expression::Literal(Value::Str("hello ".into())),
                Expression::Literal(Value::Str("world".into())),
            ],
        }).expect("compile concat builtin");
        cc.link_unresolved_references().expect("link");
        let bc = cc.get_bytecode();
        vm.run(&bc, 0).expect("concat builtin should work");
    }

    #[test]
    fn test_vm_dup_drop() {
        let mut vm = VirtualMachine::new();
        let mut bc = Vec::new();
        bc.push(OpCode::PushConst as u8); bc.push(0);
        bc.extend_from_slice(&42i64.to_be_bytes());
        bc.push(OpCode::Dup as u8);
        bc.push(OpCode::Drop as u8);
        bc.push(OpCode::Drop as u8);
        vm.run(&bc, 0).expect("dup/drop should work");
    }

    #[test]
    fn test_vm_get_variant() {
        let mut vm = VirtualMachine::new();
        let mut bc = Vec::new();
        let name = "test_variant";
        bc.push(OpCode::PushConst as u8); bc.push(4); // enum tag
        bc.extend_from_slice(&(name.len() as u16).to_be_bytes());
        bc.extend_from_slice(name.as_bytes());
        bc.push(OpCode::GetVariant as u8);
        vm.run(&bc, 0).expect("get variant should work");
    }
}

