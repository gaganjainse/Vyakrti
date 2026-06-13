#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Terminators
    Danda,        // ।
    DoubleDanda,  // ॥

    // Declarations
    FuncDeclKW,   // कार्य
    VarDeclKW,    // मान
    StructKW,     // वस्तु_विन्यासः
    TraitKW,      // गुणधर्म
    ImplKW,       // अनुष्ठान
    EnumKW,       // रूपभेदः
    MatchKW,      // समीक्षा
    Import,       // आयात

    // Control flow
    If,           // यदि
    Tarhi,        // तर्हि
    Anyatha,      // अन्यथा
    Yavat,        // यावत्
    Tavat,        // तावत्
    Pratiphala,   // प्रतिफल (return)

    // Concurrency / FFI
    AsyncKW,      // असमकाल
    AwaitKW,      // प्रतीक्षस्व
    Spawn,        // सहप्रक्रिया
    Delay,        // विलम्बः
    Foreign,      // वैदेशिक
    ForeignKW,    // विदेशीय

    // Borrow semantics
    Reference,    // सन्दर्भः
    Mutable,      // परिवर्त्य

    // Access modifiers (Swara)
    UdAtta,       // उदात्त (public)
    AnudAtta,     // अनुदात्त (private)
    Svarita,      // स्वरित (protected)

    // Boolean operators
    Cha,          // च (and)
    Va,           // वा (or)

    // ASCII comparison operators (kept for generics < > and compatibility)
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    NotEqual,

    // Devanagari comparison operators
    Samana,       // समान (==)
    Una,          // ऊन (<)
    Agra,         // अग्र (>)
    UnaSamana,    // ऊनसमान (<=)
    AgraSamana,   // अग्रसमान (>=)
    Asamana,      // असमान (!=)

    // Structural
    Colon,
    Comma,
    Dot,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    AtSign,
    Arrow,
    FatArrow,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Identifiers and literals
    Identifier(String),
    Literal(crate::vm::Value),
    IntLiteral(i64),
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            input: source.chars().collect(),
            position: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position >= self.input.len() { None } else { Some(self.input[self.position]) }
    }

    fn advance(&mut self) -> Option<char> {
        if self.position >= self.input.len() {
            None
        } else {
            let ch = self.input[self.position];
            self.position += 1;
            if ch == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
            Some(ch)
        }
    }

    fn pos(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // Danda (। U+0964) and Double Danda (॥ U+0965) — handle BEFORE generic Devanagari range check
            if ch == '\u{0964}' {
                self.advance();
                let (line, col) = self.pos();
                tokens.push(SpannedToken { token: Token::Danda, line, col });
                continue;
            }
            if ch == '\u{0965}' {
                self.advance();
                let (line, col) = self.pos();
                tokens.push(SpannedToken { token: Token::DoubleDanda, line, col });
                continue;
            }

            if ch.is_ascii_digit() || ('०'..='९').contains(&ch) {
                tokens.push(self.read_numeric_literal());
                continue;
            }

            if ch.is_alphabetic() || ch == '_' || ('\u{0900}'..='\u{097F}').contains(&ch) {
                tokens.push(self.read_identifier_or_keyword());
                continue;
            }

            self.advance();
            let (line, col) = self.pos();
            let tok = match ch {
                ':' => Token::Colon,
                ',' => Token::Comma,
                '.' => Token::Dot,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '+' => Token::Plus,
                '-' => {
                    if self.peek() == Some('>') {
                        self.advance();
                        Token::Arrow
                    } else {
                        Token::Minus
                    }
                }
                '*' => Token::Star,
                '/' => {
                    if self.peek() == Some('/') {
                        while let Some(c) = self.peek() {
                            if c == '\n' { break; }
                            self.advance();
                        }
                        continue;
                    }
                    if self.peek() == Some('*') {
                        self.advance();
                        loop {
                            match self.advance() {
                                Some('*') if self.peek() == Some('/') => { self.advance(); break; }
                                Some(_) => continue,
                                None => break,
                            }
                        }
                        continue;
                    }
                    Token::Slash
                }
                '%' => Token::Percent,
                '@' => Token::AtSign,
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::LessEqual
                    } else {
                        Token::LessThan
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::GreaterEqual
                    } else {
                        Token::GreaterThan
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::NotEqual
                    } else {
                        continue;
                    }
                }
                '=' => {
                    if self.peek() == Some('>') {
                        self.advance();
                        Token::FatArrow
                    } else if self.peek() == Some('=') {
                        self.advance();
                        Token::EqualEqual
                    } else {
                        Token::Assign
                    }
                }
                '"' => Token::Literal(crate::vm::Value::Str(self.read_string_literal())),
                _ => continue,
            };
            tokens.push(SpannedToken { token: tok, line, col });
        }
        tokens
    }

    fn read_numeric_literal(&mut self) -> SpannedToken {
        let (line, col) = self.pos();
        let mut text = String::new();
        let mut is_float = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.advance();
            } else if ('०'..='९').contains(&ch) {
                let normalized = ((ch as u32) - ('०' as u32) + ('0' as u32)) as u8 as char;
                text.push(normalized);
                self.advance();
            } else if ch == '.' {
                is_float = true;
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        let token = if is_float {
            Token::Literal(crate::vm::Value::Float(
                text.parse::<f64>().unwrap_or_else(|_| {
                    eprintln!("Warning: Invalid float literal at {}:{}: '{}'", line, col, text);
                    0.0
                })
            ))
        } else {
            Token::IntLiteral(
                text.parse::<i64>().unwrap_or_else(|_| {
                    eprintln!("Warning: Invalid integer literal at {}:{}: '{}'", line, col, text);
                    0
                })
            )
        };
        SpannedToken { token, line, col }
    }

    fn read_string_literal(&mut self) -> String {
        let mut text = String::new();
        while let Some(ch) = self.advance() {
            if ch == '"' { break; }
            if ch == '\\' {
                match self.advance() {
                    Some('n') => text.push('\n'),
                    Some('t') => text.push('\t'),
                    Some('r') => text.push('\r'),
                    Some('\\') => text.push('\\'),
                    Some('"') => text.push('"'),
                    Some(c) => { text.push('\\'); text.push(c); }
                    None => text.push('\\'),
                }
            } else {
                text.push(ch);
            }
        }
        text
    }

    fn read_identifier_or_keyword(&mut self) -> SpannedToken {
        let (line, col) = self.pos();
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ('\u{0900}'..='\u{097F}').contains(&ch) {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token = match text.as_str() {
            // Declarations
            "कार्य" => Token::FuncDeclKW,
            "मान" => Token::VarDeclKW,
            "वस्तु_विन्यासः" => Token::StructKW,
            "गुणधर्म" => Token::TraitKW,
            "अनुष्ठान" => Token::ImplKW,
            "रूपभेदः" => Token::EnumKW,
            "समीक्षा" => Token::MatchKW,
            "आयात" => Token::Import,
            // Access modifiers (Swara)
            "उदात्त" => Token::UdAtta,
            "अनुदात्त" => Token::AnudAtta,
            "स्वरित" => Token::Svarita,
            // Control flow
            "यदि" => Token::If,
            "तर्हि" => Token::Tarhi,
            "अन्यथा" => Token::Anyatha,
            "यावत्" => Token::Yavat,
            "तावत्" => Token::Tavat,
            "प्रतिफल" => Token::Pratiphala,
            // Concurrency / FFI
            "असमकाल" => Token::AsyncKW,
            "प्रतीक्षस्व" => Token::AwaitKW,
            "सहप्रक्रिया" => Token::Spawn,
            "विलम्बः" => Token::Delay,
            "वैदेशिक" => Token::Foreign,
            "विदेशीय" => Token::ForeignKW,
            // Borrow semantics
            "सन्दर्भः" => Token::Reference,
            "परिवर्त्य" => Token::Mutable,
            // Boolean literals
            "सत" => Token::Literal(crate::vm::Value::Bool(true)),
            "असत" => Token::Literal(crate::vm::Value::Bool(false)),
            // Boolean operators
            "च" => Token::Cha,
            "वा" => Token::Va,
            // Comparison operators
            "समान" => Token::Samana,
            "ऊन" => Token::Una,
            "अग्र" => Token::Agra,
            "ऊनसमान" => Token::UnaSamana,
            "अग्रसमान" => Token::AgraSamana,
            "असमान" => Token::Asamana,
            _ => Token::Identifier(text),
        };
        SpannedToken { token, line, col }
    }
}
