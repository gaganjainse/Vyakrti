use crate::vm::OpCode;

fn read_u16(bytecode: &[u8], pc: &mut usize) -> u16 {
    let hi = bytecode[*pc] as u16;
    let lo = bytecode[*pc + 1] as u16;
    *pc += 2;
    (hi << 8) | lo
}

fn read_string(bytecode: &[u8], pc: &mut usize) -> String {
    let len = read_u16(bytecode, pc) as usize;
    let s = String::from_utf8_lossy(&bytecode[*pc..*pc + len]).into_owned();
    *pc += len;
    s
}

pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(bytecode: &[u8]) {
        println!("=== VYĀKṚTI BYTECODE DISASSEMBLY ===");
        let mut pc = 0;
        while pc < bytecode.len() {
            print!("{:04X}: ", pc);
            let op = bytecode[pc];
            pc += 1;
            if op == OpCode::PushConst as u8 {
                let tag = bytecode[pc]; pc += 1;
                match tag {
                    0 => {
                        let mut bytes = [0; 8];
                        bytes.copy_from_slice(&bytecode[pc..pc + 8]); pc += 8;
                        println!("PUSH_CONST    Int({})", i64::from_be_bytes(bytes));
                    }
                    1 => {
                        let b = bytecode[pc] == 1; pc += 1;
                        println!("PUSH_CONST    Bool({})", b);
                    }
                    2 => {
                        let s = read_string(bytecode, &mut pc);
                        println!("PUSH_CONST    Str(\"{}\")", s);
                    }
                    3 => {
                        let mut bytes = [0; 8];
                        bytes.copy_from_slice(&bytecode[pc..pc + 8]); pc += 8;
                        println!("PUSH_CONST    Float({})", f64::from_be_bytes(bytes));
                    }
                    4 => {
                        let s = read_string(bytecode, &mut pc);
                        println!("PUSH_CONST    Enum({})", s);
                    }
                    5 => {
                        let n = bytecode[pc] as usize; pc += 1;
                        print!("PUSH_CONST    Struct(");
                        for i in 0..n {
                            let k = read_string(bytecode, &mut pc);
                            let _push = bytecode[pc]; pc += 1; // skip PushConst
                            let vt = bytecode[pc]; pc += 1;
                            match vt {
                                0 => { let mut b = [0; 8]; b.copy_from_slice(&bytecode[pc..pc+8]); pc += 8; print!("{}:Int({})", k, i64::from_be_bytes(b)); }
                                1 => { let v = bytecode[pc] == 1; pc += 1; print!("{}:Bool({})", k, v); }
                                2 => { let v = read_string(bytecode, &mut pc); print!("{}:Str(\"{}\")", k, v); }
                                3 => { let mut b = [0; 8]; b.copy_from_slice(&bytecode[pc..pc+8]); pc += 8; print!("{}:Float({})", k, f64::from_be_bytes(b)); }
                                _ => { print!("{}:?", k); }
                            }
                            if i + 1 < n { print!(", "); }
                        }
                        println!(")");
                    }
                    _ => println!("PUSH_CONST    unknown_tag({})", tag),
                }
            } else if op == OpCode::LoadVar as u8 {
                let s = read_string(bytecode, &mut pc);
                println!("LOAD_VAR      \"{}\"", s);
            } else if op == OpCode::StoreVar as u8 {
                let s = read_string(bytecode, &mut pc);
                println!("STORE_VAR     \"{}\"", s);
            } else if op == OpCode::Dup as u8 {
                println!("DUP");
            } else if op == OpCode::Drop as u8 {
                println!("DROP");
            } else if op == OpCode::GetVariant as u8 {
                println!("GET_VARIANT");
            } else if op == OpCode::Add as u8 { println!("ADD");
            } else if op == OpCode::Sub as u8 { println!("SUB");
            } else if op == OpCode::Mul as u8 { println!("MUL");
            } else if op == OpCode::Div as u8 { println!("DIV");
            } else if op == OpCode::Equal as u8 { println!("EQUAL");
            } else if op == OpCode::NotEqual as u8 { println!("NOT_EQUAL");
            } else if op == OpCode::LessThan as u8 { println!("LESS_THAN");
            } else if op == OpCode::GreaterThan as u8 { println!("GREATER_THAN");
            } else if op == OpCode::LessEqual as u8 { println!("LESS_EQUAL");
            } else if op == OpCode::GreaterEqual as u8 { println!("GREATER_EQUAL");
            } else if op == OpCode::FAdd as u8 { println!("FADD");
            } else if op == OpCode::FSub as u8 { println!("FSUB");
            } else if op == OpCode::FMul as u8 { println!("FMUL");
            } else if op == OpCode::FDiv as u8 { println!("FDIV");
            } else if op == OpCode::Jump as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]); pc += 4;
                println!("JUMP          {:04X}", u32::from_be_bytes(bytes));
            } else if op == OpCode::JumpIfFalse as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]); pc += 4;
                println!("JUMP_IF_FALSE {:04X}", u32::from_be_bytes(bytes));
            } else if op == OpCode::JumpIfTrue as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]); pc += 4;
                println!("JUMP_IF_TRUE  {:04X}", u32::from_be_bytes(bytes));
            } else if op == OpCode::Call as u8 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&bytecode[pc..pc + 4]); pc += 4;
                println!("CALL          {:04X}", u32::from_be_bytes(bytes));
            } else if op == OpCode::Return as u8 {
                println!("RETURN");
            } else if op == OpCode::CallBuiltin as u8 {
                let name = read_string(bytecode, &mut pc);
                let nargs = bytecode[pc]; pc += 1;
                println!("CALL_BUILTIN  \"{}\" ({} args)", name, nargs);
            } else if op == OpCode::CallForeign as u8 {
                let lib = read_string(bytecode, &mut pc);
                let sym = read_string(bytecode, &mut pc);
                let nargs = bytecode[pc]; pc += 1;
                println!("CALL_FOREIGN  \"{}\" from \"{}\" ({} args)", sym, lib, nargs);
            } else if op == OpCode::BindParams as u8 {
                let count = bytecode[pc] as usize; pc += 1;
                let mut names = Vec::new();
                for _ in 0..count {
                    names.push(read_string(bytecode, &mut pc));
                }
                println!("BIND_PARAMS   ({}) {}", count, names.join(", "));
            } else if op == OpCode::Extract as u8 {
                println!("EXTRACT");
            } else if op == OpCode::MakeEnum as u8 {
                println!("MAKE_ENUM");
            } else if op == OpCode::GetField as u8 {
                let name = read_string(bytecode, &mut pc);
                println!("GET_FIELD     \"{}\"", name);
            } else if op == OpCode::SetField as u8 {
                let name = read_string(bytecode, &mut pc);
                println!("SET_FIELD     \"{}\"", name);
            } else if op == OpCode::MakeStruct as u8 {
                let n = bytecode[pc]; pc += 1;
                println!("MAKE_STRUCT   ({} fields)", n);
            } else {
                println!("UNKNOWN_OPCODE 0x{:02X}", op);
            }
        }
        println!("====================================");
    }
}
