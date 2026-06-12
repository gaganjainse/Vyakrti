use std::collections::HashMap;
use crate::ast::{ASTNode, Expression};

pub struct MacroExpander {
    pub macro_registry: HashMap<String, (Vec<String>, Vec<ASTNode>)>,
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroExpander {
    pub fn new() -> Self {
        MacroExpander { macro_registry: HashMap::new() }
    }

    pub fn expand_program(&mut self, nodes: Vec<ASTNode>) -> Vec<ASTNode> {
        let mut expanded_nodes = Vec::new();
        for node in nodes {
            match node {
                ASTNode::MacroDecl { name, params, body } => {
                    self.macro_registry.insert(name, (params, body));
                }
                ASTNode::MacroCall { name, args } => {
                    let mut expanded_exprs = self.expand_macro_call(&name, &args);
                    expanded_nodes.append(&mut expanded_exprs);
                }
                _ => expanded_nodes.push(self.expand_node(node)),
            }
        }
        expanded_nodes
    }

    fn expand_node(&mut self, node: ASTNode) -> ASTNode {
        match node {
            ASTNode::IfStmt { condition, then_branch, else_branch } => ASTNode::IfStmt {
                condition,
                then_branch: self.expand_program(then_branch),
                else_branch: else_branch.map(|b| self.expand_program(b)),
            },
            ASTNode::WhileStmt { condition, body } => ASTNode::WhileStmt {
                condition,
                body: self.expand_program(body),
            },
            ASTNode::ReturnStmt(expr) => ASTNode::ReturnStmt(expr),
            ASTNode::ModuleDecl { name, body } => ASTNode::ModuleDecl { name, body: self.expand_program(body) },
            ASTNode::ModuleDef { name, body } => ASTNode::ModuleDef { name, body: self.expand_program(body) },
            _ => node,
        }
    }

    fn expand_macro_call(&self, name: &str, args: &[Expression]) -> Vec<ASTNode> {
        if let Some((params, template_body)) = self.macro_registry.get(name) {
            let mut substitution_map = HashMap::new();
            for (param, arg) in params.iter().zip(args.iter()) {
                substitution_map.insert(param.clone(), arg.clone());
            }
            template_body.iter().map(|node| self.substitute_node(node.clone(), &substitution_map)).collect()
        } else {
            panic!("Compile-Time Macro Error: Attempted to invoke unregistered macro variant '{}!'.", name);
        }
    }

    fn substitute_node(&self, node: ASTNode, map: &HashMap<String, Expression>) -> ASTNode {
        match node {
            ASTNode::VarDecl { name, data_type, value, .. } => ASTNode::VarDecl {
                name, data_type, value: self.substitute_expr(value, map), access_level: Default::default()
            },
            ASTNode::StatementExpr(expr) => ASTNode::StatementExpr(self.substitute_expr(expr, map)),
            ASTNode::ReturnStmt(expr) => ASTNode::ReturnStmt(self.substitute_expr(expr, map)),
            _ => node,
        }
    }

    fn substitute_expr(&self, expr: Expression, map: &HashMap<String, Expression>) -> Expression {
        match expr {
            Expression::Variable(name) => map.get(&name).cloned().unwrap_or(Expression::Variable(name)),
            Expression::Binary { left, op, right } => Expression::Binary {
                left: Box::new(self.substitute_expr(*left, map)),
                op,
                right: Box::new(self.substitute_expr(*right, map)),
            },
            _ => expr,
        }
    }
}
