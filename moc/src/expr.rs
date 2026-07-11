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
        args: Vec<TypeExpr>, // identifiers of generic type parameters
    },
}

// used in impl declaration items.
// Trait[type1] or Trait[type1, type2]
#[derive(Debug, Clone, Serialize)]
pub struct ImplDeclPart {
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
