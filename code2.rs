
// --- moc-common/src/ast.rs ---
use crate::decl::Decl;

pub type Ast = Vec<Decl>;

// --- moc-common/src/debug_utils.rs ---
use crate::token::{Token};

pub fn print_tokens(tokens: &[Token]) {
    for token in tokens {
        println!("{}", token);
    }
}

pub const INDENT: &str = "  ";
pub fn create_indent(depth: usize) -> String {
    format!("\n{}", INDENT.repeat(depth))
}


// --- moc-common/src/decl.rs ---
use serde::Serialize;

use crate::{
    CodeBlock, CodeSpan, TypedVar, expr::{Expr, GenericParam, Ident, TraitBound, TypeExpr}
};

#[derive(Debug, Clone, Serialize)]
pub struct FnSignature {
    pub ident: String,
    pub generics: Vec<GenericParam>, // Using the struct from our last step!
    pub params: Vec<TypedVar>,
    pub return_type: Option<TypeExpr>,
}

#[derive(Debug, Clone, Serialize)]
pub enum DeclKind {
    Fn {
        // function declaration
        signature: FnSignature,
        body: CodeBlock,
    },
    Use {
        path: Ident,
        alias: Option<String>,
    },
    Struct {
        ident: String,
        fields: Vec<TypedVar>,
        generics: Vec<GenericParam>, // e.g., ["T", "U"]
        impl_traits: Vec<TraitBound>,     // NEW: e.g., [PartialOrd]
    },
    Sum {
        ident: String,
        generics: Vec<GenericParam>,
        impl_traits: Vec<TraitBound>,
        variants: Vec<Variant>,
    },
    Trait {
        ident: String,
        generics: Vec<GenericParam>, // Traits can be generic too!
        methods: Vec<FnSignature>,
    },
    // global variable/constant?
    // Only for debugging stuff! If I want to just test parsing expressions without all the other shebang.
    LooseExpr(Expr),
}

#[derive(Debug, Clone, Serialize)]
pub struct Decl {
    pub span: CodeSpan,
    pub kind: DeclKind,
}

impl Decl {
    pub fn new(kind: DeclKind, span: CodeSpan) -> Self {
        Self { kind, span }
    }
}

// sum type variant stuff
#[derive(Debug, Clone, Serialize)]
pub enum VariantData {
    Unit,
    Tuple(Vec<TypeExpr>),
    Struct(Vec<TypedVar>),
}

#[derive(Debug, Clone, Serialize)]
pub struct Variant {
    pub ident: String,
    pub data: VariantData,
}


// --- moc-common/src/error.rs ---
use std::io;
use thiserror::Error;
use crate::{CodeSpan, ast::Ast, expr::Expr, token::Token};

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error(transparent)]
    ParserError(#[from] ParserError),
    #[error(transparent)]
    LexerError(#[from] LexerError),
    #[error("File operation failed")]
    FileNotFound(#[from] io::Error)
}

#[derive(Debug, Clone, Error)]
pub enum LexerError {
    #[error("Unterminated string literal")]
    UnterminatedStringLiteral(CodeSpan),

    #[error("Invalid character '{0}'")]
    InvalidCharacter(char, CodeSpan),

    #[error("Unknown escape character")]
    UnknownEscapeCharacter(CodeSpan),

    #[error("Multiple decimal points in number literal")]
    MultiDecimalPointInNumberLiteral(CodeSpan),

    #[error("Unexpected character while lexing non-decimal number literal")]
    UnexpectedCharacterLexingNonDecimalNumberLiteral(CodeSpan),

    #[error("Unknown token encountered")]
    UnknownToken(CodeSpan),
}

#[derive(Debug, Clone, Error)]
pub enum ParserError {
    #[error("{msg}")]
    UnexpectedToken {
        msg: String,
        peeking: Option<Token>,
        span: CodeSpan,
    },
}
impl ParserError {
    pub fn unexpected_token(msg: &str, peeking: Option<Token>, span: CodeSpan) -> Self {
        Self::UnexpectedToken { msg: msg.into(), peeking, span }
    }
    pub fn wrap<T>(self) -> Result<T, ParserError> {
        Err(self)
    }
}

pub type LexerResult = Result<Token, LexerError>;
pub type ParseResult = Result<Ast, ParserError>;
pub type ExprParseResult = Result<Expr, ParserError>;


// --- moc-common/src/lib.rs ---
pub mod ast;
pub mod debug_utils;
pub mod decl;
pub mod error;
pub mod expr;
pub mod op;
pub mod stmt;
pub mod token;

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
};

use derive_more::Display;
use serde::Serialize;

use crate::{
    expr::TypeExpr,
    stmt::Stmt,
};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct CodeLocation {
    pub line: usize,
    pub column: usize,
}

impl CodeLocation {
    pub fn is_in_same_line(&self, other: &Self) -> bool {
        self.line == other.line
    }
}

impl Default for CodeLocation {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl Display for CodeLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.line, self.column))
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CodeSpan {
    pub start: CodeLocation,
    pub end: CodeLocation,
}
#[derive(Debug, Display)]
pub struct ExpandCodeSpanError;
impl std::error::Error for ExpandCodeSpanError {}

impl CodeSpan {
    pub fn is_single_line(&self) -> bool {
        self.start.line == self.end.line
    }

    pub fn length(&self) -> usize {
        self.end.column - self.start.column
    }

    pub fn try_extend_left(&mut self, chars: usize) -> Result<(), ExpandCodeSpanError> {
        if self.start.column - chars > 0 {
            self.start.column -= chars;
            Ok(())
        } else {
            Err(ExpandCodeSpanError)
        }
    }

    pub fn extend_right(&mut self, chars: usize) {
        self.start.column += chars;
    }

    pub fn try_extend_both_sides(&mut self, chars: usize) -> Result<(), ExpandCodeSpanError> {
        self.try_extend_left(chars)?;
        self.extend_right(chars);
        Ok(())
    }

    /// Creates a new span that encompasses both `self` and `other`
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }
}

impl From<(CodeLocation, CodeLocation)> for CodeSpan {
    fn from(value: (CodeLocation, CodeLocation)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeBlock {
    pub stmts: Vec<Stmt>,
}

impl CodeBlock {
    pub fn new() -> Self {
        Self { stmts: Vec::new() }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ModulePath {
    pub path: VecDeque<String>,
}

impl ModulePath {
    pub fn new(module_path: VecDeque<String>) -> Self {
        ModulePath { path: module_path }
    }

    pub fn from_slice(module_path: &[String]) -> Self {
        ModulePath {
            path: module_path.iter().cloned().collect(),
        }
    }

    /// Like for example: The string "mod:submod:my_function" will yield a ModuleIdentifier with dirs: mod:submod
    pub fn from_qualified_item_identifier(ident: &str) -> Self {
        let mut path: VecDeque<String> = ident
            .split_terminator(":")
            .map(|path_dir| path_dir.to_string())
            .collect();
        path.pop_back();
        ModulePath { path }
    }
    pub fn from_string(ident: &str) -> Self {
        let path = ident
            .split_terminator(":")
            .map(|path_dir| path_dir.to_string())
            .collect();
        ModulePath { path }
    }

    pub fn remove_and_get_last_path(&mut self) -> String {
        let suffix = self
            .path
            .pop_back()
            .expect("ModulePath should not be empty.");
        suffix
    }
}

impl Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ModuleIdentifier")?;
        self.path.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedVar {
    /// the variable's identifier
    ident: String,
    /// the type identifier
    type_expr: TypeExpr,
}

impl TypedVar {
    pub fn new(ident: String, type_expr: TypeExpr) -> Self {
        Self { ident, type_expr }
    }
}


// --- moc-common/src/stmt.rs ---
use serde::Serialize;

use crate::{CodeBlock, CodeSpan, expr::{Expr, TypeExpr}, op::BinaryOp};

#[derive(Clone, Debug, Serialize)]
pub enum StmtKind {
    Print(Expr), // probably dont wanna have this as inbuilt function
    // a i32 (declaring variable)
    LocalVarDecl {
        ident: String,
        type_expr: TypeExpr
    },
    // <expr> = <expr> (updating value)
    Assignmt {
        assignee: Expr,
        new_value: Expr
    },
    // a += 10, a -= 10 etc. (operating and assigning)
    VarOperatorAssign {
        assignee: Expr,
        operator: BinaryOp,
        value: Expr,
    },
    // a i32 := 10 OR a := 10 (infers type)
    LocalVarDeclAssign {
        ident: String,
        type_expr: Option<TypeExpr>,
        value: Expr,
    },
    Break {
        value: Option<Expr>
    },
    Defer(Box<Stmt>),
    Expr(Expr), // expression statement (like function call)


    Ret(Option<Expr>), // return statement
    CodeBlock(CodeBlock),
    Next,
}

#[derive(Clone, Debug, Serialize)]
pub struct Stmt {
    span: CodeSpan,
    kind: StmtKind,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: CodeSpan) -> Self {
        Self { kind, span }
    }
}


// --- moc-common/src/token.rs ---
use std::fmt::Display;

use derive_more::Display;
use serde::Serialize;

use crate::{
    CodeSpan,
    op::{BinaryOp, UnaryOp},
};

#[macro_export]
macro_rules! token {
    ($token_kind:ident, $start:expr, $end:expr) => {
        $crate::token::Token::new(
            $crate::token::TokenKind::$token_kind,
            CodeSpan::from(($start, $end)),
        )
    };
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub value: Option<String>,
    pub span: CodeSpan,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.value() {
            write!(
                f,
                "{} from {} to {}, value: \"{}\"",
                self.kind,
                self.span.start,
                self.span.end,
                value.escape_default()
            )
        } else {
            write!(
                f,
                "{} from {} to {}",
                self.kind, self.span.start, self.span.end
            )
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize)]
pub enum TokenKind {
    AddAssign,           // +=
    Ampersand,           // &
    Equals,              // =
    At,                  // @
    BitAndAssign,        // &=
    BitOrAssign,         //  |=
    BitXorAssign,        // ^=
    BitNotAssign,        // ~=
    BitShiftLeft,        // <<
    BitShiftRight,       // >>
    BitShiftLeftAssign,  // <<=
    BitShiftRightAssign, // >>=
    Break,               // keyword
    Caret,               // ^
    CloseBrace,
    CloseParen,
    OpenBrack,
    CloseBrack,
    Colon,
    Comma,
    DeclareAssign, // :=
    Defer,
    DivAssign, // /=
    Dot,
    DoubleEquals,
    Excl, // !
    Else,
    EndOfFile,
    False,
    Fn,
    For,
    Greater,
    GreaterOrEqual,
    Ident,
    If,
    In,
    Impl,
    Is,
    Less,
    LessOrEqual,
    LineBreak, // encompassing CRLF and LF in one token.
    Loop,
    Minus,
    Percent,
    Pipe,
    ModAssign,
    MultAssign,
    Next, // keyword, like 'continue' in other languages
    ExclEquals,
    DecimalIntegerNumberLiteral,
    DecimalPointNumberLiteral,
    HexadecimalIntegerNumberLiteral,
    OctalIntegerNumberLiteral,
    BinaryIntegerNumberLiteral,
    ScientificDecimalNumberLiteral, // 1e9 = 1,000,000,000 | 1e-9 = 0.000000001 | 0.1e9 = 100,000,000 | 0.1e-9 = 0.0000000001
    ScientificHexNumberLiteral,     // 0x10p10 =  0x10 * 2^10 | 0x10p-10 = 0x10 * 2^-10
    OpenBrace,
    OpenParen,
    Plus,
    Ret,
    Semicolon,
    Slash,
    StringLiteral,
    Star,
    Struct,
    SubAssign,
    Sum,
    Tilde, // ~
    Trait,
    True,
    Use,
}

#[derive(Debug, Clone, Copy, Display, Serialize, Eq, PartialEq)]
pub enum NumberLiteralKind {
    DecimalInteger,
    DecimalPoint, // like floating point
    BinaryInteger,
    OctalInteger,
    HexadecimalInteger,
    ScientificDecimal, // 1e9 = 1,000,000,000 | 1e-9 = 0.000000001 | 0.1e9 = 100,000,000 | 0.1e-9 = 0.0000000001
    ScientificHex,     // 0x10p10 =  0x10 * 2^10 | 0x10p-10 = 0x10 * 2^-10
}

impl NumberLiteralKind {
    pub fn get_radix(&self) -> u32 {
        match self {
            NumberLiteralKind::DecimalInteger
            | NumberLiteralKind::ScientificDecimal
            | NumberLiteralKind::DecimalPoint => 10,
            NumberLiteralKind::BinaryInteger => 2,
            NumberLiteralKind::OctalInteger => 8,
            NumberLiteralKind::HexadecimalInteger | NumberLiteralKind::ScientificHex => 16,
        }
    }

    pub fn get_token_type(&self) -> TokenKind {
        match self {
            NumberLiteralKind::DecimalInteger => TokenKind::DecimalIntegerNumberLiteral,
            NumberLiteralKind::DecimalPoint => TokenKind::DecimalPointNumberLiteral,
            NumberLiteralKind::BinaryInteger => TokenKind::BinaryIntegerNumberLiteral,
            NumberLiteralKind::OctalInteger => TokenKind::OctalIntegerNumberLiteral,
            NumberLiteralKind::HexadecimalInteger => TokenKind::HexadecimalIntegerNumberLiteral,
            NumberLiteralKind::ScientificDecimal => TokenKind::ScientificDecimalNumberLiteral,
            NumberLiteralKind::ScientificHex => TokenKind::ScientificHexNumberLiteral,
        }
    }
}

impl Token {
    pub fn string_literal(literal: String, span: CodeSpan) -> Self {
        Self {
            kind: TokenKind::StringLiteral,
            value: Some(literal.into()),
            span,
        }
    }
    pub fn ident(ident: String, span: CodeSpan) -> Self {
        Self {
            kind: TokenKind::Ident,
            value: Some(ident.into()),
            span,
        }
    }
    pub fn integer(value: String, span: CodeSpan) -> Self {
        Self {
            kind: TokenKind::DecimalIntegerNumberLiteral,
            value: Some(value),
            span,
        }
    }
    pub fn number_literal(
        value: String,
        number_literal_type: NumberLiteralKind,
        span: CodeSpan,
    ) -> Self {
        Self {
            kind: number_literal_type.get_token_type(),
            value: Some(value),
            span,
        }
    }
    pub fn new(kind: TokenKind, span: CodeSpan) -> Self {
        Self {
            kind,
            value: None,
            span,
        }
    }
    pub fn with_value(r#type: TokenKind, value: String, span: CodeSpan) -> Self {
        Self {
            kind: r#type,
            value: Some(value),
            span,
        }
    }
    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    /// # Panics
    /// Panics if no value is present
    pub fn unwrap_value(&self) -> String {
        self.value.as_ref().expect("Expected value").clone()
    }

    pub fn is_assignment_operator(&self) -> bool {
        use TokenKind::*;
        matches!(
            self.kind,
            Equals
                | DeclareAssign
                | AddAssign
                | SubAssign
                | MultAssign
                | ModAssign
                | DivAssign
                | BitAndAssign
                | BitXorAssign
                | BitNotAssign
                | BitOrAssign
                | BitShiftLeftAssign
                | BitShiftRightAssign
        )
    }

    pub fn is_binary_op(&self) -> bool {
        BinaryOp::try_from(self.kind).is_ok()
    }

    pub fn is_unary_op(&self) -> bool {
        UnaryOp::try_from(self.kind).is_ok()
    }

    pub fn is_of_kind(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    pub fn is_of_any_kinds(&self, types: &[TokenKind]) -> bool {
        for r#type in types {
            if self.is_of_kind(*r#type) {
                return true;
            }
        }
        false
    }

    pub fn is_number_literal(&self) -> bool {
        TryInto::<NumberLiteralKind>::try_into(self.kind).is_ok()
    }

    pub fn infix_binding_power(&self) -> Option<(u8, u8)> {
        if let Some(binary_op) = BinaryOp::try_from(self.kind).ok() {
            return Some(binary_op.infix_binding_power());
        }
        use TokenKind::*;
        match self.kind {
            OpenParen => Some((110, 0)), // 0 on the right because it's postfix/special
            Dot => Some((120, 0)),
            OpenBrack => Some((110, 0)),
            TokenKind::Colon => Some((9, 10)),
            DeclareAssign | Equals | AddAssign | SubAssign | MultAssign | DivAssign | ModAssign
            | BitAndAssign | BitOrAssign | BitNotAssign | BitXorAssign | BitShiftLeftAssign
            | BitShiftRightAssign => Some((10, 9)), // Right-associative assignment
            _ => None,
        }
    }
}

impl From<TokenKind> for Token {
    fn from(r#type: TokenKind) -> Self {
        Self {
            kind: r#type,
            value: None,
            span: CodeSpan::default(),
        }
    }
}

impl From<NumberLiteralKind> for TokenKind {
    fn from(value: NumberLiteralKind) -> Self {
        value.get_token_type()
    }
}

#[derive(Debug)]
pub struct NonNumberLiteralTokenTypeError;
impl TryFrom<TokenKind> for NumberLiteralKind {
    type Error = NonNumberLiteralTokenTypeError;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::DecimalIntegerNumberLiteral => Ok(NumberLiteralKind::DecimalInteger),
            TokenKind::DecimalPointNumberLiteral => Ok(NumberLiteralKind::DecimalPoint),
            TokenKind::BinaryIntegerNumberLiteral => Ok(NumberLiteralKind::BinaryInteger),
            TokenKind::OctalIntegerNumberLiteral => Ok(NumberLiteralKind::OctalInteger),
            TokenKind::HexadecimalIntegerNumberLiteral => Ok(NumberLiteralKind::HexadecimalInteger),
            TokenKind::ScientificDecimalNumberLiteral => Ok(NumberLiteralKind::ScientificDecimal),
            TokenKind::ScientificHexNumberLiteral => Ok(NumberLiteralKind::ScientificHex),
            _ => Err(NonNumberLiteralTokenTypeError),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CodeLocation;

    use super::*;

    #[test]
    fn is_of_type_s() {
        let token = token!(Plus, CodeLocation::default(), CodeLocation::default());
        assert!(token.is_of_kind(TokenKind::Plus));
        assert!(token.is_of_any_kinds(&[TokenKind::Plus, TokenKind::Minus]));
        assert!(token.is_of_any_kinds(&[TokenKind::Minus, TokenKind::Plus]));
    }
}


// --- moc-common/src/expr.rs ---
use serde::Serialize;

use crate::{
    CodeBlock, CodeLocation, CodeSpan, ModulePath, op::{BinaryOp, UnaryOp}, token::{NumberLiteralKind, TokenKind}
};

#[derive(Debug, Clone, Serialize)]
pub enum ExprKind {
    // Expressions
    Assign {
        assignee: Box<Expr>,
        operator: TokenKind, // TODO: maybe make this use it's own enum type like AssignmentOp?
        value: Box<Expr>,
    },
    Binary {
        left_expr: Box<Expr>,
        operator: BinaryOp,
        right_expr: Box<Expr>,
    },
    FieldAccess {
        called_on: Box<Expr>,
        member_ident: Ident,
    },
    ArrayLiteral {
        elements: Vec<Expr>,
    },
    ArrayAccessor {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    ForLoop {
        condition: Option<Box<Expr>>, // if None, is infinite loop
        code_block: CodeBlock,
    },
    If {
        condition: Box<Expr>,
        if_block: CodeBlock,
        else_block: Option<CodeBlock>,
    },
    BoolLiteral(bool),
    Grouping(Box<Expr>),
    FnCall {
        callee: Box<Expr>, // callee / what value, what expression the function is being called on.
        args: Vec<Expr>,
    },

    // this is only part. A full generic FnCall consists of FnCall with callee being an expr of type GenericFnCallPart.
    GenericFnCallPart {
        callee: Box<Expr>,
        type_args: Option<Vec<TypeExpr>>,
    },
    Variable {
        ident: Ident,
    },
    NumberLiteral(String, NumberLiteralKind),
    StringLiteral(String),
    Unary {
        operator: UnaryOp,
        expr: Box<Expr>,
    }, // Operator followed by another expr
    Empty,
}

#[derive(Clone, Debug, Serialize)]
pub struct Expr {
    pub span: CodeSpan,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, Serialize)]
pub enum Ident {
    Simple(String),
    WithModulePrefix(ModulePath, String),
}

impl Ident {
    /// Gets the ident if of variant Simple, else gets the suffix.
    pub fn base(&self) -> &String {
        match self {
            Ident::Simple(ident) => &ident,
            Ident::WithModulePrefix(_, suffix) => &suffix,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum TypeExpr {
    Ident(Ident),
    Pointer(Box<TypeExpr>),
    Array {
        length: Option<usize>,
        type_expr: Box<TypeExpr>,
    },
    Generic {
        ident: Ident,
        params: Vec<TypeExpr>, // identifiers of generic type parameters
    },
}

// used in struct/sum declarations impl items.
// struct A impl Trait[i32]
// (the [T] is optional. only for generic traits)
// maybe rename to TraitImplDecl or something. I need to sleep.
#[derive(Debug, Clone, Serialize)]
pub struct TraitBound {
    pub ident: Ident,
    pub args: Vec<TypeExpr>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenericParam {
    pub ident: String,
    pub bounds: Option<Vec<Ident>>,
}

impl TypeExpr {
    pub fn pointer(type_expr: Self) -> Self {
        Self::Pointer(Box::new(type_expr))
    }
}

impl Expr {
    pub fn new(kind: ExprKind, span: CodeSpan) -> Self {
        Self { kind, span }
    }

    pub fn binary(left: Self, operator: BinaryOp, right: Self) -> Self {
        let span = left.span.merge(right.span);
        Self::new(
            ExprKind::Binary {
                left_expr: Box::new(left),
                operator,
                right_expr: Box::new(right),
            },
            span,
        )
    }

    pub fn unary(start: CodeLocation, operator: UnaryOp, right: Self) -> Self {
        let span = (start, (&right.span).end).into();
        Expr::new(ExprKind::Unary {
            operator,
            expr: Box::new(right),
        }, span)
    }
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}


// --- moc-common/src/op.rs ---
//! An operator is something that can form an expression together with other expressions.
//! Unary operators create unary expressions while binary operators create binary expressions.

use derive_more::Display;
use serde::Serialize;

use crate::token::TokenKind;

#[derive(Clone, Copy, Debug, Display, Serialize)]
pub enum UnaryOp {
    Negative, // -
    Not,      // ! Logical NOT
    BitNot,   // ~ Bitwise NOT
    Deref,    // * Postfix deref (pointee = pointer.*)
    AddressOf,// &
}

#[derive(Debug)]
pub struct TokenNotAUnaryOpError;

impl TryFrom<TokenKind> for UnaryOp {
    type Error = TokenNotAUnaryOpError;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Minus => Ok(UnaryOp::Negative),
            TokenKind::Excl => Ok(UnaryOp::Not),
            TokenKind::Tilde => Ok(UnaryOp::BitNot),
            TokenKind::Ampersand => Ok(UnaryOp::AddressOf),
            _ => Err(TokenNotAUnaryOpError),
        }
    }
}

impl UnaryOp {
    /// Returns the binding power for prefix operators.
    /// Higher than most binary operators so that `-a.b` is `-(a.b)`
    /// but lower than primary expressions.
    pub fn prefix_binding_power(&self) -> u8 {
        // We use 17 here because our highest binary (Mult/Div) is 15/16.
        17
    }
}

#[derive(Clone, Copy, Debug, Display, Serialize)]
pub enum BinaryOp {
    Add,
    Sub,           // Subtract
    Mult,          // Multiply
    Div,           // Divide
    Mod,           // Modulo
    BitShiftLeft,  // Bitshift left
    BitShiftRight, // Bitshift right
    BitOr,         // Bitwise OR
    BitAnd,        // Bitwise AND
    BitXor,        // Bitwise XOR
    Greater,
    Less,
    Equal,
    NotEqual,
    GreaterOrEqual,
    LessOrEqual,
}

impl BinaryOp {
    // lower binding power means lower precedence. 
    // Operators with higher precedence are evaluated before those with lower precedence in an expression. 
    // For example, multiplication has higher precedence than addition, so in 3 + 4 * 5, the multiplication is performed first.
    
    
    pub fn infix_binding_power(&self) -> (u8, u8) {
        use BinaryOp::*;
        match self {
            // Priority 1: Equality
            Equal | NotEqual => (1, 2), // Equality makes sense to be evaluated after the two sides have already been evaluated, therefore low precedence

            // Priority 2: Comparisons
            Greater | Less | GreaterOrEqual | LessOrEqual => (3, 4),

            // Priority 3: Bitwise Logic
            BitOr => (5, 6),
            BitXor => (7, 8),
            BitAnd => (9, 10),

            // Priority 4: Shifts
            BitShiftLeft | BitShiftRight => (11, 12),

            // Priority 5: Sums
            Add | Sub => (13, 14),

            // Priority 6: Products
            Mult | Div | Mod => (15, 16),
        }
    }
}

#[derive(Debug)]
pub struct TokenNotABinaryOpError;
impl TryFrom<TokenKind> for BinaryOp {
    type Error = TokenNotABinaryOpError;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        use BinaryOp::*;
        match value {
            TokenKind::Plus => Ok(Add),
            TokenKind::Minus => Ok(Sub),                     // Subtract
            TokenKind::Star => Ok(Mult),                     // Multiply
            TokenKind::Slash => Ok(Div),                     // Divide
            TokenKind::Percent => Ok(Mod),                   // Modulo
            TokenKind::BitShiftLeft => Ok(BitShiftLeft),     // Bitshift left
            TokenKind::BitShiftRight => Ok(BitShiftRight),   // Bitshift right
            TokenKind::Pipe => Ok(BitOr),                    // Bitwise OR
            TokenKind::Ampersand => Ok(BitAnd),              // Bitwise AND
            TokenKind::Caret => Ok(BitXor),                  // Bitwise XOR
            TokenKind::DoubleEquals => Ok(Equal),            // ==
            TokenKind::ExclEquals => Ok(NotEqual),           // !=
            TokenKind::Greater => Ok(Greater),               // >
            TokenKind::Less => Ok(Less),                     // <
            TokenKind::GreaterOrEqual => Ok(GreaterOrEqual), // >=
            TokenKind::LessOrEqual => Ok(LessOrEqual),       // <=
            _ => Err(TokenNotABinaryOpError),
        }
    }
}


// --- moc-cli/src/main.rs ---
use clap::Parser;
use moc_common::debug_utils;
use moc_main::CompilerOptions;
use ron::{extensions::Extensions, ser::PrettyConfig};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to .mo file
    #[arg(short, long)]
    path: String,
    /// Whether to print the token output of the lexer
    #[arg(long)]
    print_tokens: bool,
    /// Whether to print the AST output of the parser
    #[arg(long)]
    print_ast: bool,
}

fn main() {
    simple_logger::init_with_env().unwrap(); // just for debug toggleable debug logging

    let args = Args::parse();
    let mut options = CompilerOptions::default();
    options.emit_tokens = args.print_tokens;
    options.emit_ast = args.print_ast;
    let result = moc_main::compile_file(args.path, options);

    if let Some(tokens) = result.tokens {
        debug_utils::print_tokens(&tokens);
    }
    if let Some(ast) = result.ast {
        let pretty_config = PrettyConfig::default()
            .compact_arrays(true)
            .escape_strings(true)
            .separate_tuple_members(false);
        ron::Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES | Extensions::UNWRAP_VARIANT_NEWTYPES)
            .to_io_writer_pretty(std::io::stdout(), &ast, pretty_config)
            .expect("Error writing to stdout");
        println!(); // add newline char
    }
    if !result.errors.is_empty() {
        for error in result.errors {
            println!("Compiler error: {:?}", error)
        }
    }
}


// --- moc-main/src/lib.rs ---
use std::{fs::File, io::Read, path::Path};

use moc_common::{ast::Ast, error::CompilerError, token::Token};
use moc_parser::{lexer::Lexer, parser::Parser};

pub struct CompileResultData {
    pub tokens: Option<Vec<Token>>,
    pub ast: Option<Ast>,
    pub errors: Vec<CompilerError>,
}

impl CompileResultData {
    pub fn new() -> Self {
        Self {
            tokens: None,
            ast: None,
            errors: Vec::new(),
        }
    }
}

pub struct CompilerOptions {
    pub emit_ast: bool,
    pub emit_tokens: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            emit_ast: false,
            emit_tokens: false,
        }
    }
}

pub fn compile_file(path: impl AsRef<Path>, options: CompilerOptions) -> CompileResultData {
    let mut src = String::new();
    let mut meta_data = CompileResultData::new();
    match File::open(path) {
        Ok(mut file) => {
            file.read_to_string(&mut src).unwrap(); // TO-DO: Handle error
            let mut lexer = Lexer::new(&src);
            let tokens = lexer.tokens();
            for lexer_error in lexer.errors {
                meta_data
                    .errors
                    .push(CompilerError::LexerError(lexer_error));
            }
            if options.emit_tokens {
                meta_data.tokens = Some(tokens.clone()); // TODO: Find out how how to not clone all the tokens
            }
            println!("Parsing now");

            let only_expr = false; // TODO: Make this a compiler option
            let mut parser = Parser::new(tokens, only_expr);
            let ast = parser.parse();
            println!("Done parsing");

            if options.emit_ast {
                meta_data.ast = Some(ast.clone());
            }
            for parser_error in &parser.errors {
                meta_data
                    .errors
                    .push(CompilerError::ParserError(parser_error.clone()));
            }

            if parser.errors.len() == 0 {
                // TODO: continue into semantic analysis
            }
        }
        Err(io_error) => {
            meta_data.errors.push(CompilerError::FileNotFound(io_error));
        }
    }
    meta_data
}


// --- moc-parser/src/lib.rs ---
pub mod lexer;
pub mod parser;


// --- moc-parser/src/lexer.rs ---
use std::str::Chars;

use itertools::{PeekNth, peek_nth};
use log::debug;
use moc_common::error::{LexerError, LexerResult};
use moc_common::token::{NumberLiteralKind, Token, TokenKind, TokenKind::*};
use moc_common::{CodeLocation, CodeSpan};

#[derive(Clone)]
pub struct Lexer<'a> {
    chars: PeekNth<Chars<'a>>,
    last_token_end: CodeLocation,
    location: CodeLocation,
    pub errors: Vec<LexerError>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: peek_nth(input.chars()),
            location: CodeLocation::default(),
            last_token_end: CodeLocation::default(),
            errors: Vec::new(),
        }
    }

    pub fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::with_capacity(256);
        loop {
            match self.next_token() {
                Ok(token) => if token.kind == EndOfFile {
                    break;
                } else {
                    tokens.push(token);
                },
                Err(error) => self.errors.push(error),
            }
        }
        tokens
    }

    fn current_span(&self) -> CodeSpan {
        CodeSpan::from((self.last_token_end, self.location))
    }

    fn new_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.current_span())
    }

    fn count_line_break(&mut self) {
        self.location.column = 1;
        self.location.line += 1;
    }

    fn update_span(&mut self) {
        self.last_token_end = self.location;
    }

    fn advance(&mut self) -> Option<char> {
        self.location.column += 1;
        let char = self.chars.next();
        debug!("lexing char: {}", char.unwrap_or('E').escape_default());
        char
    }

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().cloned()
    }
    fn peek_nth_char(&mut self, n: usize) -> Option<char> {
        self.chars.peek_nth(n).cloned()
    }

    fn lex_linebreak(&mut self, ch: char) -> LexerResult {
        if ch == '\n' {
            return Ok(self.lex_lf());
        }
        self.lex_crlf()
    }

    fn lex_crlf(&mut self) -> Result<Token, LexerError> {
        self.advance();
        if self.peek_char() == Some('\n') {
            self.advance();
            self.update_span();
            self.count_line_break();
            return Ok(self.new_token(LineBreak));
        }
        // Only single \r found. Maybe emit warning in the future
        Err(LexerError::UnknownToken(self.current_span()))
    }

    fn lex_lf(&mut self) -> Token {
        self.advance();
        self.update_span();
        self.count_line_break();
        self.new_token(LineBreak)
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.update_span();
        let lexer_result: LexerResult = loop {
            if let Some(ch) = self.peek_char() {
                match ch {
                    '.' => {
                        self.advance();
                        break Ok(self.new_token(Dot));
                    }
                    '!' => {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            break Ok(self.new_token(ExclEquals));
                        }
                        break Ok(self.new_token(Excl));
                    }
                    '@' => {
                        self.advance();
                        break Ok(self.new_token(At));
                    }
                    ',' => {
                        self.advance();
                        break Ok(self.new_token(Comma));
                    }
                    '/' => {
                        self.advance();
                        if self.peek_char() == Some('/') {
                            // two slashes means a comment
                            if let Some(token) = self.skip_comment() {
                                return token;
                            } else {
                                continue;
                            }
                        }
                        break self.lex_operator(ch); // if we have a single slash, it's a divide operator
                    }
                    ' ' | '\t' => {
                        self.advance();
                        self.update_span(); // for every 'skipped' chararacter that doesnt result a token, we need to move the "last_token_end" cursor to the lexer's position.
                        continue;
                    } // Skip non-relevant whitespace
                    '\r' | '\n' => {
                        return self.lex_linebreak(ch);
                    }
                    '{' => {
                        self.advance();
                        break Ok(self.new_token(OpenBrace));
                    }
                    '}' => {
                        self.advance();
                        break Ok(self.new_token(CloseBrace));
                    }
                    '[' => {
                        self.advance();
                        break Ok(self.new_token(OpenBrack));
                    }
                    ']' => {
                        self.advance();
                        break Ok(self.new_token(CloseBrack));
                    }
                    '(' => {
                        self.advance();
                        break Ok(self.new_token(OpenParen));
                    }
                    ')' => {
                        self.advance();
                        break Ok(self.new_token(CloseParen));
                    }
                    ':' => {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            break Ok(self.new_token(DeclareAssign));
                        }
                        break Ok(self.new_token(Colon));
                    }
                    ';' => {
                        self.advance();
                        break Ok(self.new_token(Semicolon));
                    }
                    '=' => {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            break Ok(self.new_token(DoubleEquals));
                        }
                        break Ok(self.new_token(Equals));
                    }
                    '+' | '-' | '*' | '%' | '&' | '|' | '~' | '^' | '<' | '>' => {
                        break self.lex_operator(ch);
                    }
                    '0'..='9' => break self.lex_number(),
                    'a'..='z' | 'A'..='Z' | '_' => break Ok(self.lex_keyword_or_ident()),
                    '\"' => break self.lex_string_literal(),
                    _ => {
                        self.errors.push(LexerError::InvalidCharacter(ch, self.current_span()));
                    }
                }
            } else {
                return Ok(self.new_token(EndOfFile));
            }
        };

        return lexer_result;
    }

    fn lex_operator(&mut self, ch: char) -> LexerResult {
        self.advance();
        if ch == '<' {
            if self.peek_char() == Some('<') {
                // bit-shift left
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(self.new_token(BitShiftLeftAssign));
                }
                return Ok(self.new_token(BitShiftLeft));
            }
        }
        if ch == '>' {
            if self.peek_char() == Some('>') {
                // bit-shift right
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(self.new_token(BitShiftRightAssign));
                }
                return Ok(self.new_token(BitShiftRight));
            }
        }
        if self.peek_char() == Some('=') {
            let token_kind = match ch {
                '+' => AddAssign,
                '-' => SubAssign,
                '*' => MultAssign,
                '/' => DivAssign,
                '%' => ModAssign,
                '&' => BitAndAssign,
                '|' => BitOrAssign,
                '^' => BitXorAssign,
                '<' => LessOrEqual,
                '>' => GreaterOrEqual,
                '~' => BitNotAssign,
                _ => unreachable!(),
            };

            // is a 2 character operator
            self.advance();
            return Ok(self.new_token(token_kind));
        } else {
            let token_kind = match ch {
                '+' => Plus,
                '-' => Minus,
                '*' => Star,
                '/' => Slash,
                '%' => Percent,
                '&' => Ampersand,
                '|' => Pipe,
                '^' => Caret,
                '<' => Less,
                '>' => Greater,
                '~' => Tilde,
                _ => unreachable!(),
            };
            return Ok(self.new_token(token_kind));
        }
    }

    fn lex_number(&mut self) -> LexerResult {
        let mut num = String::new();

        let mut has_decimal_point = false;
        let mut has_exponent = false;
        let mut first_iter = true;

        while let Some(ch) = self.peek_char() {
            if first_iter && ch == '0' {
                if let Some(lex_result) = self.lex_non_decimal_literal(&mut num) {
                    return lex_result;
                } else {
                    num.push(ch);
                    self.advance();
                }
            } else if ch.is_digit(10) {
                num.push(ch);
                self.advance();
            } else if ch == '.' {
                if !has_decimal_point && !has_exponent {
                    has_decimal_point = true;
                    num.push(ch);
                    self.advance();
                } else {
                    return Err(LexerError::MultiDecimalPointInNumberLiteral(self.current_span()));
                }
            } else if ch == 'e' || ch == 'E' {
                if !has_exponent {
                    has_exponent = true;
                    num.push(ch);
                    self.advance();

                    // Exponents can optionally have a +/- sign
                    if let Some(next_ch) = self.peek_char() {
                        if next_ch == '+' || next_ch == '-' {
                            num.push(next_ch);
                            self.advance();
                        }
                    }
                } else {
                    break;
                }
            } else if ch == '_' {
                self.advance();
            } else {
                break;
            }
            first_iter = false;
        }

        // Determine the final token kind based on what we found
        let kind = if has_exponent {
            NumberLiteralKind::ScientificDecimal
        } else if has_decimal_point {
            NumberLiteralKind::DecimalPoint
        } else {
            // Assuming you have a standard integer kind, update this if your enum uses a different name
            NumberLiteralKind::DecimalInteger
        };

        Ok(Token::number_literal(
            num,
            kind,
            self.current_span(),
        ))
    }

    fn lex_non_decimal_literal(&mut self, num: &mut String) -> Option<Result<Token, LexerError>> {
        let prefix = self.peek_nth_char(1);
        if let Some(prefix) = prefix {
            match prefix {
                'x' => {
                    return Some(self.lex_non_decimal_literal_value(
                        num,
                        NumberLiteralKind::HexadecimalInteger,
                    ));
                }
                'o' => {
                    return Some(
                        self.lex_non_decimal_literal_value(num, NumberLiteralKind::OctalInteger),
                    );
                }
                'b' => {
                    return Some(
                        self.lex_non_decimal_literal_value(num, NumberLiteralKind::BinaryInteger),
                    );
                }
                '.' => return None,
                _ => {
                    if let Some(ch) = self.peek_nth_char(2) {
                        if ch.is_digit(16) {
                            return Some(Err(
                                LexerError::UnexpectedCharacterLexingNonDecimalNumberLiteral(
                                    self.current_span()
                                ),
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    fn lex_non_decimal_literal_value(
        &mut self,
        num: &mut String,
        literal_kind: NumberLiteralKind,
    ) -> LexerResult {
        // We capture the starting kind, but allow it to "upgrade" to ScientificHex
        let mut final_kind = literal_kind;
        let mut has_decimal = false;
        let mut has_exponent = false;

        self.advance_n(2);
        let mut first = true;

        while let Some(ch) = self.peek_char() {
            if ch == '_' {
                self.advance();
                continue;
            }

            if literal_kind == NumberLiteralKind::HexadecimalInteger {
                // Hex numbers can have A-F, a decimal '.', and an exponent 'p'/'P'
                if ch.is_digit(16) {
                    num.push(ch);
                    self.advance();
                } else if ch == '.' && !has_decimal && !has_exponent {
                    has_decimal = true;
                    final_kind = NumberLiteralKind::ScientificHex;
                    num.push(ch);
                    self.advance();
                } else if (ch == 'p' || ch == 'P') && !has_exponent {
                    has_exponent = true;
                    final_kind = NumberLiteralKind::ScientificHex;
                    num.push(ch);
                    self.advance();

                    // Hex exponents can also have a +/- sign
                    if let Some(next_ch) = self.peek_char() {
                        if next_ch == '+' || next_ch == '-' {
                            num.push(next_ch);
                            self.advance();
                        }
                    }
                } else {
                    break;
                }
            } else {
                // Binary and Octal fall back to the standard strict logic
                if ch.is_digit(literal_kind.get_radix()) {
                    num.push(ch);
                    self.advance();
                } else if ch.is_digit(16) || first {
                    return Err(
                        LexerError::UnexpectedCharacterLexingNonDecimalNumberLiteral(self.current_span()),
                    );
                } else {
                    break;
                }
            }
            first = false;
        }

        Ok(Token::number_literal(
            num.clone(),
            final_kind,
            (self.last_token_end, self.location).into(),
        ))
    }

    fn lex_keyword_or_ident(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        match ident.as_str() {
            "break" => self.new_token(Break),
            "defer" => self.new_token(Defer),
            "else" => self.new_token(Else),
            "false" => self.new_token(False),
            "fn" => self.new_token(Fn),
            "for" => self.new_token(For),
            "if" => self.new_token(If),
            "impl" => self.new_token(Impl),
            "in" => self.new_token(In),
            "is" => self.new_token(Is),
            "loop" => self.new_token(Loop),
            "next" => self.new_token(Next), // like continue
            "ret" => self.new_token(Ret),
            "struct" => self.new_token(Struct),
            "sum" => self.new_token(Sum),
            "true" => self.new_token(True),
            "trait" => self.new_token(Trait),
            "use" => self.new_token(Use),
            _ => Token::ident(ident, (self.last_token_end, self.location).into()),
        }
    }

    // Skips comment and returns a linebreak token if the comment ends with one.
    // maybe this is over-engineered because comments always end with a linebreak unless they are the last thing in a source file. But in that case it's irrelevant, maybe?
    fn skip_comment(&mut self) -> Option<Result<Token, LexerError>> {
        let mut line_break = None;
        while let Some(ch) = self.peek_char() {
            if ch == '\r' || ch == '\n' {
                line_break = Some(self.lex_linebreak(ch));
                break;
            } else {
                self.advance();
            }
        }
        self.update_span();
        line_break
    }

    // still need to add escaping of special characters. could do that later tho.
    fn lex_string_literal(&mut self) -> Result<Token, LexerError> {
        self.advance();
        let mut value = String::new();
        let mut ends_by_quote = false; // valid if string literal ends with another quote symbol "
        while let Some(ch) = self.peek_char() {
            self.advance();
            match ch {
                '\\' => {
                    self.handle_escape_character(&mut value)?;
                }
                '\"' => {
                    ends_by_quote = true;
                    break;
                }
                _ => {
                    value.push(ch);
                }
            }
        }
        if ends_by_quote {
            Ok(Token::string_literal(
                value,
                self.current_span(),
            ))
        } else {
            Err(LexerError::UnterminatedStringLiteral(self.current_span()))
        }
    }

    fn handle_escape_character(&mut self, literal: &mut String) -> Result<(), LexerError> {
        if let Some(ch) = self.peek_char() {
            let replacement = match ch {
                '\\' => Some('\\'),
                '\"' => Some('\"'),
                'r' => Some('\r'),
                'n' => Some('\n'),
                '0' => Some('\0'),
                't' => Some('\t'),
                '\'' => Some('\''),
                _ => None, // replace with proper error
            };
            if let Some(replacement) = replacement {
                self.advance();
                literal.push(replacement);
                return Ok(());
            }
        }
        Err(LexerError::UnknownEscapeCharacter(self.current_span()))
    }
}


// --- moc-parser/src/parser.rs ---
use std::{collections::VecDeque, usize, vec::IntoIter};

use itertools::{PeekNth, peek_nth};
use log::debug;
use moc_common::{
    CodeBlock, CodeSpan, ModulePath, TypedVar,
    ast::Ast,
    decl::{Decl, DeclKind, FnSignature, Variant, VariantData},
    error::{ExprParseResult, ParserError},
    expr::{Expr, ExprKind, GenericParam, Ident, TraitBound, TypeExpr},
    op::{BinaryOp, UnaryOp},
    stmt::{Stmt, StmtKind},
    token::{Token, TokenKind},
};

pub struct Parser {
    token_stream: PeekNth<IntoIter<Token>>,
    current_token: Option<Token>,
    ast: Ast,
    pub errors: Vec<ParserError>,
    only_expr: bool, // if true only parses an expresion, if false, parses full program structure.
}

/*
 * Writing a parser in Rust for a programming language. Now the Token struct has a codespan field where the lexer inserts where in the code (start line +column and end line + column) the Token is situated. Now I'd like to have something similar for AST nodes. But the way they are architected (with Rust enums) there is no easy way to add codespans to each ast node. Please help me with rearchitecting that.
 */

impl Parser {
    pub fn new(tokens: Vec<Token>, only_expr: bool) -> Self {
        let parser = Self {
            token_stream: peek_nth(tokens),
            current_token: None,
            ast: Ast::new(),
            errors: Vec::new(),
            only_expr,
        };
        parser
    }

    /// Creates a CodeSpan starting and ending at the current token
    pub fn start_span_at_current(&self) -> CodeSpan {
        self.unwrap_current_token().span
    }

    /// Creates a CodeSpan starting and ending at the next token
    pub fn start_span_at_next(&mut self) -> CodeSpan {
        self.peek().unwrap().span
    }

    /// Sets the given span's end to the end of the current token
    pub fn end_span(&self, span: &mut CodeSpan) {
        span.end = self.unwrap_current_token().span.end;
    }

    pub fn parse(&mut self) -> &Ast {
        if self.only_expr {
            match self.expr(0) {
                Ok(expr) => {
                    let span = expr.span;
                    self.ast.push(Decl::new(DeclKind::LooseExpr(expr), span));
                }
                Err(error) => self.errors.push(error),
            }
        } else {
            if let Err(error) = self.parse_top_level_decls() {
                self.errors.push(error);
            }
        }
        &self.ast
    }

    // PRATT PARSING >>>
    fn parse_prefix(&mut self) -> ExprParseResult {
        let token = self.advance().unwrap();
        use TokenKind::*;
        match token.kind {
            DecimalIntegerNumberLiteral
            | DecimalPointNumberLiteral
            | BinaryIntegerNumberLiteral
            | OctalIntegerNumberLiteral
            | HexadecimalIntegerNumberLiteral
            | ScientificDecimalNumberLiteral
            | ScientificHexNumberLiteral => Ok(Expr::new(
                ExprKind::NumberLiteral(
                    token.unwrap_value(),
                    token.kind.try_into().unwrap(), // converts the TokenKind into the NumberLiteralKind. Because we know the TokenKind is a number literal, the conversion shouldn't panic.
                ),
                token.span,
            )),
            Ident => {
                let mut span = self.start_span_at_current();
                let ident = self.parse_path_or_ident();
                self.end_span(&mut span);
                Ok(Expr::new(
                    ExprKind::Variable {
                        ident, // could "evolve into" function call, module identifier or just stay a simple variable.
                    },
                    span,
                ))
            }
            Minus | Excl | Tilde | Ampersand => {
                let operator = UnaryOp::try_from(token.kind).unwrap();
                let bp = operator.prefix_binding_power();
                let start = token.span.start; // taking the start position of the operator.

                let right = self.expr(bp)?; // We call expr recursively to get what this operator applies to
                Ok(Expr::unary(start, operator, right))
            }
            For => return self.parse_for_loop(),
            True => Ok(Expr::new(ExprKind::BoolLiteral(true), token.span)),
            False => Ok(Expr::new(ExprKind::BoolLiteral(false), token.span)),
            StringLiteral => Ok(Expr::new(
                ExprKind::StringLiteral(token.unwrap_value()),
                token.span,
            )),
            OpenParen => {
                let mut span = self.start_span_at_current();
                let inner = self.expr(0)?;
                self.try_consume_token(CloseParen, "Expected ')'")?;
                self.end_span(&mut span);
                Ok(Expr::new(ExprKind::Grouping(inner.boxed()), span))
            }
            OpenBrack => self.parse_array_literal(),
            If => self.parse_if_else_expr(),
            _ => Err(ParserError::UnexpectedToken {
                msg: "Expected an expression".into(),
                peeking: Some(token.clone()),
                span: token.span,
            }),
        }
    }

    fn expr(&mut self, min_bp: u8) -> ExprParseResult {
        // 2 + 2 * 3
        // * (3,4) // higher precedence means the (sub)expression is evaluated before the lower precedence operators. That means it's nested deeper in the AST.
        // + (1,2)

        let mut span = self.start_span_at_next();
        let mut left = self.parse_prefix()?; // handles literals, (groups), and prefix ! - *

        loop {
            let op_token = match self.peek() {
                Some(t) => t.clone(),
                None => break,
            };
            let (l_bp, r_bp) = match op_token.infix_binding_power() {
                Some(bp) => bp,
                None => break,
            };
            if l_bp < min_bp {
                break;
            }

            self.advance();

            if op_token.is_assignment_operator() {
                let right = self.expr(r_bp)?;
                span.merge(right.span);
                left = Expr::new(
                    ExprKind::Assign {
                        assignee: Box::new(left),
                        operator: op_token.kind,
                        value: Box::new(right),
                    },
                    span,
                );
                continue;
            }

            match op_token.kind {
                TokenKind::OpenParen => {
                    let args = self.parse_fn_call_args()?;
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::FnCall {
                            callee: Box::new(left),
                            args,
                        },
                        span,
                    );
                    continue;
                }
                TokenKind::Dot => {
                    // Zig-style postfix deref operator *
                    // Example: <expr>.*
                    if self.matches_advance(TokenKind::Star) {
                        self.end_span(&mut span);
                        left = Expr::new(
                            ExprKind::Unary {
                                operator: UnaryOp::Deref,
                                expr: Box::new(left),
                            },
                            span,
                        );
                        continue;
                    }

                    self.skip_line_terminators();

                    self.try_consume_token(
                        TokenKind::Ident,
                        "Expected identifier or '*' after '.'",
                    )?;
                    let member_ident = self.parse_path_or_ident();
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::FieldAccess {
                            called_on: Box::new(left),
                            member_ident,
                        },
                        span,
                    );
                }
                TokenKind::OpenBrack => {
                    let index_expr = self.expr(0)?;

                    self.try_consume_token(
                        TokenKind::CloseBrack,
                        "Expected ']' after array index",
                    )?;
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::ArrayAccessor {
                            array: Box::new(left),
                            index: Box::new(index_expr),
                        },
                        span,
                    );
                    continue;
                }
                // Explicit Generic Function Call (e.g., foo:[T])
                TokenKind::Colon => {
                    self.try_consume_token(
                        TokenKind::OpenBrack,
                        "Expected '[' after ':' for generic call",
                    )?;

                    let mut type_args = Vec::new();
                    if !self.matches_advance(TokenKind::CloseBrack) {
                        loop {
                            self.skip_line_terminators();
                            type_args.push(self.parse_type_expr()?); // Pure TypeExpr parsing!
                            self.skip_line_terminators();

                            if self.matches_advance(TokenKind::CloseBrack) {
                                break;
                            }
                            if self.matches_advance(TokenKind::Comma) {
                                continue;
                            }

                            return ParserError::unexpected_token(
                                "Expected ',' or ']'",
                                self.peek().cloned(),
                                span,
                            )
                            .wrap();
                        }
                    }
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::GenericFnCallPart {
                            callee: Box::new(left),
                            type_args: Some(type_args),
                        },
                        span,
                    );
                    continue;
                }
                _ => {
                    // Try to turn the token into a BinaryOp
                    let op = BinaryOp::try_from(op_token.kind)
                        .expect("should've been handled in infix_binding_power");
                    // We can unwrap/expect because infix_binding_power returns None and breaks the loop before we get here.
                    self.skip_line_terminators(); // allows to have binary expressions span across multiple lines
                    let right = self.expr(r_bp)?;

                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::Binary {
                            left_expr: Box::new(left),
                            operator: op,
                            right_expr: Box::new(right),
                        },
                        span,
                    );
                }
            }
        }

        Ok(left)
    }

    // <<< PRATT PARSING END

    /// Parses the top level declarations within .mo files that declare items.
    fn parse_top_level_decls(&mut self) -> Result<(), ParserError> {
        loop {
            if let Some(token) = self.peek().cloned() {
                match token.kind {
                    TokenKind::Use => {
                        let use_decl = self.parse_use_decl()?;
                        self.ast.push(use_decl);
                    }
                    TokenKind::Struct => {
                        let struct_decl = self.parse_struct_decl()?;
                        self.ast.push(struct_decl);
                    }
                    TokenKind::Sum => {
                        let sum_decl = self.parse_sum_decl()?;
                        self.ast.push(sum_decl);
                    }
                    TokenKind::Fn => {
                        let fn_decl = self.parse_fn_decl()?;
                        self.ast.push(fn_decl);
                    }
                    TokenKind::Trait => {
                        let trait_decl = self.parse_trait_decl()?;
                        self.ast.push(trait_decl);
                    }
                    TokenKind::LineBreak => {
                        self.advance();
                        continue; // continue for now
                    }
                    TokenKind::EndOfFile => {
                        self.advance();
                        break;
                    }
                    _ => {
                        // Just add error, don't stop the loop...
                        self.errors.push(ParserError::unexpected_token(
                            "Unexpected token parsing top-level declarations",
                            Some(token.clone()),
                            token.span,
                        ));
                        self.synchronize_top_level();
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn synchronize_top_level(&mut self) {
        // Consume the offending token that triggered the error.
        // If you don't do this, you might end up in an infinite loop 
        // re-evaluating the same bad token.
        self.advance(); 
    
        // Skip tokens until we find something that looks like 
        // the start of a valid top-level declaration.
        self.skip_tokens_unless_of_kinds(&[
            TokenKind::Fn,
            TokenKind::Struct,
            TokenKind::Use,
            TokenKind::Sum,
            TokenKind::Trait,
        ]);
    }

    // Parsing:
    // use <module_path_element>(:<module_path_element>)* ("<alias>")?
    //
    // # Examples:
    // use io | use io "foo"
    // use io:print | use io:print "printie"
    fn parse_use_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance(); // consume 'use'
        let mut span = self.start_span_at_current();

        self.try_consume_token(TokenKind::Ident, "Expected identifier")?;
        let path = self.parse_path_or_ident();

        let alias = if self.matches_advance(TokenKind::StringLiteral) {
            Some(self.unwrap_current_token().unwrap_value())
        } else {
            None
        };
        self.end_span(&mut span);
        Ok(Decl::new(DeclKind::Use { path, alias }, span))
    }

    fn parse_fn_signature(&mut self) -> Result<FnSignature, ParserError> {
        self.advance(); // consume 'fn'

        self.try_consume_token(TokenKind::Ident, "Expected function identifier")?;
        let ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;

        self.try_consume_token(TokenKind::OpenParen, "Expected open parenthesis")?;

        // Parse parameters
        let mut params = Vec::new();
        if !self.matches_advance(TokenKind::CloseParen) {
            loop {
                self.try_consume_token(TokenKind::Ident, "Expected variable identifier")?;
                let var_ident = self.unwrap_current_token().unwrap_value();

                let type_expr = self.parse_type_expr()?;
                params.push(TypedVar::new(var_ident, type_expr));

                if self.matches_advance(TokenKind::CloseParen) {
                    break;
                }
                if self.matches_advance(TokenKind::Comma) {
                    continue;
                }
                let token = self.peek().cloned();
                self.errors.push(ParserError::unexpected_token(
                    "Expected argument delimiter ',' or closed parenthesis ')'",
                    token.clone(),
                    token.unwrap().span,
                ));
                self.advance();
            }
        }
        // Parse return type
        let mut return_type = None;
        if !self.matches_any(&[TokenKind::LineBreak, TokenKind::OpenBrace]) {
            let type_expr = self.parse_type_expr()?;
            return_type = Some(type_expr);
        }
        Ok(FnSignature {
            ident,
            generics,
            params,
            return_type,
        })
    }

    // fn abc() ret_type { /*...*/ }
    fn parse_fn_decl(&mut self) -> Result<Decl, ParserError> {
        let mut span = self.start_span_at_next();

        let signature = self.parse_fn_signature()?;
        self.skip_tokens(TokenKind::LineBreak);

        let body = self.parse_code_block()?;

        self.end_span(&mut span);
        Ok(Decl::new(DeclKind::Fn { signature, body }, span))
    }

    fn parse_code_block(&mut self) -> Result<CodeBlock, ParserError> {
        debug!("parsing code block");
        self.try_consume_token(TokenKind::OpenBrace, "Expected open brace")?;

        self.skip_tokens(TokenKind::LineBreak); // move on if there's a linebreak.

        let mut code_block = CodeBlock::new();
        loop {
            if self.matches_advance(TokenKind::CloseBrace) {
                debug!("matched closebrace, returning codeblock");
                return Ok(code_block);
            }
            let stmt = self.parse_stmt()?;
            code_block.stmts.push(stmt);
            self.skip_tokens(TokenKind::LineBreak);
        }
    }

    /// Parses optional generic parameters: '[T, U]' or '[T impl Trait1 Trait2]
    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, ParserError> {
        let mut generics = Vec::new();
        // Check if there is an open paren. If not, return empty vec!
        if self.matches_advance(TokenKind::OpenBrack) {
            if !self.matches(TokenKind::CloseBrack) {
                loop {
                    self.try_consume_token(TokenKind::Ident, "Expected generic type identifier")?;
                    let generic_ident = self.unwrap_current_token().unwrap_value();

                    let mut bounds = Vec::new();

                    // If we see an impl, parse the trait bounds!
                    if self.matches_advance(TokenKind::Impl) {
                        loop {
                            self.try_consume_token(TokenKind::Ident, "Expected trait")?;
                            bounds.push(self.parse_path_or_ident());

                            if !self.matches(TokenKind::Ident) {
                                break;
                            }
                        }
                    }

                    // Add the generic_ident and its bounds to your list
                    generics.push(GenericParam {
                        ident: generic_ident,
                        bounds: Some(bounds),
                    });

                    if self.matches_advance(TokenKind::CloseBrack) {
                        break;
                    }
                    // Allow trailing commas!
                    if self.matches_advance(TokenKind::Comma) {
                        self.skip_line_terminators();
                        if self.matches_advance(TokenKind::CloseBrack) {
                            break;
                        }
                        continue;
                    }
                    let token = self.peek();
                    return Err(ParserError::unexpected_token(
                        "Expected ',' or ')'",
                        token.cloned(),
                        token.unwrap().span,
                    ));
                }
            }
        }
        Ok(generics)
    }

    /// Parses optional trait implementations: 'impl Trait1, Trait2' or 'impl Trait1[i32], Trait2[string]
    fn parse_impl_traits(&mut self) -> Result<Vec<TraitBound>, ParserError> {
        let mut traits = Vec::new();
        if self.matches_advance(TokenKind::Impl) {
            loop {
                self.try_consume_token(TokenKind::Ident, "Expected trait")?;
                let ident = self.parse_path_or_ident();

                let mut args = Vec::new();
                if self.matches_advance(TokenKind::OpenBrack) {
                    if !self.matches_advance(TokenKind::CloseBrack) {
                        loop {
                            self.skip_line_terminators();

                            // Trait arguments are full type expressions
                            args.push(self.parse_type_expr()?);

                            self.skip_line_terminators();

                            if self.matches_advance(TokenKind::CloseBrack) {
                                break;
                            }
                            if self.matches_advance(TokenKind::Comma) {
                                continue;
                            }
                            let token = self.peek();
                            return Err(ParserError::unexpected_token(
                                "Expected ',' or ']'",
                                token.cloned(),
                                token.unwrap().span,
                            ));
                        }
                    }
                }

                traits.push(TraitBound { ident, args });
                if !self.matches_advance(TokenKind::Comma) {
                    break;
                }
                self.skip_line_terminators();
            }
        }
        Ok(traits)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        let mut span = self.start_span_at_next(); // this span is only used when no statement could be parsed
        if let Some(token) = self.peek().cloned() {
            match token.kind {
                TokenKind::Ret => return self.parse_ret_stmt(),
                TokenKind::Defer => return self.parse_defer_stmt(),
                TokenKind::Break => return self.parse_break_stmt(),
                TokenKind::Next => {
                    self.advance();
                    return Ok(Stmt::new(StmtKind::Next, token.span));
                }
                TokenKind::OpenBrace => {
                    let mut span = self.start_span_at_next();
                    let code_block = self.parse_code_block()?;
                    self.end_span(&mut span);
                    return Ok(Stmt::new(StmtKind::CodeBlock(code_block), span));
                }
                TokenKind::Ident => {
                    // Parse variable declarations
                    if let Some(var_decl) = self.parse_var_decl()? {
                        // Make sure to consume the possible line break / semicolon here
                        self.skip_line_terminators();
                        return Ok(var_decl);
                    }
                }
                _ => {}
            }

            // If it wasn't a dedicated statement or a variable declaration,
            // it MUST be an expression statement (e.g. `foo()` or `a = 10` or `a.b += 5`)

            let mut span = self.start_span_at_next();
            let expr = self.parse_expression()?;

            // If the next token isn't a valid terminator or the end of a block, throw an error!
            if !self.matches_any(&[TokenKind::LineBreak, TokenKind::Semicolon])
                && !self.matches(TokenKind::CloseBrace)
            {
                let token = self.peek();
                return Err(ParserError::unexpected_token(
                    "Expected newline or semicolon after expression statement",
                    token.cloned(),
                    token.unwrap().span,
                ));
            }

            self.end_span(&mut span);
            self.skip_line_terminators();
            return Ok(Stmt::new(StmtKind::Expr(expr), span));
        }

        self.end_span(&mut span);
        Err(ParserError::unexpected_token(
            "Couldn't parse statement",
            self.peek().cloned(),
            span,
        ))
    }

    // Parses statements of this form:
    // a i32 := 10    | DeclAssignmt
    // a := 10        | DeclAssignmt (no type identifier. type to be inferred)
    // a [3]i32 := 10 | DeclAssignmt with complex type
    fn parse_var_decl(&mut self) -> Result<Option<Stmt>, ParserError> {
        let mut span = self.start_span_at_current();

        // 1. Scan ahead to find out if this statement contains a ':='
        let mut i = 0;
        let mut is_decl = false;

        loop {
            match self.peek_nth(i) {
                Some(token) if token.kind == TokenKind::DeclareAssign => {
                    is_decl = true;
                    break;
                }
                Some(token)
                    if token.is_of_any_kinds(&[
                        TokenKind::LineBreak,
                        TokenKind::Semicolon,
                        TokenKind::Equals,
                        TokenKind::OpenBrace,
                    ]) =>
                {
                    // We hit a statement boundary or an assignment, so it's not a var decl
                    break;
                }
                None => break, // EOF
                _ => i += 1,
            }
        }

        // 2. If no ':=' was found, bail out so parse_stmt can parse it as an expression.
        if !is_decl {
            return Ok(None);
        }

        // 3. We are guaranteed this is a variable declaration now!

        // consume the variable identifier
        let ident = self.advance().unwrap().unwrap_value();

        // 4. Parse the optional type expression
        let mut type_expr = None;
        if !self.matches(TokenKind::DeclareAssign) {
            // If the next token isn't ':=', there must be a type expression here
            type_expr = Some(self.parse_type_expr()?);
        }

        // 5. Consume ':=' and parse the value
        self.try_consume_token(TokenKind::DeclareAssign, "Expected ':='")?;
        let value = self.parse_expression()?;

        self.end_span(&mut span);

        Ok(Some(Stmt::new(
            StmtKind::LocalVarDeclAssign {
                ident,
                type_expr,
                value,
            },
            span,
        )))
    }

    // like '[1,2,3,4,5,6,7,8]'
    fn parse_array_literal(&mut self) -> Result<Expr, ParserError> {
        let mut elements = Vec::new();
        let mut span = self.start_span_at_current();
        if !self.matches_advance(TokenKind::CloseBrack) {
            loop {
                let element = self.parse_expression()?;
                elements.push(element);

                self.skip_line_terminators();

                if self.matches_advance(TokenKind::CloseBrack) {
                    break;
                }
                if self.matches_advance(TokenKind::Comma) {
                    self.skip_line_terminators();

                    if self.matches_advance(TokenKind::CloseBrack) {
                        break;
                    }
                    continue;
                }

                let token = self.peek();
                return ParserError::unexpected_token(
                    "Expected comma ',' or closed bracket ']'",
                    token.cloned(),
                    token.unwrap().span,
                )
                .wrap();
            }
        }
        self.end_span(&mut span);
        return Ok(Expr::new(ExprKind::ArrayLiteral { elements }, span));
    }

    #[track_caller]
    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParserError> {
        debug!("parsing type expr from: {}", std::panic::Location::caller());
        let mut span = self.start_span_at_next();
        // array type
        let mut array_length = None;
        if self.matches_advance(TokenKind::OpenBrack) {
            if self.matches_any_advance(&[TokenKind::DecimalIntegerNumberLiteral]) {
                let length: usize = self
                    .unwrap_current_token()
                    .unwrap_value()
                    .parse()
                    .expect("If this panics, there is a lexer bug");
                array_length = Some(length);
            }

            self.try_consume_token(TokenKind::CloseBrack, "Expected closed bracket ']'")?;
            return Ok(TypeExpr::Array {
                length: array_length,
                type_expr: Box::new(self.parse_type_expr()?),
            });
        }

        // pointer type
        if self.matches_advance(TokenKind::Star) {
            return Ok(TypeExpr::pointer(self.parse_type_expr()?));
        }

        // identifier with optional module path prefix
        if self.matches_any_advance(&[TokenKind::Ident]) {
            let base_ident = self.parse_path_or_ident();

            // generic type
            if self.matches_advance(TokenKind::OpenBrack) {
                let mut generic_params = Vec::new();
                if !self.matches(TokenKind::CloseBrack) {
                    loop {
                        self.skip_line_terminators();

                        generic_params.push(self.parse_type_expr()?);
                        self.skip_line_terminators();

                        if self.matches_advance(TokenKind::CloseBrack) {
                            break;
                        }
                        if self.matches_advance(TokenKind::Comma) {
                            continue;
                        }
                        let token = self.peek();
                        return ParserError::unexpected_token(
                            "Expected ',' or ']' after generic type argument",
                            token.cloned(),
                            token.unwrap().span,
                        )
                        .wrap();
                    }
                }
                return Ok(TypeExpr::Generic {
                    ident: base_ident,
                    params: generic_params,
                });
            }

            // not generic, just simple ident.
            return Ok(TypeExpr::Ident(base_ident));
        }
        self.end_span(&mut span);
        ParserError::unexpected_token("Expected type expression", self.peek().cloned(), span).wrap()
    }

    #[allow(dead_code)]
    fn consume_line_terminator(&mut self) -> Result<(), ParserError> {
        self.try_consume_token2(
            &[TokenKind::LineBreak, TokenKind::Semicolon],
            "Expected line break or ;",
        )
    }

    fn parse_fn_call_args(&mut self) -> Result<Vec<Expr>, ParserError> {
        // parse arguments
        let mut args = Vec::new();

        if !self.matches_advance(TokenKind::CloseParen) {
            loop {
                let expr = self.parse_expression()?;
                args.push(expr);

                self.skip_line_terminators();

                if self.matches_advance(TokenKind::CloseParen) {
                    break;
                }
                if self.matches_advance(TokenKind::Comma) {
                    self.skip_line_terminators(); // added this

                    // trailing commas
                    // If we skipped the comma/newlines and hit a ')', we are done.
                    if self.matches_advance(TokenKind::CloseParen) {
                        break;
                    }
                    continue;
                }
                let token = self.peek();
                return ParserError::unexpected_token(
                    "Expected argument delimiter ',' or closed parenthesis ')'",
                    token.cloned(),
                    token.unwrap().span,
                )
                .wrap();
            }
        }
        Ok(args)
    }

    // TODO:
    // for-in loop for collections, for loop without condition (like loop keyword in Rust)
    // TODO: Make it be an expression for break with value.
    //
    // Parsing:
    // for <bool expr>? <code block>
    fn parse_for_loop(&mut self) -> Result<Expr, ParserError> {
        let mut span = self.start_span_at_current();
        if let Some(token) = self.peek()
            && token.is_of_kind(TokenKind::OpenBrace)
        {
            let code_block = self.parse_code_block()?;
            self.end_span(&mut span);
            return Ok(Expr::new(
                ExprKind::ForLoop {
                    condition: None,
                    code_block,
                },
                span,
            ));
        }
        let condition = Some(self.parse_expression()?.boxed());
        let code_block = self.parse_code_block()?;
        self.end_span(&mut span);
        let stmt = Expr::new(
            ExprKind::ForLoop {
                condition,
                code_block,
            },
            span,
        );
        Ok(stmt)
    }

    // TODO:
    // if is expr (like switch or match)
    fn parse_if_else_expr(&mut self) -> Result<Expr, ParserError> {
        let mut span = self.start_span_at_current();
        let condition = self.parse_expression()?.boxed();
        let if_block = self.parse_code_block()?;
        let else_block = if self.matches_advance(TokenKind::Else) {
            Some(self.parse_code_block()?)
        } else {
            None
        };

        self.end_span(&mut span);
        Ok(Expr::new(
            ExprKind::If {
                condition,
                if_block,
                else_block,
            },
            span,
        ))
    }

    fn parse_defer_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        let stmt = self.parse_stmt()?;
        self.end_span(&mut span);
        Ok(Stmt::new(StmtKind::Defer(Box::new(stmt)), span))
    }

    fn parse_break_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        if self.matches_any(&[TokenKind::LineBreak, TokenKind::Semicolon]) {
            self.end_span(&mut span);
            return Ok(Stmt::new(StmtKind::Break { value: None }, span));
        }
        let expr = self.parse_expression()?;
        self.end_span(&mut span);
        Ok(Stmt::new(StmtKind::Break { value: Some(expr) }, span))
    }

    fn parse_ret_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        if self.matches_advance(TokenKind::LineBreak) {
            self.end_span(&mut span);
            Ok(Stmt::new(StmtKind::Ret(None), span))
        } else {
            let expr = self.parse_expression()?;
            Ok(Stmt::new(StmtKind::Ret(Some(expr)), span))
        }
    }

    /*
    Parse struct declaration of form:
    struct Foo {
        a int
        b int
    }
    unit struct:
    struct Foo {}
     */
    fn parse_struct_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        self.try_consume_token(TokenKind::Ident, "Expected struct identifier")?;
        let struct_ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;
        let impl_traits = self.parse_impl_traits()?;

        self.skip_tokens(TokenKind::LineBreak);
        self.try_consume_token(TokenKind::OpenBrace, "Expected open brace or impl")?; // this error message is weird
        let mut fields = Vec::new();
        loop {
            self.skip_tokens(TokenKind::LineBreak);
            if self.matches_advance(TokenKind::CloseBrace) {
                break;
            }
            self.try_consume_token(TokenKind::Ident, "Expected variable identifier")?;
            let var_ident = self.unwrap_current_token().unwrap_value();

            let type_expr = self.parse_type_expr()?;
            fields.push(TypedVar::new(var_ident, type_expr));
            if !self
                .peek()
                .is_some_and(|t| t.is_of_kind(TokenKind::CloseBrace))
            {
                // If next isn't token CloseBrace, means we are expecting next struct field declaration
                self.try_consume_token2(
                    &[TokenKind::LineBreak, TokenKind::Comma],
                    "Expected comma ',' or linebreak",
                )?;
            }
        }
        self.end_span(&mut span);
        Ok(Decl::new(
            DeclKind::Struct {
                ident: struct_ident,
                fields,
                generics,
                impl_traits,
            },
            span,
        ))
    }

    fn parse_sum_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance(); // consume 'sum' (assuming TokenKind::Sum)
        let mut span = self.start_span_at_current();

        self.try_consume_token(TokenKind::Ident, "Expected sum type identifier")?;
        let sum_ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;
        let impl_traits = self.parse_impl_traits()?;

        self.skip_line_terminators();
        self.try_consume_token(TokenKind::OpenBrace, "Expected '{'")?;

        debug!("matched closebrace, returning codeblock");
        let mut variants = Vec::new();

        loop {
            self.skip_line_terminators();
            if self.matches_advance(TokenKind::CloseBrace) {
                break;
            }

            self.try_consume_token(TokenKind::Ident, "Expected variant identifier")?;
            let variant_ident = self.unwrap_current_token().unwrap_value();

            // Figure out which kind of variant this is
            let data = if self.matches_advance(TokenKind::OpenParen) {
                // 1. Tuple Variant
                let mut types = Vec::new();
                if !self.matches_advance(TokenKind::CloseParen) {
                    loop {
                        self.skip_line_terminators();
                        types.push(self.parse_type_expr()?);
                        self.skip_line_terminators();

                        if self.matches_advance(TokenKind::CloseParen) {
                            break;
                        }
                        self.try_consume_token(TokenKind::Comma, "Expected ',' or ')'")?;
                    }
                }
                VariantData::Tuple(types)
            } else if self.matches_advance(TokenKind::OpenBrace) {
                // 2. Struct Variant
                let mut fields = Vec::new();
                loop {
                    self.skip_line_terminators();
                    if self.matches_advance(TokenKind::CloseBrace) {
                        break;
                    }

                    self.try_consume_token(TokenKind::Ident, "Expected field identifier")?;
                    let field_ident = self.unwrap_current_token().unwrap_value();
                    let type_expr = self.parse_type_expr()?;

                    fields.push(TypedVar::new(field_ident, type_expr));

                    // Fields can be separated by commas or linebreaks
                    self.matches_any_advance(&[TokenKind::Comma, TokenKind::LineBreak]);
                }
                VariantData::Struct(fields)
            } else {
                // 3. Unit Variant
                VariantData::Unit
            };

            variants.push(Variant {
                ident: variant_ident,
                data,
            });

            // Variants themselves must be separated by commas or linebreaks
            if !self
                .peek()
                .is_some_and(|t| t.is_of_kind(TokenKind::CloseBrace))
            {
                self.try_consume_token2(
                    &[TokenKind::LineBreak, TokenKind::Comma],
                    "Expected ',', or linebreak between variants",
                )?;
            }
        }

        self.end_span(&mut span);

        Ok(Decl::new(
            DeclKind::Sum {
                ident: sum_ident,
                generics,
                impl_traits,
                variants,
            },
            span,
        ))
    }

    fn parse_trait_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance(); // Consume 'trait' keyword
        let mut span = self.start_span_at_current();

        self.try_consume_token(TokenKind::Ident, "Expected trait identifier")?;
        let ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;

        self.skip_line_terminators();
        self.try_consume_token(TokenKind::OpenBrace, "Expected '{'")?;

        let mut methods = Vec::new();

        loop {
            self.skip_line_terminators();
            if self.matches_advance(TokenKind::CloseBrace) {
                break;
            }

            methods.push(self.parse_fn_signature()?);

            // Allow (but don't strictly require) linebreaks/semicolons between methods
            self.skip_line_terminators();
        }

        self.end_span(&mut span);

        Ok(Decl::new(
            DeclKind::Trait {
                ident,
                generics,
                methods,
            },
            span,
        ))
    }

    // :Expressions

    fn parse_expression(&mut self) -> ExprParseResult {
        debug!("parsing expression {}", self.parser_state_dbg_info());
        self.expr(0)
    }

    // :Utils

    /// Parses a path or simple ident
    /// Assumes the first identifier has just been consumed.
    fn parse_path_or_ident(&mut self) -> Ident {
        let first_ident = self.unwrap_current_token().unwrap_value();
        let mut path_parts = VecDeque::new();
        path_parts.push_back(first_ident);

        while self.is_next_of_kind(TokenKind::Colon) {
            if let Some(next_token) = self.peek_nth(1) {
                if next_token.kind == TokenKind::Ident {
                    self.advance(); // consume ':'
                    self.advance(); // consume Ident
                    path_parts.push_back(self.unwrap_current_token().unwrap_value());
                    continue;
                }
            }
            break; // Break if it's not followed by an Ident (e.g. `:[` generic call)
        }

        if path_parts.len() > 1 {
            let ident_name = path_parts.pop_back().unwrap();
            let module_path = ModulePath { path: path_parts };
            Ident::WithModulePrefix(module_path, ident_name)
        } else {
            Ident::Simple(path_parts.pop_front().unwrap())
        }
    }

    /// Gets and clones current token.
    /// # Panics
    /// If current token is None
    fn unwrap_current_token(&self) -> Token {
        self.current_token.clone().unwrap()
    }

    fn current_token(&self) -> Option<Token> {
        self.current_token.clone()
    }

    #[track_caller]
    fn advance(&mut self) -> Option<Token> {
        self.current_token = self.token_stream.next();

        // Debug info:
        if let Some(current_token) = self.current_token() {
            if let Some(peeked_token) = self.token_stream.peek().cloned() {
                debug!(
                    "{} advanced from {}, peeking {}",
                    std::panic::Location::caller(),
                    current_token,
                    peeked_token
                );
            }
        }
        return self.current_token();
    }

    /// Advances n times.
    #[allow(dead_code)]
    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    /// Skips tokens that are of the same kind as given.
    fn skip_tokens(&mut self, token_kind: TokenKind) {
        loop {
            if let Some(token) = self.peek() {
                if token.is_of_kind(token_kind) {
                    let token = token.clone();
                    self.advance();
                    debug!("Skipped token: {}", token);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Skips tokens that are of any of the kinds given
    #[allow(dead_code)]
    fn skip_tokens_of_kinds(&mut self, token_kinds: &[TokenKind]) {
        loop {
            if let Some(token) = self.peek() {
                if token.is_of_any_kinds(token_kinds) {
                    debug!("Peeking token: {}, skipping", token);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Skips all tokens that are not of the given kinds until it hits a token of the kinds given
    fn skip_tokens_unless_of_kinds(&mut self, token_kinds: &[TokenKind]) {
        self.skip_tokens_if_predicate(|parser| !parser.is_next_of_kinds(token_kinds));
    }

    /// Skips all tokens if the predicate evaluates to true until it evaluates to false
    fn skip_tokens_if_predicate<F>(&mut self, predicate: F)
    where
        F: Fn(&mut Self) -> bool,
    {
        loop {
            if let Some(token) = self.peek().cloned() {
                if predicate(self) {
                    let token = token.clone();
                    self.advance();
                    debug!("Skipped token: {}", token);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn skip_line_terminators(&mut self) {
        self.skip_tokens_of_kinds(&[TokenKind::LineBreak, TokenKind::Semicolon]);
    }

    fn peek(&mut self) -> Option<&Token> {
        self.token_stream.peek()
    }

    #[allow(dead_code)]
    fn peek_nth(&mut self, n: usize) -> Option<&Token> {
        self.token_stream.peek_nth(n) // Note: maybe check if this causes problems at the end of files...
    }

    /// checks if next token is of one of the given types, then moves on to that token
    fn matches_any_advance(&mut self, tokens: &[TokenKind]) -> bool {
        if self.is_next_of_kinds(tokens) {
            self.advance();
            return true;
        }
        false
    }

    #[track_caller]
    fn matches_advance(&mut self, token: TokenKind) -> bool {
        debug!("{}", std::panic::Location::caller());
        if self.is_next_of_kind(token) {
            self.advance();
            return true;
        }
        false
    }

    /// checks if next token is of one of the given types
    fn matches_any(&mut self, tokens: &[TokenKind]) -> bool {
        if self.is_next_of_kinds(tokens) {
            return true;
        }
        false
    }

    #[allow(dead_code)]
    #[track_caller]
    fn matches(&mut self, token: TokenKind) -> bool {
        debug!("{}", std::panic::Location::caller());
        if self.is_next_of_kind(token) {
            return true;
        }
        false
    }

    /// peeks multiple tokens to see if they match the given token types in row.
    #[allow(dead_code)]
    fn matches_in_row(&mut self, tokens: &[TokenKind]) -> bool {
        for (n, token_type) in tokens.iter().enumerate() {
            if let Some(peeked_token) = self.peek_nth(n) {
                if &peeked_token.kind == token_type {
                    continue;
                } else {
                    return false;
                }
            }
        }
        true
    }

    #[allow(dead_code)]
    fn matches_predicate<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(&mut Parser) -> bool,
    {
        if predicate(self) {
            self.advance();
            return true;
        }
        false
    }

    // checks if the following token is of the given type.
    fn is_next_of_kind(&mut self, token_type: TokenKind) -> bool {
        if let Some(current_token) = self.peek() {
            if token_type == current_token.kind && token_type != TokenKind::EndOfFile {
                // TODO: Check if EndOfFile check is necessary
                return true;
            }
        }
        false
    }

    // checks if the following token is of one of the given types.
    fn is_next_of_kinds(&mut self, tokens: &[TokenKind]) -> bool {
        for token in tokens {
            if self.is_next_of_kind(*token) {
                return true;
            }
        }
        false
    }

    // if next token is of given type, advances. If not, return an error with given message.
    fn try_consume_token(&mut self, token_type: TokenKind, msg: &str) -> Result<(), ParserError> {
        if self.is_next_of_kind(token_type) {
            self.advance();
            return Ok(());
        }
        let token = self.peek();
        Err(ParserError::unexpected_token(
            msg,
            token.cloned(),
            token.unwrap().span,
        ))
    }

    /// If next token is of any of given types, advances. If not, return an error with given message.
    fn try_consume_token2(&mut self, tokens: &[TokenKind], msg: &str) -> Result<(), ParserError> {
        if self.is_next_of_kinds(tokens) {
            self.advance();
            return Ok(());
        }
        let token = self.peek();
        Err(ParserError::unexpected_token(
            msg,
            token.cloned(),
            token.unwrap().span,
        ))
    }

    /// Prints some debug info about the current state of the parser
    fn parser_state_dbg_info(&mut self) -> String {
        let current = self.unwrap_current_token();
        let line = current.span.start.line;
        let col = current.span.end.column;
        let r#type = current.kind;
        let mut str = String::new();

        str.push_str(&format!("Current: \"{}\", loc: {}:{}", r#type, line, col));

        if let Some(next) = self.peek() {
            let line = next.span.end.line;
            let col = next.span.end.column;
            let r#type = next.kind;
            str.push_str(&format!(" --- Next: \"{}\", loc: {}:{}", r#type, line, col));
        }
        str
    }
}

