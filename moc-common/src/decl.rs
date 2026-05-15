use serde::Serialize;

use crate::{
    CodeBlock, CodeSpan, TypedVar, expr::{Expr, GenericParam, Ident, ImplDeclPart, TypeExpr}
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
    },
    Sum {
        ident: String,
        generics: Vec<GenericParam>,
        variants: Vec<Variant>,
    },
    Trait {
        ident: String,
        generics: Vec<GenericParam>, // Traits can be generic too!
        methods: Vec<FnSignature>,
    },
    Impl {
        parts: Vec<ImplDeclPart>
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
