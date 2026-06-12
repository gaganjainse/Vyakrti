use std::collections::HashMap;
use crate::ast::{ASTNode, Expression};
pub use crate::ast::AccessModifier;
use crate::vm::Value;

/// Kāraka (vibhakti case-ending) tags for argument binding.
/// Arguments can appear in any order; the type checker resolves
/// parameter slots by matching these tags rather than position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Karaka {
    Kartr,        // कर्तृ / _kartr       — nominative, subject/agent
    Karma,        // कर्म / _karma       — accusative, object
    Karana,       // करण / _karana       — instrumental, tool
    Sampradana,   // सम्प्रदान / _sampradana — dative, recipient
    Apadana,      // अपादान / _apadana    — ablative, source
    Sambandha,    // सम्बन्ध / _sambandha  — genitive, possession
    Adhikarana,   // अधिकरण / _adhikarana  — locative, location/context
    Default,      // no vibhakti suffix — positional fallback
}

/// Pairs of (suffix, Karaka) for Devanagari and transliterated suffixes.
const KARAKA_SUFFIXES: &[(&str, &str, Karaka)] = &[
    ("_कर्तृ",      "_kartr",       Karaka::Kartr),
    ("_कर्म",      "_karma",       Karaka::Karma),
    ("_करण",      "_karana",      Karaka::Karana),
    ("_सम्प्रदान", "_sampradana",  Karaka::Sampradana),
    ("_अपादान",    "_apadana",     Karaka::Apadana),
    ("_सम्बन्ध",   "_sambandha",   Karaka::Sambandha),
    ("_अधिकरण",   "_adhikarana",  Karaka::Adhikarana),
];

/// Extract Kāraka tag from an identifier name.
/// Returns (base_name, karaka).
pub fn extract_karaka(name: &str) -> (String, Karaka) {
    for (deva, slp1, karaka) in KARAKA_SUFFIXES {
        if let Some(base) = name.strip_suffix(deva) {
            return (base.to_string(), *karaka);
        }
        if let Some(base) = name.strip_suffix(slp1) {
            return (base.to_string(), *karaka);
        }
    }
    (name.to_string(), Karaka::Default)
}

/// Render a Kāraka as a human-readable label (for error messages).
pub fn karaka_label(k: &Karaka) -> &'static str {
    match k {
        Karaka::Kartr => "कर्तृ (agent/subject)",
        Karaka::Karma => "कर्म (object)",
        Karaka::Karana => "करण (instrument)",
        Karaka::Sampradana => "सम्प्रदान (recipient)",
        Karaka::Apadana => "अपादान (source)",
        Karaka::Sambandha => "सम्बन्ध (possession)",
        Karaka::Adhikarana => "अधिकरण (location)",
        Karaka::Default => "positional",
    }
}

/// Gana internal type encoding (3-bit type + 3-bit state, never user-facing).
/// Encoding: byte = (state << 3) | type_bits
/// Type bits (0-2): न=0(void), स=1(str), ज=2(bool), य=3(array), र=4(float), त=5(func), म=7(int)
/// State bits (3-5): भ=0(declared), र=1(assigned), त=2(in-use)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GanaType(u8);

impl GanaType {
    pub const VOID: u8 = 0;
    pub const STR: u8 = 1;
    pub const BOOL: u8 = 2;
    pub const ARRAY: u8 = 3;
    pub const FLOAT: u8 = 4;
    pub const FUNC: u8 = 5;
    pub const INT: u8 = 7;

    pub const DECLARED: u8 = 0;
    pub const ASSIGNED: u8 = 1;
    pub const IN_USE: u8 = 2;

    pub fn new(typ: u8) -> Self {
        GanaType((Self::ASSIGNED << 3) | typ)
    }

    pub fn typ(self) -> u8 { self.0 & 0x07 }
    pub fn state(self) -> u8 { (self.0 >> 3) & 0x07 }
    pub fn with_state(self, state: u8) -> Self { GanaType((state << 3) | self.typ()) }

    pub fn int() -> Self { Self::new(Self::INT) }
    pub fn bool() -> Self { Self::new(Self::BOOL) }
    pub fn str() -> Self { Self::new(Self::STR) }
    pub fn float() -> Self { Self::new(Self::FLOAT) }
    pub fn void() -> Self { Self::new(Self::VOID) }
    pub fn func() -> Self { Self::new(Self::FUNC) }
    pub fn array() -> Self { Self::new(Self::ARRAY) }
}

/// Resolved type information for a symbol.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    Known(GanaType),
    UserDefined(String),
    Unknown,
}

/// A symbol table entry.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub base_name: String,
    pub karaka: Karaka,
    pub resolved_type: ResolvedType,
    pub depth: usize,
    pub is_function: bool,
    pub param_count: usize,
    /// Parameter info: (full_param_name, base_name, karaka, type)
    pub param_types: Vec<(String, String, Karaka, ResolvedType)>,
    pub access_level: crate::ast::AccessModifier,
}

/// Hierarchical scope-aware symbol table.
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    pub current_depth: usize,
    errors: Vec<String>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut global = HashMap::new();
        let builtin_types: Vec<(&str, &str, GanaType)> = vec![
            ("सत", "सत", GanaType::bool()),
            ("असत", "असत", GanaType::bool()),
            ("मुद्रण", "मुद्रण", GanaType::void()),
            ("दैर्घ्यम्", "दैर्घ्यम्", GanaType::int()),
            ("प्रकारः", "प्रकारः", GanaType::str()),
            ("योजन", "योजन", GanaType::str()),
            ("अस्ति", "अस्ति", GanaType::bool()),
            ("संख्या", "संख्या", GanaType::int()),
            ("रूपान्तर", "रूपान्तर", GanaType::str()),
        ];
        for &(name, base, gana) in &builtin_types {
            global.insert(name.into(), Symbol {
                name: name.into(), base_name: base.into(),
                karaka: Karaka::Default, resolved_type: ResolvedType::Known(gana),
                depth: 0, is_function: true, param_count: 1, param_types: vec![],
                access_level: AccessModifier::Public,
            });
        }
        SymbolTable { scopes: vec![global], current_depth: 0, errors: Vec::new() }
    }

    pub fn push_scope(&mut self) {
        self.current_depth += 1;
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        self.current_depth = if self.current_depth > 0 { self.current_depth - 1 } else { 0 };
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, symbol);
        }
    }

    pub fn update_state(&mut self, name: &str, new_state: u8) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.get_mut(name) {
                if let ResolvedType::Known(ref mut gana) = sym.resolved_type {
                    *gana = gana.with_state(new_state);
                }
                return;
            }
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn lookup_base(&self, base_name: &str, karaka: Karaka) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            for sym in scope.values() {
                if sym.base_name == base_name && sym.karaka == karaka {
                    return Some(sym);
                }
            }
        }
        None
    }

    pub fn push_error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    pub fn take_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.errors)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Total count of live symbol entries across all scopes.
    pub fn symbol_count(&self) -> usize {
        self.scopes.iter().map(|s| s.len()).sum()
    }
}

/// Resolve function call arguments by kāraka tags.
/// Parameters with kāraka suffixes are matched by tag rather than position.
/// Parameters/arguments without kāraka use positional fallback (as confirmed).
pub fn resolve_args_by_karaka(
    param_types: &[(String, String, Karaka, ResolvedType)],
    args: &[Expression],
) -> Result<Vec<Expression>, String> {
    let n = param_types.len();
    if args.len() != n {
        return Err(format!("expected {} arguments, got {}", n, args.len()));
    }

    // Separate args into kāraka-tagged and positional
    let mut karaka_args: Vec<(Karaka, usize)> = Vec::new(); // (karaka, arg_index)
    let mut positional_arg_indices: Vec<usize> = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        if let Expression::Variable(name) = arg {
            let (_, karaka) = extract_karaka(name);
            if karaka != Karaka::Default {
                karaka_args.push((karaka, i));
                continue;
            }
        }
        positional_arg_indices.push(i);
    }

    // Build mapping: param_index → arg_index
    let mut param_to_arg: Vec<Option<usize>> = vec![None; n];
    let mut used_params: Vec<bool> = vec![false; n];

    // First pass: match kāraka-tagged arguments to parameters
    for (karaka, arg_idx) in &karaka_args {
        let mut matched = false;
        for (p_idx, (_, _base, p_karaka, _)) in param_types.iter().enumerate() {
            if !used_params[p_idx] && p_karaka == karaka {
                param_to_arg[p_idx] = Some(*arg_idx);
                used_params[p_idx] = true;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(format!(
                "no parameter with kāraka {:?} found for argument {}",
                karaka, arg_idx
            ));
        }
    }

    // Second pass: positional fallback for remaining parameters
    let mut pos_iter = positional_arg_indices.into_iter();
    for p_idx in 0..n {
        if !used_params[p_idx] {
            match pos_iter.next() {
                Some(arg_idx) => {
                    param_to_arg[p_idx] = Some(arg_idx);
                }
                None => {
                    return Err(format!(
                        "not enough positional arguments for parameter '{}'",
                        param_types[p_idx].0
                    ));
                }
            }
        }
    }

    // Reorder args according to param order
    let mut reordered = Vec::with_capacity(n);
    for arg_idx in param_to_arg.iter().take(n).flatten() {
        reordered.push(args[*arg_idx].clone());
    }
    Ok(reordered)
}

/// Resolves the type of an expression (returns a string representation).
/// Convert a Devanagari type name to its Gana encoding.
/// Falls back to GanaType::void() for unknown names.
fn gana_from_str(s: &str) -> GanaType {
    match s {
        "अङ्क" => GanaType::int(),
        "सत्यता" => GanaType::bool(),
        "शब्द" => GanaType::str(),
        "दशमलव" => GanaType::float(),
        "शून्य" => GanaType::void(),
        _ => GanaType::void(),
    }
}

/// Extract just the type bits from a ResolvedType for comparison (ignores state).
fn gana_type_bits(t: &ResolvedType) -> Option<u8> {
    match t {
        ResolvedType::Known(g) => Some(g.typ()),
        _ => None,
    }
}

fn expr_type(expr: &Expression, table: &SymbolTable) -> ResolvedType {
    match expr {
        Expression::Literal(v) => match v {
            Value::Int(_) => ResolvedType::Known(GanaType::int()),
            Value::Float(_) => ResolvedType::Known(GanaType::float()),
            Value::Bool(_) => ResolvedType::Known(GanaType::bool()),
            Value::Str(_) => ResolvedType::Known(GanaType::str()),
            _ => ResolvedType::Unknown,
        },
        Expression::IntLiteral(_) => ResolvedType::Known(GanaType::int()),
        Expression::Variable(name) => {
            let (base, karaka) = extract_karaka(name);
            table.lookup_base(&base, karaka)
                .or_else(|| table.lookup(name))
                .or_else(|| table.lookup(&base))
                .map(|s| s.resolved_type.clone())
                .unwrap_or(ResolvedType::Unknown)
        }
        Expression::Binary { left, op, right } => {
            let lt = expr_type(left, table);
            let rt = expr_type(right, table);
            match op.as_str() {
                "+" | "-" | "*" | "/" | "%" => {
                    let lt_bits = gana_type_bits(&lt);
                    let rt_bits = gana_type_bits(&rt);
                    if lt_bits == Some(GanaType::INT) && rt_bits == Some(GanaType::INT) {
                        ResolvedType::Known(GanaType::int())
                    } else if lt_bits == Some(GanaType::FLOAT) || rt_bits == Some(GanaType::FLOAT) {
                        ResolvedType::Known(GanaType::float())
                    } else {
                        ResolvedType::Unknown
                    }
                }
                "==" | "!=" | "<" | ">" | "<=" | ">=" => ResolvedType::Known(GanaType::bool()),
                _ => ResolvedType::Unknown,
            }
        }
        Expression::Call { name, .. } => {
            table.lookup(name).map(|s| s.resolved_type.clone()).unwrap_or(ResolvedType::Unknown)
        }
        _ => ResolvedType::Unknown,
    }
}

/// The Kāraka-aware semantic analyzer and type checker.
pub struct TypeChecker {
    pub table: SymbolTable,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker { table: SymbolTable::new() }
    }

    pub fn check_program(&mut self, nodes: Vec<ASTNode>) -> Result<Vec<ASTNode>, Vec<String>> {
        let mut rewritten = Vec::new();
        for node in nodes {
            if let Ok(n) = self.check_node(node) {
                rewritten.push(n);
            }
        }
        if self.table.has_errors() {
            Err(self.table.take_errors())
        } else {
            Ok(rewritten)
        }
    }

    fn check_node(&mut self, node: ASTNode) -> Result<ASTNode, ()> {
        match node {
            ASTNode::VarDecl { name, data_type, value, access_level } => {
                let val_type = expr_type(&value, &self.table);
                let declared = data_type.clone().unwrap_or_default();

                if !declared.is_empty() && val_type != ResolvedType::Unknown {
                    let expected_bits = Some(gana_from_str(&declared).typ());
                    let val_bits = gana_type_bits(&val_type);
                    if val_bits.is_some() && val_bits != expected_bits {
                        self.table.push_error(format!(
                            "type mismatch: variable '{}' declared as '{}' but value has type '{:?}'",
                            name, declared, val_type
                        ));
                    }
                }

                let resolved = if !declared.is_empty() {
                    ResolvedType::Known(gana_from_str(&declared))
                } else {
                    val_type
                };

                self.table.insert(name.clone(), Symbol {
                    name: name.clone(), base_name: name.clone(),
                    karaka: Karaka::Default, resolved_type: resolved,
                    depth: self.table.current_depth, is_function: false,
                    param_count: 0, param_types: vec![],
                    access_level,
                });
                let checked_value = self.check_expr(value)?;
                Ok(ASTNode::VarDecl { name, data_type, value: checked_value, access_level })
            }

            ASTNode::FuncDecl { name, parameters, return_type, body, access_level } => {
                let param_types: Vec<(String, String, Karaka, ResolvedType)> = parameters.iter().map(|(n, t)| {
                    let (base, karaka) = extract_karaka(n);
                    (n.clone(), base, karaka, ResolvedType::Known(gana_from_str(t)))
                }).collect();

                let resolved_return_type = ResolvedType::Known(gana_from_str(&return_type));
                self.table.insert(name.clone(), Symbol {
                    name: name.clone(), base_name: name.clone(),
                    karaka: Karaka::Default, resolved_type: resolved_return_type,
                    depth: self.table.current_depth, is_function: true,
                    param_count: parameters.len(),
                    param_types: param_types.clone(),
                    access_level,
                });

                self.table.push_scope();
                for ((p_name, p_type), (_, _base, karaka, _)) in parameters.iter().zip(param_types.iter()) {
                    let base_name = _base.clone();
                    self.table.insert(p_name.clone(), Symbol {
                        name: p_name.clone(), base_name,
                        karaka: *karaka, resolved_type: ResolvedType::Known(gana_from_str(p_type)),
                        depth: self.table.current_depth, is_function: false,
                        param_count: 0, param_types: vec![],
                        access_level: AccessModifier::Private,
                    });
                }
                let mut checked_body = Vec::new();
                for stmt in body {
                    if let Ok(s) = self.check_node(stmt) { checked_body.push(s); }
                }
                self.table.pop_scope();

                Ok(ASTNode::FuncDecl { name, parameters, return_type, body: checked_body, access_level })
            }

            ASTNode::IfStmt { condition, then_branch, else_branch } => {
                let cond = self.check_expr(condition)?;
                let cond_type = expr_type(&cond, &self.table);
                if cond_type != ResolvedType::Unknown && gana_type_bits(&cond_type) != Some(GanaType::BOOL) {
                    self.table.push_error(format!(
                        "if condition must be boolean (सत्यता), got '{:?}'", cond_type
                    ));
                }
                self.table.push_scope();
                let mut then_checked = Vec::new();
                for s in then_branch { if let Ok(st) = self.check_node(s) { then_checked.push(st); } }
                self.table.pop_scope();

                let mut else_checked = None;
                if let Some(eb) = else_branch {
                    self.table.push_scope();
                    let mut eb_checked = Vec::new();
                    for s in eb { if let Ok(st) = self.check_node(s) { eb_checked.push(st); } }
                    self.table.pop_scope();
                    else_checked = Some(eb_checked);
                }
                Ok(ASTNode::IfStmt { condition: cond, then_branch: then_checked, else_branch: else_checked })
            }

            ASTNode::WhileStmt { condition, body } => {
                let cond = self.check_expr(condition)?;
                self.table.push_scope();
                let mut checked = Vec::new();
                for s in body { if let Ok(st) = self.check_node(s) { checked.push(st); } }
                self.table.pop_scope();
                Ok(ASTNode::WhileStmt { condition: cond, body: checked })
            }

            ASTNode::StatementExpr(expr) => {
                let checked = self.check_expr(expr)?;
                Ok(ASTNode::StatementExpr(checked))
            }
            ASTNode::ReturnStmt(expr) => {
                let checked = self.check_expr(expr)?;
                Ok(ASTNode::ReturnStmt(checked))
            }

            ASTNode::Block(stmts) => {
                self.table.push_scope();
                let mut checked = Vec::new();
                for s in stmts { if let Ok(st) = self.check_node(s) { checked.push(st); } }
                self.table.pop_scope();
                Ok(ASTNode::Block(checked))
            }

            ASTNode::EnumDecl { name, variants, access_level } => {
                let v_names: Vec<String> = variants.iter().map(|(vn, _)| vn.clone()).collect();
                self.table.insert(name.clone(), Symbol {
                    name: name.clone(), base_name: name.clone(),
                    karaka: Karaka::Default, resolved_type: ResolvedType::UserDefined("रूपभेदः".into()),
                    depth: self.table.current_depth, is_function: false,
                    param_count: 0, param_types: vec![],
                    access_level,
                });
                for vn in &v_names {
                    self.table.insert(vn.clone(), Symbol {
                        name: vn.clone(), base_name: vn.clone(),
                        karaka: Karaka::Default, resolved_type: ResolvedType::UserDefined(format!("{}::{}", name, vn)),
                        depth: self.table.current_depth, is_function: false,
                        param_count: 0, param_types: vec![],
                        access_level: AccessModifier::Private,
                    });
                }
                Ok(ASTNode::EnumDecl { name, variants, access_level })
            }

            ASTNode::StructDecl { name, attributes, fields, access_level } => {
                self.table.insert(name.clone(), Symbol {
                    name: name.clone(), base_name: name.clone(),
                    karaka: Karaka::Default, resolved_type: ResolvedType::UserDefined("वस्तु_विन्यासः".into()),
                    depth: self.table.current_depth, is_function: false,
                    param_count: 0, param_types: vec![],
                    access_level,
                });
                Ok(ASTNode::StructDecl { name, attributes, fields, access_level })
            }

            _ => Ok(node),
        }
    }

    fn check_expr(&mut self, expr: Expression) -> Result<Expression, ()> {
        match expr {
            Expression::Variable(name) => {
                let (base, karaka) = extract_karaka(&name);
                let exists = self.table.lookup(&name).is_some()
                    || self.table.lookup(&base).is_some()
                    || self.table.lookup_base(&base, karaka).is_some();
                if !exists {
                    self.table.push_error(format!("undefined variable '{}'", name));
                }
                // State machine: reject DECLARED reads, transition ASSIGNED → IN_USE
                let declared = self.table.lookup(&name).is_some_and(|s| {
                    matches!(s.resolved_type, ResolvedType::Known(g) if g.state() == GanaType::DECLARED)
                });
                if declared {
                    self.table.push_error(format!("variable '{}' is declared but not assigned", name));
                }
                if self.table.lookup(&name).is_some() {
                    self.table.update_state(&name, GanaType::IN_USE);
                }
                Ok(Expression::Variable(name))
            }

            Expression::Binary { left, op, right } => {
                let l = self.check_expr(*left)?;
                let r = self.check_expr(*right)?;
                let lt = expr_type(&l, &self.table);
                let rt = expr_type(&r, &self.table);
                let numeric = |t: &ResolvedType| -> bool {
                    let bits = gana_type_bits(t);
                    bits == Some(GanaType::INT) || bits == Some(GanaType::FLOAT)
                };
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" => {
                        if lt != ResolvedType::Unknown && rt != ResolvedType::Unknown
                            && !numeric(&lt) && !numeric(&rt)
                        {
                            self.table.push_error(format!(
                                "operator '{}' requires numeric operands, got '{:?}' and '{:?}'",
                                op, lt, rt
                            ));
                        }
                    }
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                        let lt_bits = gana_type_bits(&lt);
                        let rt_bits = gana_type_bits(&rt);
                        if lt_bits != rt_bits && lt != ResolvedType::Unknown && rt != ResolvedType::Unknown {
                            self.table.push_error(format!(
                                "comparison between incompatible types '{:?}' and '{:?}'", lt, rt
                            ));
                        }
                    }
                    _ => {}
                }
                Ok(Expression::Binary { left: Box::new(l), op, right: Box::new(r) })
            }

            Expression::Call { name, args } => {
                let func_sym = self.table.lookup(&name).cloned();
                let func_exists = func_sym.is_some();
                if !func_exists {
                    self.table.push_error(format!("undefined function '{}'", name));
                    let mut checked_args = Vec::new();
                    for arg in args {
                        checked_args.push(self.check_expr(arg)?);
                    }
                    return Ok(Expression::Call { name, args: checked_args });
                }

                let sym = func_sym.unwrap();
                let expected = sym.param_count;
                if args.len() != expected {
                    self.table.push_error(format!(
                        "function '{}' expects {} arguments, got {}", name, expected, args.len()
                    ));
                }

                // Resolve arguments by kāraka tags (order-free binding with positional fallback)
                // Skip for builtins (empty param_types) — they use positional args directly
                let resolved_args = if sym.param_types.is_empty() {
                    args.to_vec()
                } else {
                    resolve_args_by_karaka(&sym.param_types, &args)
                        .unwrap_or_else(|e| {
                            self.table.push_error(format!("in call to '{}': {}", name, e));
                            args.to_vec()
                        })
                };

                let mut checked_args = Vec::new();
                for arg in resolved_args {
                    checked_args.push(self.check_expr(arg)?);
                }

                Ok(Expression::Call { name, args: checked_args })
            }

            Expression::MethodCall { instance, method_name, args } => {
                let i = self.check_expr(*instance)?;
                let mut checked = Vec::new();
                for a in args { checked.push(self.check_expr(a)?); }
                Ok(Expression::MethodCall { instance: Box::new(i), method_name, args: checked })
            }

            Expression::MatchExpr { eval_target, arms } => {
                let target = self.check_expr(*eval_target)?;
                let mut checked_arms = Vec::new();
                for (variant, binds, arm_expr) in arms {
                    let checked_arm = self.check_expr(arm_expr)?;
                    checked_arms.push((variant, binds, checked_arm));
                }
                Ok(Expression::MatchExpr { eval_target: Box::new(target), arms: checked_arms })
            }

            Expression::Literal(_) | Expression::IntLiteral(_) => Ok(expr),

            _ => Ok(expr),
        }
    }
}
