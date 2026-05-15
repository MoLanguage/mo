use std::collections::HashMap;

use moc_common::{ast::Ast};

// builtin types:
// Int8, Int16, Int32, Int64
//

pub enum Type {
    Builtin(BuiltinType),
    Custom(CustomType),
    Array(Box<Type>, usize),
    Pointer(Box<Type>),
}

pub struct CustomType {
    pub name: String,
    pub byte_width: usize,
}

pub enum BuiltinType {
    SignedInt8,
    SignedInt16,
    SignedInt32,
    SignedInt64,
    SignedInt128,
    UnsignedInt8,
    UnsignedInt16,
    UnsignedInt32,
    UnsignedInt64,
    UnsignedInt128,
    Bool, // in the future maybe bools of other widths
    Unknown,
    Empty, // for expression statements with no return type
}

pub struct Function {
    pub name: String,
    pub return_type: Type,
}

pub enum Symbol {
    Function {
        name: String,
        params: Vec<(String, Type)>,
    },
    Struct {
        name: String,
        params: Vec<(String, Type)>,
    },
}

pub struct SymbolTable {
    pub functions: HashMap<String, Symbol>,
    pub structs: HashMap<String, Symbol>,
    pub uses: HashMap<String, Symbol>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            uses: HashMap::new(),
        }
    }
}

pub struct ScopeStack {
    scopes: Vec<SymbolTable>,
}

pub struct SemanticAnalyzer {
    ast: Ast,
}

impl SemanticAnalyzer {
    fn new(ast: Ast) -> Self {
        Self { ast }
    }

    fn analyze(&mut self) -> SymbolTable {
        let decls = SymbolTable::new();
        for decl in &self.ast {
            
        }
        decls
    }
}
