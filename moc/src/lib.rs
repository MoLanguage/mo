pub mod ast;
pub mod debug_utils;
pub mod decl;
pub mod error;
pub mod expr;
pub mod op;
pub mod stmt;
pub mod token;
pub mod semantic;
pub mod parser;
pub mod lexer;
pub mod syntax;

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
};

use derive_more::Display;
use serde::Serialize;

use crate::{
    error::Diagnostic, expr::TypeExpr, stmt::Stmt,
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

use std::{fs::File, io::Read, path::Path};

use crate::{ast::Ast, token::Token};
use crate::{lexer::Lexer, parser::Parser};

pub struct CompileResultData {
    pub tokens: Option<Vec<Token>>,
    pub ast: Option<Ast>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileResultData {
    pub fn new() -> Self {
        Self {
            tokens: None,
            ast: None,
            diagnostics: Vec::new(),
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
            for lexer_error in lexer.diagnostics {
                meta_data
                    .diagnostics
                    .push(lexer_error);
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
            for parser_error in &parser.diagnostics {
                meta_data
                    .diagnostics
                    .push(parser_error.clone());
            }

            if parser.diagnostics.len() == 0 {
                // TODO: continue into semantic analysis
            }
        }
        Err(io_error) => {
        	eprintln!("{:?}", io_error)
        }
    }
    meta_data
}
