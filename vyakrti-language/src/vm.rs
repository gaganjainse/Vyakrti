use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    PushConst   = 0x01,
    LoadVar     = 0x02,
    StoreVar    = 0x03,
    Dup         = 0x04,
    Drop        = 0x05,
    GetVariant  = 0x06,
    Add         = 0x07,
    Sub         = 0x08,
    Mul         = 0x09,
    Div         = 0x0A,
    Equal       = 0x0B,
    NotEqual    = 0x0C,
    LessThan    = 0x0D,
    GreaterThan = 0x0E,
    LessEqual   = 0x0F,
    GreaterEqual= 0x10,
    Jump        = 0x11,
    JumpIfFalse = 0x12,
    Call        = 0x13,
    Return      = 0x14,
    FAdd        = 0x15,
    FSub        = 0x16,
    FMul        = 0x17,
    FDiv        = 0x18,
    CallBuiltin = 0x19,
    Extract     = 0x1A,
    MakeEnum    = 0x1B,
    GetField    = 0x1C,
    SetField    = 0x1D,
    MakeStruct  = 0x1E,
    CallForeign = 0x1F,
    JumpIfTrue  = 0x20,
    BindParams  = 0x21,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Func(u32),
    Enum(String, Box<Value>),
    Struct(HashMap<String, Value>),
}

pub type BuiltinFn = fn(&mut VirtualMachine, &[Value]) -> Result<Value, VmError>;

#[derive(Debug)]
pub struct VmError(pub String);

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct VirtualMachine {
    data_stack: Vec<Value>,
    call_stack: Vec<(usize, HashMap<String, Value>)>,
    local_frames: Vec<HashMap<String, Value>>,
    pub globals: HashMap<String, Value>,
    pub builtins: HashMap<String, BuiltinFn>,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachine {
    pub fn new() -> Self {
        let mut vm = VirtualMachine {
            data_stack: Vec::new(),
            call_stack: Vec::new(),
            local_frames: Vec::new(),
            globals: HashMap::new(),
            builtins: HashMap::new(),
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        // मुद्रण — pops top of stack and prints
        self.add_builtin("मुद्रण", |vm, args| {
            let val = if args.is_empty() { vm.pop()? } else { args[0].clone() };
            match &val {
                Value::Int(n) => println!("{}", n),
                Value::Float(f) => println!("{}", f),
                Value::Bool(b) => println!("{}", if *b { "सत" } else { "असत" }),
                Value::Str(s) => println!("{}", s),
                Value::Enum(name, _) => println!("{}", name),
                Value::Struct(fields) => println!("{{ {} }}", fields.keys().cloned().collect::<Vec<_>>().join(", ")),
                _ => println!("(null)"),
            }
            Ok(Value::Null)
        });

        // दैर्घ्यम् — pops a string, pushes its length as Int
        self.add_builtin("दैर्घ्यम्", |_vm, args| {
            if args.is_empty() {
                return Err(VmError("दैर्घ्यम् expects 1 argument".into()));
            }
            match &args[0] {
                Value::Str(s) => Ok(Value::Int(s.len() as i64)),
                _ => Err(VmError("दैर्घ्यम् expects a string argument".into())),
            }
        });

        // प्रकारः — pops a value, pushes a type-name string
        self.add_builtin("प्रकारः", |_vm, args| {
            if args.is_empty() {
                return Err(VmError("प्रकारः expects 1 argument".into()));
            }
            let type_name = match &args[0] {
                Value::Int(_) => "अङ्क",
                Value::Float(_) => "दशमलव",
                Value::Bool(_) => "सत्यता",
                Value::Str(_) => "शब्द",
                Value::Func(_) => "कार्य",
                Value::Enum(name, _) => name,
                Value::Struct(_) => "वस्तु_विन्यासः",
                Value::Null => "शून्य",
            };
            Ok(Value::Str(type_name.to_string()))
        });

        // String utility builtins
        self.add_builtin("योजन", |_vm, args| {
            if args.len() < 2 { return Err(VmError("योजन expects 2 arguments".into())); }
            match (&args[0], &args[1]) {
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::Str(a), other) => Ok(Value::Str(format!("{}{:?}", a, other))),
                _ => Ok(Value::Str(format!("{:?}{:?}", args[0], args[1]))),
            }
        });
        self.add_builtin("अस्ति", |_vm, args| {
            if args.len() < 2 { return Err(VmError("अस्ति expects 2 arguments".into())); }
            match (&args[0], &args[1]) {
                (Value::Str(haystack), Value::Str(needle)) => Ok(Value::Bool(haystack.contains(needle.as_str()))),
                _ => Err(VmError("अस्ति expects string arguments".into())),
            }
        });
        self.add_builtin("संख्या", |_vm, args| {
            if args.is_empty() { return Err(VmError("संख्या expects 1 argument".into())); }
            match &args[0] {
                Value::Str(s) => s.trim().parse::<i64>().map(Value::Int).or(Ok(Value::Int(0))),
                Value::Int(n) => Ok(Value::Int(*n)),
                _ => Err(VmError("संख्या expects a string or int".into())),
            }
        });
        self.add_builtin("रूपान्तर", |_vm, args| {
            if args.is_empty() { return Err(VmError("रूपान्तर expects 1 argument".into())); }
            Ok(Value::Str(format!("{:?}", args[0])))
        });
    }

    fn add_builtin(&mut self, name: &str, f: BuiltinFn) {
        self.builtins.insert(name.to_string(), f);
    }

    pub fn is_builtin_name(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    fn read_u16(&self, bytecode: &[u8], pc: &mut usize) -> u16 {
        let hi = bytecode[*pc] as u16;
        let lo = bytecode[*pc + 1] as u16;
        *pc += 2;
        (hi << 8) | lo
    }

    fn read_string(&self, bytecode: &[u8], pc: &mut usize) -> String {
        let len = self.read_u16(bytecode, pc) as usize;
        let s = String::from_utf8_lossy(&bytecode[*pc..*pc + len]).into_owned();
        *pc += len;
        s
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.data_stack.pop().ok_or(VmError("stack underflow".into()))
    }

    fn expect_int(pair: (Value, Value)) -> Result<(i64, i64), VmError> {
        match pair {
            (Value::Int(b), Value::Int(a)) => Ok((a, b)),
            _ => Err(VmError("expected integer operands".into())),
        }
    }

    fn expect_float(pair: (Value, Value)) -> Result<(f64, f64), VmError> {
        match pair {
            (Value::Float(b), Value::Float(a)) => Ok((a, b)),
            (Value::Int(b), Value::Int(a)) => Ok((a as f64, b as f64)),
            (Value::Float(b), Value::Int(a)) => Ok((a as f64, b)),
            (Value::Int(b), Value::Float(a)) => Ok((a, b as f64)),
            _ => Err(VmError("expected numeric operands".into())),
        }
    }

    fn pop2(&mut self) -> Result<(Value, Value), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        Ok((a, b))
    }

    fn load_var(&self, name: &str) -> Value {
        for frame in self.local_frames.iter().rev() {
            if let Some(value) = frame.get(name) {
                return value.clone();
            }
        }
        self.globals.get(name).cloned().unwrap_or(Value::Null)
    }

    fn store_var(&mut self, name: String, value: Value) {
        if let Some(frame) = self.local_frames.last_mut() {
            frame.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    pub fn run(&mut self, bytecode: &[u8], mut pc: usize) -> Result<(), VmError> {
        let mut instruction_count: u64 = 0;
        let max_instructions: u64 = 10_000_000; // Prevent infinite loops
        while pc < bytecode.len() {
            instruction_count += 1;
            if instruction_count > max_instructions {
                return Err(VmError(format!(
                    "execution limit exceeded ({} instructions) — possible infinite loop",
                    max_instructions
                )));
            }
            let instruction = bytecode[pc];
            pc += 1;

            if instruction == OpCode::PushConst as u8 {
                let type_tag = bytecode[pc]; pc += 1;
                match type_tag {
                    0 => {
                        let mut bytes = [0; 8];
                        bytes.copy_from_slice(&bytecode[pc..pc + 8]); pc += 8;
                        self.data_stack.push(Value::Int(i64::from_be_bytes(bytes)));
                    }
                    1 => { self.data_stack.push(Value::Bool(bytecode[pc] == 1)); pc += 1; }
                    2 => { let s = self.read_string(bytecode, &mut pc); self.data_stack.push(Value::Str(s)); }
                    3 => {
                        let mut bytes = [0; 8];
                        bytes.copy_from_slice(&bytecode[pc..pc + 8]); pc += 8;
                        self.data_stack.push(Value::Float(f64::from_be_bytes(bytes)));
                    }
                    4 => {
                        let name = self.read_string(bytecode, &mut pc);
                        self.data_stack.push(Value::Enum(name, Box::new(Value::Null)));
                    }
                    5 => {
                        let field_count = bytecode[pc] as usize; pc += 1;
                        let mut fields = HashMap::new();
                        for _ in 0..field_count {
                            let k = self.read_string(bytecode, &mut pc);
                            // PushConst for value
                            let _tag = bytecode[pc]; pc += 1; // skip PushConst opcode
                            let type_tag = bytecode[pc]; pc += 1;
                            let v = match type_tag {
                                0 => { let mut bytes = [0; 8]; bytes.copy_from_slice(&bytecode[pc..pc + 8]); pc += 8; Value::Int(i64::from_be_bytes(bytes)) }
                                1 => { let b = bytecode[pc] == 1; pc += 1; Value::Bool(b) }
                                2 => { let s = self.read_string(bytecode, &mut pc); Value::Str(s) }
                                3 => { let mut bytes = [0; 8]; bytes.copy_from_slice(&bytecode[pc..pc + 8]); pc += 8; Value::Float(f64::from_be_bytes(bytes)) }
                                _ => Value::Null,
                            };
                            fields.insert(k, v);
                        }
                        self.data_stack.push(Value::Struct(fields));
                    }
                    _ => {}
                }
            } else if instruction == OpCode::LoadVar as u8 {
                let name = self.read_string(bytecode, &mut pc);
                let val = self.load_var(&name);
                self.data_stack.push(val);
            } else if instruction == OpCode::StoreVar as u8 {
                let name = self.read_string(bytecode, &mut pc);
                let val = self.pop()?;
                self.store_var(name, val);
            } else if instruction == OpCode::Dup as u8 {
                let val = self.data_stack.last().cloned().ok_or(VmError("stack underflow on dup".into()))?;
                self.data_stack.push(val);
            } else if instruction == OpCode::Drop as u8 {
                self.pop()?;
            } else if instruction == OpCode::GetVariant as u8 {
                let val = self.pop()?;
                let name = match val {
                    Value::Enum(n, _) => n,
                    other => format!("{:?}", other),
                };
                self.data_stack.push(Value::Str(name));
            } else if instruction == OpCode::Extract as u8 {
                // Pop an Enum value and push just its payload (unwrap the box)
                let val = self.pop()?;
                let payload = match val {
                    Value::Enum(_, payload) => *payload,
                    other => other,
                };
                self.data_stack.push(payload);
            } else if instruction == OpCode::MakeEnum as u8 {
                // Pop payload, pop name string, push Enum(name, payload)
                let payload = self.pop()?;
                let name = match self.pop()? {
                    Value::Str(s) => s,
                    _ => return Err(VmError("MakeEnum: expected string name".into())),
                };
                self.data_stack.push(Value::Enum(name, Box::new(payload)));
            } else if instruction == OpCode::Add as u8 {
                let pair = self.pop2()?;
                if let Ok((a, b)) = Self::expect_int(pair) {
                    self.data_stack.push(Value::Int(a + b));
                }
            } else if instruction == OpCode::Sub as u8 {
                let pair = self.pop2()?;
                if let Ok((a, b)) = Self::expect_int(pair) {
                    self.data_stack.push(Value::Int(a - b));
                }
            } else if instruction == OpCode::Mul as u8 {
                let pair = self.pop2()?;
                if let Ok((a, b)) = Self::expect_int(pair) {
                    self.data_stack.push(Value::Int(a * b));
                }
            } else if instruction == OpCode::Div as u8 {
                let pair = self.pop2()?;
                if let Ok((a, b)) = Self::expect_int(pair) {
                    if b == 0 { return Err(VmError("division by zero".into())); }
                    self.data_stack.push(Value::Int(a / b));
                }
            } else if instruction == OpCode::FAdd as u8 {
                let pair = self.pop2()?;
                let (a, b) = Self::expect_float(pair)?;
                self.data_stack.push(Value::Float(a + b));
            } else if instruction == OpCode::FSub as u8 {
                let pair = self.pop2()?;
                let (a, b) = Self::expect_float(pair)?;
                self.data_stack.push(Value::Float(a - b));
            } else if instruction == OpCode::FMul as u8 {
                let pair = self.pop2()?;
                let (a, b) = Self::expect_float(pair)?;
                self.data_stack.push(Value::Float(a * b));
            } else if instruction == OpCode::FDiv as u8 {
                let pair = self.pop2()?;
                let (a, b) = Self::expect_float(pair)?;
                if b == 0.0 { return Err(VmError("float division by zero".into())); }
                self.data_stack.push(Value::Float(a / b));
            } else if instruction == OpCode::Equal as u8 {
                let (a, b) = self.pop2()?;
                self.data_stack.push(Value::Bool(a == b));
            } else if instruction == OpCode::NotEqual as u8 {
                let (a, b) = self.pop2()?;
                self.data_stack.push(Value::Bool(a != b));
            } else if instruction == OpCode::LessThan as u8 {
                let (a, b) = self.pop2()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a < b)),
                    (Value::Float(a), Value::Float(b)) => self.data_stack.push(Value::Bool(a < b)),
                    (Value::Float(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a < b as f64)),
                    (Value::Int(a), Value::Float(b)) => self.data_stack.push(Value::Bool((a as f64) < b)),
                    _ => self.data_stack.push(Value::Bool(false)),
                }
            } else if instruction == OpCode::GreaterThan as u8 {
                let (a, b) = self.pop2()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a > b)),
                    (Value::Float(a), Value::Float(b)) => self.data_stack.push(Value::Bool(a > b)),
                    (Value::Float(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a > b as f64)),
                    (Value::Int(a), Value::Float(b)) => self.data_stack.push(Value::Bool((a as f64) > b)),
                    _ => self.data_stack.push(Value::Bool(false)),
                }
            } else if instruction == OpCode::LessEqual as u8 {
                let (a, b) = self.pop2()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a <= b)),
                    (Value::Float(a), Value::Float(b)) => self.data_stack.push(Value::Bool(a <= b)),
                    (Value::Float(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a <= b as f64)),
                    (Value::Int(a), Value::Float(b)) => self.data_stack.push(Value::Bool((a as f64) <= b)),
                    _ => self.data_stack.push(Value::Bool(false)),
                }
            } else if instruction == OpCode::GreaterEqual as u8 {
                let (a, b) = self.pop2()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a >= b)),
                    (Value::Float(a), Value::Float(b)) => self.data_stack.push(Value::Bool(a >= b)),
                    (Value::Float(a), Value::Int(b)) => self.data_stack.push(Value::Bool(a >= b as f64)),
                    (Value::Int(a), Value::Float(b)) => self.data_stack.push(Value::Bool((a as f64) >= b)),
                    _ => self.data_stack.push(Value::Bool(false)),
                }
            } else if instruction == OpCode::Jump as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]);
                pc = u32::from_be_bytes(bytes) as usize;
            } else if instruction == OpCode::JumpIfFalse as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]);
                let dest = u32::from_be_bytes(bytes) as usize;
                if let Value::Bool(false) = self.pop()? { pc = dest; } else { pc += 4; }
            } else if instruction == OpCode::JumpIfTrue as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]);
                let dest = u32::from_be_bytes(bytes) as usize;
                if let Value::Bool(true) = self.pop()? { pc = dest; } else { pc += 4; }
            } else if instruction == OpCode::Call as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]);
                let dest = u32::from_be_bytes(bytes) as usize; pc += 4;
                let frame = HashMap::new();
                self.call_stack.push((pc, frame));
                self.local_frames.push(HashMap::new());
                pc = dest;
            } else if instruction == OpCode::Return as u8 {
                if let Some((ret_pc, _local_frame)) = self.call_stack.pop() {
                    self.local_frames.pop();
                    pc = ret_pc;
                } else {
                    break;
                }
            } else if instruction == OpCode::GetField as u8 {
                let field_name = self.read_string(bytecode, &mut pc);
                let val = self.pop()?;
                match val {
                    Value::Struct(fields) => {
                        self.data_stack.push(fields.get(&field_name).cloned().unwrap_or(Value::Null));
                    }
                    other => {
                        self.data_stack.push(other);
                    }
                }
            } else if instruction == OpCode::SetField as u8 {
                let field_name = self.read_string(bytecode, &mut pc);
                let val = self.pop()?;
                let mut struct_val = self.pop()?;
                if let Value::Struct(ref mut fields) = struct_val {
                    fields.insert(field_name, val);
                }
                self.data_stack.push(struct_val);
            } else if instruction == OpCode::MakeStruct as u8 {
                let field_count = bytecode[pc] as usize; pc += 1;
                let mut fields = HashMap::new();
                for _ in 0..field_count {
                    let name = self.read_string(bytecode, &mut pc);
                    let val = self.pop()?;
                    fields.insert(name, val);
                }
                self.data_stack.push(Value::Struct(fields));
            } else if instruction == OpCode::CallForeign as u8 {
                let lib_path = self.read_string(bytecode, &mut pc);
                let sym_name = self.read_string(bytecode, &mut pc);
                let nargs = bytecode[pc] as usize; pc += 1;
                let mut args = Vec::with_capacity(nargs);
                for _ in 0..nargs {
                    args.push(self.pop()?);
                }
                args.reverse();
                // Basic FFI: load library, look up symbol, call as i64->i64
                unsafe {
                    match libloading::Library::new(&lib_path) {
                        Ok(lib) => {
                            if let Ok(func) = lib.get::<extern "C" fn(i64) -> i64>(sym_name.as_bytes()) {
                                let input = if !args.is_empty() {
                                    match &args[0] {
                                        Value::Int(n) => *n,
                                        Value::Float(f) => *f as i64,
                                        _ => 0,
                                    }
                                } else { 0 };
                                let result = func(input);
                                self.data_stack.push(Value::Int(result));
                            } else {
                                self.data_stack.push(Value::Null);
                            }
                            // Note: Library handle is intentionally leaked here.
                            // A production implementation should store loaded libraries
                            // in a resource pool for proper cleanup.
                            // For now, we use mem::forget to avoid double-free since
                            // libloading::Library's Drop calls dlclose/FreeLibrary.
                            std::mem::forget(lib);
                        }
                        Err(_) => {
                            self.data_stack.push(Value::Null);
                        }
                    }
                }
            } else if instruction == OpCode::CallBuiltin as u8 {
                let name = self.read_string(bytecode, &mut pc);
                let nargs = bytecode[pc] as usize; pc += 1;
                let mut args = Vec::with_capacity(nargs);
                for _ in 0..nargs {
                    args.push(self.pop()?);
                }
                args.reverse();
                let func = *self.builtins.get(&name).ok_or(VmError(format!("unknown builtin '{}'", name)))?;
                let result = func(self, &args)?;
                self.data_stack.push(result);
            } else if instruction == OpCode::BindParams as u8 {
                let count = bytecode[pc] as usize; pc += 1;
                let mut names = Vec::with_capacity(count);
                for _ in 0..count {
                    names.push(self.read_string(bytecode, &mut pc));
                }
                let mut args = Vec::with_capacity(count);
                for _ in 0..count {
                    args.push(self.pop()?);
                }
                args.reverse();
                let frame = self.local_frames.last_mut()
                    .ok_or(VmError("parameter binding outside function frame".into()))?;
                for (name, value) in names.into_iter().zip(args.into_iter()) {
                    frame.insert(name, value);
                }
            }
        }
        Ok(())
    }

    pub fn print_top_stack_value(&self) {
        if let Some(val) = self.data_stack.last() {
            match val {
                Value::Int(n) => println!("=> [अङ्क]: {}", n),
                Value::Float(f) => println!("=> [दशमलव]: {}", f),
                Value::Bool(b) => println!("=> [सत्यता]: {}", if *b { "सत" } else { "असत" }),
                Value::Str(s) => println!("=> [शब्द]: {}", s),
                Value::Enum(name, _) => println!("=> [रूपभेदः::{}]", name),
                Value::Struct(_) => println!("=> [वस्तु_विन्यासः]"),
                _ => println!("=> [शून्य]"),
            }
        }
    }

    pub fn stack_top(&self) -> Option<&Value> {
        self.data_stack.last()
    }
}
