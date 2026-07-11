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
