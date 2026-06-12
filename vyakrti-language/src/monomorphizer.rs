use std::collections::HashMap;
use crate::ast::{ASTNode, Expression};

pub struct ASTMonomorphizer {
    pub templates: HashMap<String, (Vec<String>, Vec<(String, String)>, String, Vec<ASTNode>)>,
    pub generated_functions: Vec<ASTNode>,
}

impl Default for ASTMonomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTMonomorphizer {
    pub fn new() -> Self {
        ASTMonomorphizer { templates: HashMap::new(), generated_functions: Vec::new() }
    }

    pub fn process_program(&mut self, nodes: Vec<ASTNode>) -> Vec<ASTNode> {
        let mut cleaned_nodes = Vec::new();
        for node in nodes {
            match node {
                ASTNode::GenericFuncDecl { name, type_params, parameters, return_type, body, .. } => {
                    self.templates.insert(name, (type_params, parameters, return_type, body));
                }
                _ => cleaned_nodes.push(node),
            }
        }

        let mut finalized_nodes = Vec::new();
        for node in cleaned_nodes {
            finalized_nodes.push(self.specialize_node(node));
        }

        let mut output = self.generated_functions.clone();
        output.append(&mut finalized_nodes);
        output
    }

    fn specialize_node(&mut self, node: ASTNode) -> ASTNode {
        match node {
            ASTNode::VarDecl { name, data_type, value, .. } => ASTNode::VarDecl {
                name, data_type, value: self.specialize_expr(value), access_level: Default::default()
            },
            ASTNode::StatementExpr(expr) => ASTNode::StatementExpr(self.specialize_expr(expr)),
            ASTNode::ReturnStmt(expr) => ASTNode::ReturnStmt(self.specialize_expr(expr)),
            _ => node,
        }
    }

    fn specialize_expr(&mut self, expr: Expression) -> Expression {
        match expr {
            Expression::GenericCall { name, concrete_types, args } => {
                if let Some((type_params, parameters, return_type, body)) = self.templates.get(&name).cloned() {
                    let mangled_name = format!("{}_{}", name, concrete_types.join("_"));
                    let mut type_map = HashMap::new();
                    for (placeholder, concrete) in type_params.iter().zip(concrete_types.iter()) {
                        type_map.insert(placeholder.clone(), concrete.clone());
                    }

                    let specialized_params: Vec<(String, String)> = parameters.iter().map(|(p_name, p_type)| {
                        (p_name.clone(), type_map.get(p_type).unwrap_or(p_type).clone())
                    }).collect();

                    let concrete_func = ASTNode::FuncDecl {
                        name: mangled_name.clone(),
                        parameters: specialized_params,
                        return_type: type_map.get(&return_type).unwrap_or(&return_type).clone(),
                        body,
                        access_level: crate::ast::AccessModifier::Private,
                    };

                    if !self.generated_functions.contains(&concrete_func) {
                        self.generated_functions.push(concrete_func);
                    }

                    Expression::Call {
                        name: mangled_name,
                        args: args.into_iter().map(|e| self.specialize_expr(e)).collect(),
                    }
                } else {
                    panic!("Compile-Time Generic Resolution Fault: Template configuration blueprint '{}' vanished.", name);
                }
            }
            _ => expr,
        }
    }
}
