use std::collections::HashMap;
use crate::ast::{ASTNode, Expression};

pub struct BorrowChecker {
    pub ownership_registry: HashMap<String, bool>,
    pub borrow_registry: HashMap<String, (usize, bool)>,
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl BorrowChecker {
    pub fn new() -> Self {
        BorrowChecker { ownership_registry: HashMap::new(), borrow_registry: HashMap::new() }
    }

    pub fn check_program(&mut self, nodes: &[ASTNode]) -> Result<(), String> {
        for node in nodes { self.check_node(node)?; }
        Ok(())
    }

    fn check_node(&mut self, node: &ASTNode) -> Result<(), String> {
        match node {
            ASTNode::VarDecl { name, value, .. } => {
                self.check_expr(value)?;
                self.ownership_registry.insert(name.clone(), true);
                self.borrow_registry.insert(name.clone(), (0, false));
            }
            ASTNode::IfStmt { condition, then_branch, else_branch } => {
                self.check_expr(condition)?;
                for s in then_branch { self.check_node(s)?; }
                if let Some(eb) = else_branch { for s in eb { self.check_node(s)?; } }
            }
            ASTNode::WhileStmt { condition, body } => {
                self.check_expr(condition)?;
                for s in body { self.check_node(s)?; }
            }
            ASTNode::StatementExpr(expr) => self.check_expr(expr)?,
            ASTNode::ReturnStmt(expr) => self.check_expr(expr)?,
            ASTNode::Block(stmts) => { for s in stmts { self.check_node(s)?; } }
            _ => {}
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Variable(name) => {
                // Read-only access does NOT move ownership.
                // Only check that the variable exists and is not mutably borrowed.
                if let Some(&(_imm_c, mut_b)) = self.borrow_registry.get(name) {
                    if mut_b {
                        return Err(format!("cannot read '{}' because it is mutably borrowed", name));
                    }
                }
            }
            Expression::BorrowExpr { target, is_mutable } => {
                if let Some(false) = self.ownership_registry.get(target) {
                    return Err(format!("cannot reference moved value '{}'", target));
                }
                let (imm_c, mut_b) = self.borrow_registry.entry(target.clone()).or_insert((0, false));
                if *is_mutable {
                    if *imm_c > 0 || *mut_b {
                        return Err(format!("cannot mutably borrow '{}'", target));
                    }
                    *mut_b = true;
                } else {
                    if *mut_b {
                        return Err(format!("cannot immutably borrow '{}'", target));
                    }
                    *imm_c += 1;
                }
            }
            Expression::Binary { left, right, .. } => {
                self.check_expr(left)?;
                self.check_expr(right)?;
            }
            Expression::Call { args, .. } => {
                for arg in args { self.check_expr(arg)?; }
            }
            Expression::Literal(_) | Expression::IntLiteral(_) => {}
            _ => {}
        }
        Ok(())
    }
}
