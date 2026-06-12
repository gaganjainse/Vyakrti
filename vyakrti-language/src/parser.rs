use crate::lexer::{Token, SpannedToken};
use crate::ast::{ASTNode, AccessModifier, Expression};
use crate::vm::Value;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn peek(&self) -> Option<&SpannedToken> {
        if self.position >= self.tokens.len() { None } else { Some(&self.tokens[self.position]) }
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        if self.position >= self.tokens.len() {
            None
        } else {
            let tok = self.tokens[self.position].clone();
            self.position += 1;
            Some(tok)
        }
    }

    fn expect_advance(&mut self, expected: &Token, msg: &str) -> Result<SpannedToken, String> {
        match self.advance() {
            Some(t) if t.token == *expected => Ok(t),
            Some(t) => Err(format!("error at {}:{}: expected '{}', got '{:?}'", t.line, t.col, msg, t.token)),
            None => Err(format!("error: expected '{}', got end of input", msg)),
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut program = Vec::new();
        while self.peek().is_some() {
            // Skip DoubleDanda markers (block/program end)
            if matches!(self.peek(), Some(SpannedToken { token: Token::DoubleDanda, .. })) {
                self.advance();
                continue;
            }
            // Skip single Danda at top level (statement separator)
            if matches!(self.peek(), Some(SpannedToken { token: Token::Danda, .. })) {
                self.advance();
                continue;
            }
            program.push(self.parse_statement()?);
        }
        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<ASTNode, String> {
        // Check for access modifier prefix
        let access = self.parse_access_modifier();

        match self.peek().map(|t| &t.token) {
            Some(Token::VarDeclKW) => self.parse_var_declaration(access),
            Some(Token::FuncDeclKW) | Some(Token::AsyncKW) => self.parse_func_or_async_declaration(access),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::Yavat) => self.parse_while_statement(),
            Some(Token::Import) => self.parse_import_statement(),
            Some(Token::Spawn) => self.parse_spawn_statement(),
            Some(Token::Delay) => self.parse_delay_statement(),
            Some(Token::ForeignKW) => self.parse_ffi_declaration(access),
            Some(Token::EnumKW) => self.parse_enum_declaration(access),
            Some(Token::TraitKW) => self.parse_trait_declaration(access),
            Some(Token::ImplKW) => self.parse_impl_block(),
            Some(Token::AtSign) => self.parse_attributed_struct(access),
            Some(Token::StructKW) => self.parse_standard_struct(access),
            Some(Token::Pratiphala) => self.parse_return_statement(),
            Some(Token::Danda) => {
                self.advance();
                Ok(ASTNode::NoOp)
            }
            Some(Token::DoubleDanda) => {
                self.advance();
                Ok(ASTNode::NoOp)
            }
            _ => {
                let expr = self.parse_expression(0)?;
                self.expect_advance(&Token::Danda, "।")?;
                Ok(ASTNode::StatementExpr(expr))
            }
        }
    }

    fn parse_access_modifier(&mut self) -> AccessModifier {
        match self.peek().map(|t| &t.token) {
            Some(Token::UdAtta) => { self.advance(); AccessModifier::Public }
            Some(Token::AnudAtta) => { self.advance(); AccessModifier::Private }
            Some(Token::Svarita) => { self.advance(); AccessModifier::Protected }
            _ => AccessModifier::Private,
        }
    }

    fn parse_var_declaration(&mut self, access: AccessModifier) -> Result<ASTNode, String> {
        self.advance();
        let name = match self.advance() {
            Some(SpannedToken { token: Token::Identifier(id), .. }) => id,
            Some(t) => return Err(format!("error at {}:{}: expected identifier for variable name", t.line, t.col)),
            None => return Err("expected identifier for variable name".into()),
        };

        let mut data_type = None;
        if matches!(self.peek(), Some(SpannedToken { token: Token::Colon, .. })) {
            self.advance();
            if let Some(SpannedToken { token: Token::Identifier(t_name), .. }) = self.advance() {
                data_type = Some(t_name);
            }
        }

        self.expect_advance(&Token::Assign, "=")?;
        let value = self.parse_expression(0)?;
        self.expect_advance(&Token::Danda, "।")?;

        Ok(ASTNode::VarDecl { name, data_type, value, access_level: access })
    }

    fn parse_func_or_async_declaration(&mut self, access: AccessModifier) -> Result<ASTNode, String> {
        let is_async = if matches!(self.peek(), Some(SpannedToken { token: Token::AsyncKW, .. })) {
            self.advance();
            true
        } else {
            false
        };

        self.expect_advance(&Token::FuncDeclKW, "कार्य")?;

        let func_name = match self.advance() {
            Some(SpannedToken { token: Token::Identifier(id), .. }) => id,
            Some(t) => return Err(format!("error at {}:{}: expected function name", t.line, t.col)),
            None => return Err("expected function name".into()),
        };

        let mut type_params = Vec::new();
        if matches!(self.peek(), Some(SpannedToken { token: Token::LessThan, .. })) {
            self.advance();
            while !matches!(self.peek(), Some(SpannedToken { token: Token::GreaterThan, .. })) {
                if let Some(SpannedToken { token: Token::Identifier(tp), .. }) = self.advance() {
                    type_params.push(tp);
                }
                if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
            }
            self.advance();
        }

        self.expect_advance(&Token::LeftParen, "(")?;

        let mut parameters = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
            if let Some(SpannedToken { token: Token::Identifier(p_name), .. }) = self.advance() {
                let mut p_type = "विविध".to_string();
                if matches!(self.peek(), Some(SpannedToken { token: Token::Colon, .. })) {
                    self.advance();
                    if let Some(SpannedToken { token: Token::Identifier(t), .. }) = self.advance() { p_type = t; }
                }
                parameters.push((p_name, p_type));
            }
            if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
        }
        self.advance();

        let mut return_type = "शून्य".to_string();
        if matches!(self.peek(), Some(SpannedToken { token: Token::Arrow, .. })) {
            self.advance();
            match self.advance() {
                Some(SpannedToken { token: Token::Identifier(rt), .. }) => return_type = rt,
                Some(t) => return Err(format!("error at {}:{}: expected return type after ->", t.line, t.col)),
                None => return Err("expected return type after ->".into()),
            }
        }

        self.expect_advance(&Token::LeftBrace, "{")?;

        let mut body = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightBrace, .. })) {
            body.push(self.parse_statement()?);
        }
        self.advance();

        if is_async {
            Ok(ASTNode::AsyncFuncDecl { name: func_name, parameters, return_type, body, access_level: access })
        } else if !type_params.is_empty() {
            Ok(ASTNode::GenericFuncDecl { name: func_name, type_params, parameters, return_type, body, access_level: access })
        } else {
            Ok(ASTNode::FuncDecl { name: func_name, parameters, return_type, body, access_level: access })
        }
    }

    fn parse_if_statement(&mut self) -> Result<ASTNode, String> {
        self.advance();
        let condition = self.parse_expression(0)?;
        self.expect_advance(&Token::Tarhi, "तर्हि")?;

        let then_branch = self.parse_block_body()?;
        let mut else_branch = None;

        if matches!(self.peek(), Some(SpannedToken { token: Token::Anyatha, .. })) {
            self.advance();
            else_branch = Some(self.parse_block_body()?);
        }

        Ok(ASTNode::IfStmt { condition, then_branch, else_branch })
    }

    fn parse_while_statement(&mut self) -> Result<ASTNode, String> {
        self.advance();
        let condition = self.parse_expression(0)?;
        self.expect_advance(&Token::Tavat, "तावत्")?;
        let body = self.parse_block_body()?;
        Ok(ASTNode::WhileStmt { condition, body })
    }

    fn parse_block_body(&mut self) -> Result<Vec<ASTNode>, String> {
        self.expect_advance(&Token::LeftBrace, "{")?;
        let mut body = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightBrace, .. })) {
            body.push(self.parse_statement()?);
        }
        self.advance();
        Ok(body)
    }

    fn parse_import_statement(&mut self) -> Result<ASTNode, String> {
        self.advance();
        let file_path = match self.advance() {
            Some(SpannedToken { token: Token::Literal(Value::Str(path)), .. }) => path,
            Some(t) => return Err(format!("error at {}:{}: expected string path for import", t.line, t.col)),
            None => return Err("expected string path for import".into()),
        };
        self.expect_advance(&Token::Danda, "।")?;
        let namespace_alias = file_path.replace(".vya", "");
        Ok(ASTNode::ImportDecl { file_path, namespace_alias })
    }

    fn parse_spawn_statement(&mut self) -> Result<ASTNode, String> {
        self.advance();
        let expr = self.parse_expression(0)?;
        self.expect_advance(&Token::Danda, "।")?;
        Ok(ASTNode::SpawnStmt(expr))
    }

    fn parse_delay_statement(&mut self) -> Result<ASTNode, String> {
        self.advance();
        let expr = self.parse_expression(0)?;
        self.expect_advance(&Token::Danda, "।")?;
        Ok(ASTNode::DelayStmt(expr))
    }

    fn parse_return_statement(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume प्रतिफल
        let expr = self.parse_expression(0)?;
        self.expect_advance(&Token::Danda, "।")?;
        Ok(ASTNode::ReturnStmt(expr))
    }

    fn parse_ffi_declaration(&mut self, access: AccessModifier) -> Result<ASTNode, String> {
        self.advance();
        let library_path = match self.advance() {
            Some(SpannedToken { token: Token::Literal(Value::Str(lib)), .. }) => lib,
            Some(t) => return Err(format!("error at {}:{}: expected string for library path", t.line, t.col)),
            None => return Err("expected string for library path".into()),
        };
        let symbol_name = match self.advance() {
            Some(SpannedToken { token: Token::Identifier(id), .. }) => id,
            Some(t) => return Err(format!("error at {}:{}: expected function symbol name", t.line, t.col)),
            None => return Err("expected function symbol name".into()),
        };
        self.expect_advance(&Token::LeftParen, "(")?;
        let mut parameters = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
            if let Some(SpannedToken { token: Token::Identifier(p_name), .. }) = self.advance() {
                let mut p_type = "अङ्क".to_string();
                if matches!(self.peek(), Some(SpannedToken { token: Token::Colon, .. })) {
                    self.advance();
                    if let Some(SpannedToken { token: Token::Identifier(t), .. }) = self.advance() { p_type = t; }
                }
                parameters.push((p_name, p_type));
            }
            if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
        }
        self.advance();
        let mut return_type = "शून्य".to_string();
        if matches!(self.peek(), Some(SpannedToken { token: Token::Arrow, .. })) {
            self.advance();
            if let Some(SpannedToken { token: Token::Identifier(rt), .. }) = self.advance() { return_type = rt; }
        }
        self.expect_advance(&Token::Danda, "।")?;
        Ok(ASTNode::FFIDecl { library_path, symbol_name, parameters, return_type, access_level: access })
    }

    fn parse_enum_declaration(&mut self, access: AccessModifier) -> Result<ASTNode, String> {
        self.advance();
        let name = match self.advance() {
            Some(SpannedToken { token: Token::Identifier(id), .. }) => id,
            Some(t) => return Err(format!("error at {}:{}: expected enum name", t.line, t.col)),
            None => return Err("expected enum name".into()),
        };
        self.expect_advance(&Token::LeftBrace, "{")?;
        let mut variants = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightBrace, .. })) {
            if let Some(SpannedToken { token: Token::Identifier(v_name), .. }) = self.advance() {
                let mut payload = None;
                if matches!(self.peek(), Some(SpannedToken { token: Token::LeftParen, .. })) {
                    self.advance();
                    let mut types = Vec::new();
                    while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
                        if let Some(SpannedToken { token: Token::Identifier(t), .. }) = self.advance() { types.push(t); }
                        if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                    }
                    self.advance();
                    payload = Some(types);
                }
                variants.push((v_name, payload));
            }
            if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
        }
        self.advance();
        Ok(ASTNode::EnumDecl { name, variants, access_level: access })
    }

    fn parse_trait_declaration(&mut self, access: AccessModifier) -> Result<ASTNode, String> {
        self.advance();
        let name = match self.advance() {
            Some(SpannedToken { token: Token::Identifier(id), .. }) => id,
            Some(t) => return Err(format!("error at {}:{}: expected trait name", t.line, t.col)),
            None => return Err("expected trait name".into()),
        };
        self.expect_advance(&Token::LeftBrace, "{")?;
        let mut method_signatures = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightBrace, .. })) {
            if matches!(self.peek(), Some(SpannedToken { token: Token::FuncDeclKW, .. })) {
                self.advance();
                if let Some(SpannedToken { token: Token::Identifier(m_name), .. }) = self.advance() {
                    self.expect_advance(&Token::LeftParen, "(")?;
                    let mut params = Vec::new();
                    while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
                        if let Some(SpannedToken { token: Token::Identifier(p), .. }) = self.advance() {
                            let mut t = "विविध".to_string();
                            if matches!(self.peek(), Some(SpannedToken { token: Token::Colon, .. })) { self.advance(); if let Some(SpannedToken { token: Token::Identifier(t_id), .. }) = self.advance() { t = t_id; } }
                            params.push((p, t));
                        }
                        if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                    }
                    self.advance();
                    let mut r_type = "शून्य".to_string();
                    if matches!(self.peek(), Some(SpannedToken { token: Token::Arrow, .. })) { self.advance(); if let Some(SpannedToken { token: Token::Identifier(rt), .. }) = self.advance() { r_type = rt; } }
                    self.expect_advance(&Token::Danda, "।")?;
                    method_signatures.push((m_name, params, r_type));
                }
            } else {
                self.advance();
            }
        }
        self.advance();
        Ok(ASTNode::TraitDecl { name, method_signatures, access_level: access })
    }

    fn parse_impl_block(&mut self) -> Result<ASTNode, String> {
        self.advance();
        let trait_name = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected trait name", t.line, t.col)), None => return Err("expected trait name".into()) };
        self.advance();
        let type_name = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected type name", t.line, t.col)), None => return Err("expected type name".into()) };
        self.expect_advance(&Token::LeftBrace, "{")?;
        let mut methods = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightBrace, .. })) {
            methods.push(self.parse_statement()?);
        }
        self.advance();
        Ok(ASTNode::ImplBlock { trait_name, type_name, methods })
    }

    fn parse_attributed_struct(&mut self, access: AccessModifier) -> Result<ASTNode, String> {
        self.advance();
        let attr = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected attribute name", t.line, t.col)), None => return Err("expected attribute name".into()) };
        self.expect_advance(&Token::StructKW, "वस्तु_विन्यासः")?;
        let name = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected struct name", t.line, t.col)), None => return Err("expected struct name".into()) };
        let fields = self.parse_struct_fields()?;
        Ok(ASTNode::StructDecl { name, attributes: vec![attr], fields, access_level: access })
    }

    fn parse_standard_struct(&mut self, access: AccessModifier) -> Result<ASTNode, String> {
        self.advance();
        let name = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected struct name", t.line, t.col)), None => return Err("expected struct name".into()) };
        let fields = self.parse_struct_fields()?;
        Ok(ASTNode::StructDecl { name, attributes: Vec::new(), fields, access_level: access })
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<(String, String)>, String> {
        self.expect_advance(&Token::LeftBrace, "{")?;
        let mut fields = Vec::new();
        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightBrace, .. })) {
            if let Some(SpannedToken { token: Token::Identifier(f_name), .. }) = self.advance() {
                self.expect_advance(&Token::Colon, ":")?;
                if let Some(SpannedToken { token: Token::Identifier(f_type), .. }) = self.advance() {
                    fields.push((f_name, f_type));
                }
            }
            if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
        }
        self.advance();
        Ok(fields)
    }

    fn parse_expression(&mut self, precedence: u8) -> Result<Expression, String> {
        let mut left = match self.advance() {
            Some(SpannedToken { token: Token::Literal(val), .. }) => Expression::Literal(val),
            Some(SpannedToken { token: Token::IntLiteral(val), .. }) => Expression::IntLiteral(val),
            Some(SpannedToken { token: Token::LeftParen, .. }) => {
                let expr = self.parse_expression(0)?;
                self.expect_advance(&Token::RightParen, ")")?;
                expr
            }
            Some(SpannedToken { token: Token::Identifier(id), .. }) => {
                if matches!(self.peek(), Some(SpannedToken { token: Token::LessThan, .. })) {
                    self.advance();
                    let mut concrete_types = Vec::new();
                    while !matches!(self.peek(), Some(SpannedToken { token: Token::GreaterThan, .. })) {
                        if let Some(SpannedToken { token: Token::Identifier(t), .. }) = self.advance() { concrete_types.push(t); }
                        if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                    }
                    self.advance();
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
                        args.push(self.parse_expression(0)?);
                        if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                    }
                    self.advance();
                    Expression::GenericCall { name: id, concrete_types, args }
                } else if matches!(self.peek(), Some(SpannedToken { token: Token::LeftParen, .. })) {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
                        args.push(self.parse_expression(0)?);
                        if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                    }
                    self.advance();
                    Expression::Call { name: id, args }
                } else {
                    Expression::Variable(id)
                }
            }
            Some(SpannedToken { token: Token::MatchKW, .. }) => {
                let target = self.parse_expression(0)?;
                self.expect_advance(&Token::LeftBrace, "{")?;
                let mut arms = Vec::new();
                while !matches!(self.peek(), Some(SpannedToken { token: Token::RightBrace, .. })) {
                    let v_pattern = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected variant pattern", t.line, t.col)), None => return Err("expected variant pattern".into()) };
                    let mut binds = Vec::new();
                    if matches!(self.peek(), Some(SpannedToken { token: Token::LeftParen, .. })) {
                        self.advance();
                        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
                            if let Some(SpannedToken { token: Token::Identifier(b), .. }) = self.advance() { binds.push(b); }
                            if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                        }
                        self.advance();
                    }
                    self.expect_advance(&Token::FatArrow, "=>")?;
                    let expr = self.parse_expression(0)?;
                    arms.push((v_pattern, binds, expr));
                    if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                }
                self.advance();
                Expression::MatchExpr { eval_target: Box::new(target), arms }
            }
            Some(SpannedToken { token: Token::AwaitKW, .. }) => Expression::AwaitExpr { target_future: Box::new(self.parse_expression(7)?) },
            Some(SpannedToken { token: Token::Reference, .. }) => {
                let is_mut = if matches!(self.peek(), Some(SpannedToken { token: Token::Mutable, .. })) { self.advance(); true } else { false };
                let target = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected variable name after ref", t.line, t.col)), None => return Err("expected variable name after ref".into()) };
                Expression::BorrowExpr { target, is_mutable: is_mut }
            }
            Some(t) => return Err(format!("error at {}:{}: unexpected token {:?}", t.line, t.col, t.token)),
            None => return Err("unexpected end of input in expression".into()),
        };

        while let Some(tok) = self.peek() {
            let next_prec = self.get_precedence(&tok.token);
            if precedence >= next_prec { break; }
            let spanned = self.advance().ok_or("expected operator token".to_string())?;

            match spanned.token {
                Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent | Token::EqualEqual | Token::NotEqual | Token::LessThan | Token::GreaterThan | Token::LessEqual | Token::GreaterEqual | Token::Samana | Token::Una | Token::Agra | Token::UnaSamana | Token::AgraSamana | Token::Asamana | Token::Cha | Token::Va => {
                    let op_str = match spanned.token {
                        Token::Plus => "+", Token::Minus => "-", Token::Star => "*", Token::Slash => "/", Token::Percent => "%",
                        Token::EqualEqual | Token::Samana => "==",
                        Token::NotEqual | Token::Asamana => "!=",
                        Token::LessThan | Token::Una => "<",
                        Token::GreaterThan | Token::Agra => ">",
                        Token::LessEqual | Token::UnaSamana => "<=",
                        Token::GreaterEqual | Token::AgraSamana => ">=",
                        Token::Cha => "च",
                        Token::Va => "वा",
                        _ => "",
                    }.to_string();
                    let right = self.parse_expression(next_prec)?;
                    left = Expression::Binary { left: Box::new(left), op: op_str, right: Box::new(right) };
                }
                Token::Dot => {
                    let field_name = match self.advance() { Some(SpannedToken { token: Token::Identifier(id), .. }) => id, Some(t) => return Err(format!("error at {}:{}: expected field name after .", t.line, t.col)), None => return Err("expected field name after .".into()) };
                    if matches!(self.peek(), Some(SpannedToken { token: Token::LeftParen, .. })) {
                        self.advance();
                        let mut args = Vec::new();
                        while !matches!(self.peek(), Some(SpannedToken { token: Token::RightParen, .. })) {
                            args.push(self.parse_expression(0)?);
                            if matches!(self.peek(), Some(SpannedToken { token: Token::Comma, .. })) { self.advance(); }
                        }
                        self.advance();
                        left = Expression::MethodCall { instance: Box::new(left), method_name: field_name, args };
                    } else {
                        left = Expression::FieldAccess { object: Box::new(left), field: field_name };
                    }
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn get_precedence(&self, token: &Token) -> u8 {
        match token {
            Token::Dot | Token::LeftParen => 8,
            Token::Star | Token::Slash | Token::Percent => 6,
            Token::Plus | Token::Minus => 5,
            Token::LessThan | Token::GreaterThan | Token::LessEqual | Token::GreaterEqual | Token::EqualEqual | Token::NotEqual
                | Token::Samana | Token::Una | Token::Agra | Token::UnaSamana | Token::AgraSamana | Token::Asamana => 4,
            Token::Cha => 3, // च binds tighter than वा
            Token::Va => 2,
            Token::Assign => 1,
            _ => 0,
        }
    }
}
