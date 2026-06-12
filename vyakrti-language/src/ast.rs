/// Access modifiers mapped from Vedic Svara markers.
/// उदात्त = public, अनुदात्त = private, स्वरित = protected.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AccessModifier {
    Public,    // उदात्त
    #[default]
    Private,   // अनुदात्त
    Protected, // स्वरित
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    VarDecl {
        name: String,
        data_type: Option<String>,
        value: Expression,
        access_level: AccessModifier,
    },
    FuncDecl {
        name: String,
        parameters: Vec<(String, String)>,
        return_type: String,
        body: Vec<ASTNode>,
        access_level: AccessModifier,
    },
    GenericFuncDecl {
        name: String,
        type_params: Vec<String>,
        parameters: Vec<(String, String)>,
        return_type: String,
        body: Vec<ASTNode>,
        access_level: AccessModifier,
    },
    AsyncFuncDecl {
        name: String,
        parameters: Vec<(String, String)>,
        return_type: String,
        body: Vec<ASTNode>,
        access_level: AccessModifier,
    },
    StructDecl {
        name: String,
        attributes: Vec<String>,
        fields: Vec<(String, String)>,
        access_level: AccessModifier,
    },
    TraitDecl {
        name: String,
        method_signatures: Vec<(String, Vec<(String, String)>, String)>,
        access_level: AccessModifier,
    },
    ImplBlock {
        trait_name: String,
        type_name: String,
        methods: Vec<ASTNode>,
    },
    EnumDecl {
        name: String,
        variants: Vec<(String, Option<Vec<String>>)>,
        access_level: AccessModifier,
    },
    IfStmt {
        condition: Expression,
        then_branch: Vec<ASTNode>,
        else_branch: Option<Vec<ASTNode>>,
    },
    WhileStmt {
        condition: Expression,
        body: Vec<ASTNode>,
    },
    ModuleDecl {
        name: String,
        body: Vec<ASTNode>,
    },
    ModuleDef {
        name: String,
        body: Vec<ASTNode>,
    },
    ImportDecl {
        file_path: String,
        namespace_alias: String,
    },
    SpawnStmt(Expression),
    DelayStmt(Expression),
    ReturnStmt(Expression),
    FFIDecl {
        library_path: String,
        symbol_name: String,
        parameters: Vec<(String, String)>,
        return_type: String,
        access_level: AccessModifier,
    },
    StatementExpr(Expression),
    Block(Vec<ASTNode>),
    MacroDecl {
        name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    },
    MacroCall {
        name: String,
        args: Vec<Expression>,
    },
    NoOp,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(crate::vm::Value),
    IntLiteral(i64),
    Variable(String),
    Binary {
        left: Box<Expression>,
        op: String,
        right: Box<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
    GenericCall {
        name: String,
        concrete_types: Vec<String>,
        args: Vec<Expression>,
    },
    MethodCall {
        instance: Box<Expression>,
        method_name: String,
        args: Vec<Expression>,
    },
    BorrowExpr {
        target: String,
        is_mutable: bool,
    },
    MatchExpr {
        eval_target: Box<Expression>,
        arms: Vec<(String, Vec<String>, Expression)>,
    },
    AwaitExpr {
        target_future: Box<Expression>,
    },
}
