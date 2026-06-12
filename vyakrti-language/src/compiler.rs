use std::collections::HashMap;
use crate::ast::{ASTNode, Expression};
use crate::vm::OpCode;

#[derive(Debug)]
pub enum CompileError {
    UnsupportedNode(String),
    UnsupportedOp(String),
    LinkError(String),
    Io(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnsupportedNode(msg) => write!(f, "compile error: unsupported node: {}", msg),
            CompileError::UnsupportedOp(msg) => write!(f, "compile error: unsupported operator: {}", msg),
            CompileError::LinkError(msg) => write!(f, "link error: {}", msg),
            CompileError::Io(msg) => write!(f, "i/o error: {}", msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, CompileError>;

/// Built-in function names that the compiler treats as CallBuiltin.
/// Only pure Devanagari names; SLP1 removed (hard cutoff).
const BUILTIN_NAMES: &[&str] = &[
    "मुद्रण",
    "दैर्घ्यम्",
    "प्रकारः",
    "योजन",
    "अस्ति",
    "संख्या",
    "रूपान्तर",
];

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_NAMES.contains(&name)
}

pub struct BytecodeCompiler {
    bytecode: Vec<u8>,
    pub function_table: HashMap<String, u32>,
    pub unresolved_calls: Vec<(usize, String)>,
    pub current_module: Option<String>,
    pub struct_registry: HashMap<String, Vec<String>>,
    pub foreign_table: HashMap<String, (String, String, usize)>,
}

impl Default for BytecodeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        BytecodeCompiler {
            bytecode: Vec::new(), function_table: HashMap::new(),
            unresolved_calls: Vec::new(), current_module: None,
            struct_registry: HashMap::new(),
            foreign_table: HashMap::new(),
        }
    }

    pub fn get_bytecode(&self) -> Vec<u8> { self.bytecode.clone() }

    fn emit_u8(&mut self, val: u8) { self.bytecode.push(val); }

    fn emit_u16(&mut self, val: u16) {
        self.bytecode.extend_from_slice(&val.to_be_bytes());
    }

    fn emit_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        if bytes.len() > 65535 {
            panic!("string too long: {} bytes", bytes.len());
        }
        self.emit_u16(bytes.len() as u16);
        self.bytecode.extend_from_slice(bytes);
    }

    fn emit_u32_placeholder(&mut self) -> usize {
        let idx = self.bytecode.len();
        self.bytecode.extend_from_slice(&[0, 0, 0, 0]);
        idx
    }

    fn emit_u32_at(&mut self, idx: usize, val: u32) {
        let bytes = val.to_be_bytes();
        self.bytecode[idx..(4 + idx)].copy_from_slice(&bytes);
    }

    pub fn compile(&mut self, node: ASTNode) -> Result<()> {
        match node {
            ASTNode::VarDecl { name, value, .. } => {
                let full_var_name = self.current_module.as_ref().map_or(name.clone(), |m| format!("{}::{}", m, name));
                self.compile_expr(value)?;
                self.emit_u8(OpCode::StoreVar as u8);
                self.emit_string(&full_var_name);
            }
            ASTNode::FuncDecl { name, parameters, body, .. } => {
                let full_func_name = self.current_module.as_ref().map_or(name.clone(), |m| format!("{}::{}", m, name));
                self.emit_u8(OpCode::Jump as u8);
                let skip_placeholder = self.emit_u32_placeholder();
                let entry_pc = self.bytecode.len() as u32;
                self.function_table.insert(full_func_name, entry_pc);

                self.emit_u8(OpCode::BindParams as u8);
                self.emit_u8(parameters.len() as u8);
                for (param_name, _) in parameters {
                    self.emit_string(&param_name);
                }

                for stmt in body { self.compile(stmt)?; }
                self.emit_u8(OpCode::Return as u8);

                let post_pc = self.bytecode.len() as u32;
                self.emit_u32_at(skip_placeholder, post_pc);
            }
            ASTNode::IfStmt { condition, then_branch, else_branch } => {
                self.compile_expr(condition)?;
                self.emit_u8(OpCode::JumpIfFalse as u8);
                let else_placeholder = self.emit_u32_placeholder();

                for stmt in then_branch { self.compile(stmt)?; }
                self.emit_u8(OpCode::Jump as u8);
                let end_placeholder = self.emit_u32_placeholder();

                let else_pc = self.bytecode.len() as u32;
                self.emit_u32_at(else_placeholder, else_pc);

                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts { self.compile(stmt)?; }
                }
                let end_pc = self.bytecode.len() as u32;
                self.emit_u32_at(end_placeholder, end_pc);
            }
            ASTNode::WhileStmt { condition, body } => {
                let loop_start_pc = self.bytecode.len() as u32;
                self.compile_expr(condition)?;
                self.emit_u8(OpCode::JumpIfFalse as u8);
                let exit_placeholder = self.emit_u32_placeholder();

                for stmt in body { self.compile(stmt)?; }
                self.emit_u8(OpCode::Jump as u8);
                let jump_back_placeholder = self.emit_u32_placeholder();
                self.emit_u32_at(jump_back_placeholder, loop_start_pc);

                let exit_pc = self.bytecode.len() as u32;
                self.emit_u32_at(exit_placeholder, exit_pc);
            }
            ASTNode::StructDecl { name, fields, .. } => {
                let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                self.struct_registry.insert(name, field_names);
            }
            ASTNode::FFIDecl { library_path, symbol_name, parameters, .. } => {
                let func_name = symbol_name.clone();
                self.foreign_table.insert(func_name, (library_path, symbol_name, parameters.len()));
            }
            ASTNode::ModuleDef { name, body } => {
                let old_mod = self.current_module.clone();
                self.current_module = Some(name);
                for stmt in body { self.compile(stmt)?; }
                self.current_module = old_mod;
            }
            ASTNode::StatementExpr(expr) => self.compile_expr(expr)?,
            ASTNode::ReturnStmt(expr) => {
                self.compile_expr(expr)?;
                self.emit_u8(OpCode::Return as u8);
            }
            ASTNode::Block(stmts) => {
                for stmt in stmts {
                    self.compile(stmt)?;
                }
            }
            ASTNode::NoOp
            | ASTNode::EnumDecl { .. }
            | ASTNode::TraitDecl { .. }
            | ASTNode::ImplBlock { .. }
            | ASTNode::ImportDecl { .. }
            | ASTNode::ModuleDecl { .. }
            | ASTNode::MacroDecl { .. } => {}
            ASTNode::MacroCall { name, .. } => {
                return Err(CompileError::UnsupportedNode(format!(
                    "macro call '{}!' reached bytecode generation before expansion",
                    name
                )));
            }
            ASTNode::GenericFuncDecl { name, .. } => {
                return Err(CompileError::UnsupportedNode(format!(
                    "generic function '{}' reached bytecode generation before monomorphization",
                    name
                )));
            }
            ASTNode::AsyncFuncDecl { name, .. } => {
                return Err(CompileError::UnsupportedNode(format!(
                    "async function '{}' is parsed but not executable yet",
                    name
                )));
            }
            ASTNode::SpawnStmt(_) => {
                return Err(CompileError::UnsupportedNode("spawn statements are not executable yet".into()));
            }
            ASTNode::DelayStmt(_) => {
                return Err(CompileError::UnsupportedNode("delay statements are not executable yet".into()));
            }
        }
        Ok(())
    }

    pub fn compile_expr(&mut self, expr: Expression) -> Result<()> {
        match expr {
            Expression::Literal(val) => {
                self.emit_u8(OpCode::PushConst as u8);
                match val {
                    crate::vm::Value::Int(n) => { self.emit_u8(0); self.bytecode.extend_from_slice(&n.to_be_bytes()); }
                    crate::vm::Value::Bool(b) => { self.emit_u8(1); self.emit_u8(if b { 1 } else { 0 }); }
                    crate::vm::Value::Str(s) => { self.emit_u8(2); self.emit_string(&s); }
                    crate::vm::Value::Float(f) => { self.emit_u8(3); self.bytecode.extend_from_slice(&f.to_be_bytes()); }
                    crate::vm::Value::Enum(ref name, _) => { self.emit_u8(4); self.emit_string(name); }
                    crate::vm::Value::Struct(ref fields) => {
                        self.emit_u8(5);
                        self.emit_u8(fields.len() as u8);
                        for (k, v) in fields {
                            self.emit_string(k);
                            self.emit_u8(OpCode::PushConst as u8);
                            // Recurse: encode the value inline
                            match v {
                                crate::vm::Value::Int(n) => { self.emit_u8(0); self.bytecode.extend_from_slice(&n.to_be_bytes()); }
                                crate::vm::Value::Bool(b) => { self.emit_u8(1); self.emit_u8(if *b { 1 } else { 0 }); }
                                crate::vm::Value::Str(s) => { self.emit_u8(2); self.emit_string(s); }
                                crate::vm::Value::Float(f) => { self.emit_u8(3); self.bytecode.extend_from_slice(&f.to_be_bytes()); }
                                _ => { self.emit_u8(0); self.bytecode.extend_from_slice(&0i64.to_be_bytes()); }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Expression::IntLiteral(n) => {
                self.emit_u8(OpCode::PushConst as u8);
                self.emit_u8(0);
                self.bytecode.extend_from_slice(&n.to_be_bytes());
            }
            Expression::Variable(name) => {
                self.emit_u8(OpCode::LoadVar as u8);
                self.emit_string(&name);
            }
            Expression::Binary { left, op, right } => {
                // च (and) short-circuit: compile left, JumpIfFalse → end, compile right
                if op.as_str() == "च" {
                    self.compile_expr(*left)?;
                    self.emit_u8(OpCode::JumpIfFalse as u8);
                    let end_placeholder = self.emit_u32_placeholder();
                    self.compile_expr(*right)?;
                    let end_pc = self.bytecode.len() as u32;
                    self.emit_u32_at(end_placeholder, end_pc);
                    return Ok(());
                }
                // वा (or) short-circuit: compile left, Dup + JumpIfTrue → join, compile right
                if op.as_str() == "वा" {
                    self.compile_expr(*left)?;
                    self.emit_u8(OpCode::Dup as u8);
                    self.emit_u8(OpCode::JumpIfTrue as u8);
                    let join_placeholder = self.emit_u32_placeholder();
                    self.emit_u8(OpCode::Drop as u8);
                    self.compile_expr(*right)?;
                    let join_pc = self.bytecode.len() as u32;
                    self.emit_u32_at(join_placeholder, join_pc);
                    return Ok(());
                }
                self.compile_expr(*left)?;
                self.compile_expr(*right)?;
                let opcode = match op.as_str() {
                    "+" => OpCode::Add,
                    "-" => OpCode::Sub,
                    "*" => OpCode::Mul,
                    "/" => OpCode::Div,
                    "==" => OpCode::Equal,
                    "!=" => OpCode::NotEqual,
                    "<" => OpCode::LessThan,
                    ">" => OpCode::GreaterThan,
                    "<=" => OpCode::LessEqual,
                    ">=" => OpCode::GreaterEqual,
                    _ => return Err(CompileError::UnsupportedOp(op)),
                };
                self.emit_u8(opcode as u8);
            }
            Expression::MatchExpr { eval_target, arms } => {
                // Compile target onto stack
                self.compile_expr(*eval_target)?;
                let mut end_placeholders = Vec::new();
                for (pattern_name, binds, body_expr) in arms {
                    // Dup target, extract variant name, compare with pattern
                    self.emit_u8(OpCode::Dup as u8);
                    self.emit_u8(OpCode::GetVariant as u8);
                    self.emit_u8(OpCode::PushConst as u8);
                    self.emit_u8(2); // string tag
                    self.emit_string(&pattern_name);
                    self.emit_u8(OpCode::Equal as u8);
                    self.emit_u8(OpCode::JumpIfFalse as u8);
                    let next_placeholder = self.emit_u32_placeholder();

                    // Match! The original target is still on stack; extract payload for bindings
                    self.emit_u8(OpCode::Extract as u8); // pop Enum, push payload
                    if !binds.is_empty() {
                        for bind_name in binds {
                            self.emit_u8(OpCode::Dup as u8);
                            self.emit_u8(OpCode::StoreVar as u8);
                            self.emit_string(&bind_name);
                        }
                        self.emit_u8(OpCode::Drop as u8); // remove extra copy
                    } else {
                        // No bindings: drop the payload, it won't be used
                        self.emit_u8(OpCode::Drop as u8);
                    }
                    self.compile_expr(body_expr)?;
                    self.emit_u8(OpCode::Jump as u8);
                    let end_placeholder = self.emit_u32_placeholder();
                    end_placeholders.push(end_placeholder);

                    // Patch JumpIfFalse destination
                    let next_pc = self.bytecode.len() as u32;
                    self.emit_u32_at(next_placeholder, next_pc);
                }
                // No arm matched: drop target, push null
                self.emit_u8(OpCode::Drop as u8);
                self.emit_u8(OpCode::PushConst as u8);
                self.emit_u8(0); // int tag = 0
                self.bytecode.extend_from_slice(&0i64.to_be_bytes());
                for ep in end_placeholders {
                    let end_pc = self.bytecode.len() as u32;
                    self.emit_u32_at(ep, end_pc);
                }
            }
            Expression::FieldAccess { object, field } => {
                self.compile_expr(*object)?;
                self.emit_u8(OpCode::GetField as u8);
                self.emit_string(&field);
            }
            Expression::MethodCall { instance, method_name, args } => {
                // Compile instance (self) first, then args
                self.compile_expr(*instance)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                // Emit as regular Call to method_name; VM treats self as first arg
                self.emit_u8(OpCode::Call as u8);
                let placeholder = self.emit_u32_placeholder();
                self.unresolved_calls.push((placeholder, method_name));
            }
            Expression::Call { name, args } => {
                // Compile args first; clone since we consume args below
                for arg in args.iter() { self.compile_expr(arg.clone())?; }
                if is_builtin(&name) {
                    self.emit_u8(OpCode::CallBuiltin as u8);
                    self.emit_string(&name);
                    self.emit_u8(args.len() as u8);
                } else if self.foreign_table.contains_key(&name) {
                    let (ref lib_path, ref sym_name, _nargs) = self.foreign_table[&name].clone();
                    self.emit_u8(OpCode::CallForeign as u8);
                    self.emit_string(lib_path);
                    self.emit_string(sym_name);
                    self.emit_u8(args.len() as u8);
                } else {
                    self.emit_u8(OpCode::Call as u8);
                    let placeholder = self.emit_u32_placeholder();
                    self.unresolved_calls.push((placeholder, name));
                }
            }
            Expression::GenericCall { name, .. } => {
                return Err(CompileError::UnsupportedOp(format!(
                    "generic call '{}' reached bytecode generation before monomorphization",
                    name
                )));
            }
            Expression::BorrowExpr { target, .. } => {
                return Err(CompileError::UnsupportedOp(format!(
                    "borrow expression for '{}' is not executable yet",
                    target
                )));
            }
            Expression::AwaitExpr { .. } => {
                return Err(CompileError::UnsupportedOp("await expressions are not executable yet".into()));
            }
        }
        Ok(())
    }

    pub fn link_unresolved_references(&mut self) -> Result<()> {
        let table = self.function_table.clone();
        let calls = self.unresolved_calls.clone();
        for &(placeholder, ref name) in &calls {
            if is_builtin(name) {
                continue;
            }
            if let Some(&address) = table.get(name) {
                self.emit_u32_at(placeholder, address);
            } else {
                return Err(CompileError::LinkError(format!("failed to locate function target '{}'", name)));
            }
        }
        Ok(())
    }

}
