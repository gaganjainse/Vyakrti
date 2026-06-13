use crate::ast::{ASTNode, Expression};
use crate::jit_memory::ExecutableMemory;

pub struct JitCompiler;

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl JitCompiler {
    pub fn new() -> Self { JitCompiler }

    pub fn compile_function(&mut self, body: &[ASTNode]) -> Result<ExecutableMemory, String> {
        let mut machine_code: Vec<u8> = Vec::new();

        machine_code.extend_from_slice(&[
            0x55,
            0x48, 0x89, 0xE5,
            0x48, 0x89, 0xF8,
        ]);

        for node in body {
            if let ASTNode::StatementExpr(Expression::Binary { op, right, .. }) = node {
                if let Expression::IntLiteral(val) = &**right {
                    match op.as_str() {
                        "+" => {
                            machine_code.push(0x48); machine_code.push(0x05);
                            machine_code.extend_from_slice(&(*val as i32).to_le_bytes());
                        }
                        "-" => {
                            machine_code.push(0x48); machine_code.push(0x2D);
                            machine_code.extend_from_slice(&(*val as i32).to_le_bytes());
                        }
                        _ => return Err(format!("JIT Error: Instruction operation '{}' not supported inside runtime bare-metal code generator compilation steps.", op)),
                    }
                }
            }
        }

        machine_code.extend_from_slice(&[
            0x5D,
            0xC3,
        ]);

        let mut jit_page = ExecutableMemory::allocate(machine_code.len())?;
        jit_page.write_bytes(&machine_code);
        jit_page.seal_and_protect_page()?;
        Ok(jit_page)
    }
}
