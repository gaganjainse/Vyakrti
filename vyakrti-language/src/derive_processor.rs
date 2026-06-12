use crate::ast::{ASTNode, Expression};

pub struct DeriveProcessor;

impl Default for DeriveProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl DeriveProcessor {
    pub fn new() -> Self { DeriveProcessor }

    pub fn expand_attributes(&mut self, nodes: Vec<ASTNode>) -> Vec<ASTNode> {
        let mut output_nodes = Vec::new();
        for node in nodes {
            match node.clone() {
                ASTNode::StructDecl { name, attributes, fields, .. } => {
                    output_nodes.push(node);
                    for attr in attributes {
                        if attr == "मुद्रणयोग्यता" {
                            output_nodes.push(self.generate_debug_impl(&name, &fields));
                        }
                    }
                }
                _ => output_nodes.push(node),
            }
        }
        output_nodes
    }

    fn generate_debug_impl(&self, struct_name: &str, fields: &[(String, String)]) -> ASTNode {
        let mut method_body = Vec::new();
        for (field_name, _) in fields {
            method_body.push(ASTNode::StatementExpr(Expression::Call {
                name: "मुद्रण".to_string(),
                args: vec![Expression::Variable(format!("स्वयं.{}", field_name))],
            }));
        }

        ASTNode::ImplBlock {
            trait_name: "मुद्रणयोग्यता".to_string(),
            type_name: struct_name.to_string(),
            methods: vec![ASTNode::FuncDecl {
                name: "विवरणम्".to_string(),
                parameters: vec![("स्वयं".to_string(), struct_name.to_string())],
                return_type: "शून्य".to_string(),
                body: method_body,
                access_level: crate::ast::AccessModifier::Public,
            }],
        }
    }
}
