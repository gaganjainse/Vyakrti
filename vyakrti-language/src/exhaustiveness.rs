use std::collections::HashMap;
use crate::ast::{ASTNode, Expression};

pub struct ExhaustivenessAnalyzer {
    pub enum_registry: HashMap<String, Vec<String>>,
    pub variant_to_enum: HashMap<String, String>,
}

impl Default for ExhaustivenessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ExhaustivenessAnalyzer {
    pub fn new() -> Self {
        ExhaustivenessAnalyzer { enum_registry: HashMap::new(), variant_to_enum: HashMap::new() }
    }

    pub fn analyze_program(&mut self, nodes: &[ASTNode]) -> Result<(), String> {
        for node in nodes {
            if let ASTNode::EnumDecl { name, variants, .. } = node {
                let v_names: Vec<String> = variants.iter().map(|(vn, _)| vn.clone()).collect();
                for vn in &v_names { self.variant_to_enum.insert(vn.clone(), name.clone()); }
                self.enum_registry.insert(name.clone(), v_names);
            }
        }
        for node in nodes { self.verify_node(node)?; }
        Ok(())
    }

    fn verify_node(&self, node: &ASTNode) -> Result<(), String> {
        match node {
            ASTNode::VarDecl { value, .. } => self.verify_expr(value),
            ASTNode::StatementExpr(expr) => self.verify_expr(expr),
            _ => Ok(()),
        }
    }

    fn verify_expr(&self, expr: &Expression) -> Result<(), String> {
        if let Expression::MatchExpr { arms, .. } = expr {
            if arms.is_empty() { return Err("Compile-Time Error: Matching expression matrix must contain cases.".to_string()); }
            let first_pattern = &arms[0].0;
            if let Some(enum_parent) = self.variant_to_enum.get(first_pattern) {
                let mandatory_variants = &self.enum_registry[enum_parent];
                let matched_variants: Vec<String> = arms.iter().map(|(v, _, _)| v.clone()).collect();
                let mut unhandled = Vec::new();
                for mand in mandatory_variants {
                    if !matched_variants.contains(mand) { unhandled.push(mand.clone()); }
                }
                if !unhandled.is_empty() {
                    return Err(format!("Compile-Time Interface Failure: Match block for enum '{}' missing branch variants: {:?}", enum_parent, unhandled));
                }
            }
        }
        Ok(())
    }
}
