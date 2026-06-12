use crate::ast::{ASTNode, Expression};
use crate::vm::Value;

pub struct ASTOptimizer;

impl Default for ASTOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTOptimizer {
    pub fn new() -> Self { ASTOptimizer }

    pub fn optimize_program(&self, nodes: Vec<ASTNode>) -> Vec<ASTNode> {
        nodes.into_iter().map(|node| self.optimize_node(node)).collect()
    }

    fn optimize_node(&self, node: ASTNode) -> ASTNode {
        match node {
            ASTNode::VarDecl { name, data_type, value, .. } => ASTNode::VarDecl {
                name, data_type, value: self.optimize_expr(value), access_level: Default::default()
            },
            ASTNode::IfStmt { condition, then_branch, else_branch } => {
                match self.optimize_expr(condition) {
                    Expression::Literal(Value::Bool(true)) => ASTNode::Block(self.optimize_program(then_branch)),
                    Expression::Literal(Value::Bool(false)) => {
                        if let Some(else_nodes) = else_branch {
                            ASTNode::Block(self.optimize_program(else_nodes))
                        } else {
                            ASTNode::NoOp
                        }
                    }
                    cond => ASTNode::IfStmt {
                        condition: cond,
                        then_branch: self.optimize_program(then_branch),
                        else_branch: else_branch.map(|b| self.optimize_program(b)),
                    },
                }
            }
            ASTNode::ReturnStmt(expr) => ASTNode::ReturnStmt(self.optimize_expr(expr)),
            ASTNode::ModuleDecl { name, body } => ASTNode::ModuleDecl { name, body: self.optimize_program(body) },
            ASTNode::ModuleDef { name, body } => ASTNode::ModuleDef { name, body: self.optimize_program(body) },
            _ => node,
        }
    }

    fn normalize_int(&self, expr: Expression) -> Expression {
        match expr {
            Expression::IntLiteral(n) => Expression::Literal(Value::Int(n)),
            other => other,
        }
    }

    fn optimize_expr(&self, expr: Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let opt_left = self.normalize_int(self.optimize_expr(*left));
                let opt_right = self.normalize_int(self.optimize_expr(*right));

                if let (Expression::Literal(Value::Int(l)), Expression::Literal(Value::Int(r))) = (&opt_left, &opt_right) {
                    match op.as_str() {
                        "+" => Expression::Literal(Value::Int(l + r)),
                        "-" => Expression::Literal(Value::Int(l - r)),
                        "*" => Expression::Literal(Value::Int(l * r)),
                        "/" => {
                            if *r == 0 {
                                Expression::Binary { left: Box::new(opt_left), op, right: Box::new(opt_right) }
                            } else {
                                Expression::Literal(Value::Int(l / r))
                            }
                        }
                        "==" => Expression::Literal(Value::Bool(l == r)),
                        _ => Expression::Binary { left: Box::new(opt_left), op, right: Box::new(opt_right) },
                    }
                } else {
                    Expression::Binary { left: Box::new(opt_left), op, right: Box::new(opt_right) }
                }
            }
            Expression::IntLiteral(n) => Expression::Literal(Value::Int(n)),
            _ => expr,
        }
    }
}
