use std::env;
use std::fs;
use std::io::{self, Write};

fn run_pipeline(source: &str, disasm: bool) -> Result<(), String> {
    use vyakriti::lexer::Lexer;
    use vyakriti::parser::Parser;
    use vyakriti::semantic::TypeChecker;
    use vyakriti::macro_expander::MacroExpander;
    use vyakriti::derive_processor::DeriveProcessor;
    use vyakriti::monomorphizer::ASTMonomorphizer;
    use vyakriti::optimizer::ASTOptimizer;
    use vyakriti::borrow_checker::BorrowChecker;
    use vyakriti::exhaustiveness::ExhaustivenessAnalyzer;
    use vyakriti::compiler::BytecodeCompiler;
    use vyakriti::vm::VirtualMachine;

    let mut lexer = Lexer::new(source);
    let spanned_tokens = lexer.tokenize();
    let _tokens: Vec<_> = spanned_tokens.iter().map(|st| st.token.clone()).collect();

    let mut parser = Parser::new(spanned_tokens);
    let raw_ast = parser.parse_program().map_err(|e| format!("Parse error: {}", e))?;

    let mut type_checker = TypeChecker::new();
    let checked_ast = type_checker.check_program(raw_ast)
        .map_err(|errors| format!("Semantic errors:\n  {}", errors.join("\n  ")))?;

    let mut macro_engine = MacroExpander::new();
    let macro_ast = macro_engine.expand_program(checked_ast);

    let mut derive_engine = DeriveProcessor::new();
    let derived_ast = derive_engine.expand_attributes(macro_ast);

    let mut monomorphizer = ASTMonomorphizer::new();
    let monomorphized_ast = monomorphizer.process_program(derived_ast);

    let optimizer = ASTOptimizer::new();
    let optimized_ast = optimizer.optimize_program(monomorphized_ast);

    let mut safety_analyzer = BorrowChecker::new();
    safety_analyzer.check_program(&optimized_ast)?;

    let mut exhaustiveness_pass = ExhaustivenessAnalyzer::new();
    exhaustiveness_pass.analyze_program(&optimized_ast)
        .map_err(|e| e.to_string())?;

    let mut bytecode_compiler = BytecodeCompiler::new();
    for node in optimized_ast {
        bytecode_compiler.compile(node).map_err(|e| format!("{}", e))?;
    }
    bytecode_compiler.link_unresolved_references()
        .map_err(|e| format!("{}", e))?;
    let bytecode = bytecode_compiler.get_bytecode();

    if disasm {
        vyakriti::disassembler::Disassembler::disassemble(&bytecode);
    }

    let mut vm = VirtualMachine::new();
    vm.run(&bytecode, 0).map_err(|e| format!("VM error: {}", e))?;
    vm.print_top_stack_value();
    Ok(())
}

fn cmd_compile(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: vy compile <file.vya>");
        std::process::exit(1);
    }
    let src_path = &args[0];
    let source = fs::read_to_string(src_path)
        .unwrap_or_else(|e| { eprintln!("Error reading {}: {}", src_path, e); std::process::exit(1); });

    match run_pipeline(&source, true) {
        Ok(()) => {},
        Err(e) => { eprintln!("Compilation failed: {}", e); std::process::exit(1); }
    }
}

fn cmd_run(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: vy run <file.vya>");
        std::process::exit(1);
    }
    let src_path = &args[0];
    let source = fs::read_to_string(src_path)
        .unwrap_or_else(|e| { eprintln!("Error reading {}: {}", src_path, e); std::process::exit(1); });

    match run_pipeline(&source, false) {
        Ok(()) => {},
        Err(e) => { eprintln!("Runtime error: {}", e); std::process::exit(1); }
    }
}

fn cmd_repl() {
    println!("Vyākṛti REPL v2026.1.0");
    println!("Type :quit to exit");
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() || line.trim() == ":quit" {
            break;
        }
        let source = format!("मान _ = {} ।", line.trim());
        match run_pipeline(&source, false) {
            Ok(()) => {},
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Vyākṛti Language Toolchain v2026.1.0");
        eprintln!("Usage:");
        eprintln!("  vy compile <file.vya>   Compile and run with disassembly");
        eprintln!("  vy run <file.vya>       Run a Vyākṛti source file");
        eprintln!("  vy repl                Start interactive REPL");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "compile" => cmd_compile(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "repl" => cmd_repl(),
        _ => {
            eprintln!("Unknown command: {}. Use compile, run, or repl.", args[1]);
            std::process::exit(1);
        }
    }
}
