use crate::{CodeSpan, ast::Ast, expr::Expr, token::Token};
use std::io;
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: CodeSpan,
}

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("File operation failed")]
    FileNotFound(#[from] io::Error),
}

impl From<LexerError> for Diagnostic {
    fn from(value: LexerError) -> Self {
        value.into_diagnostic()
    }
}

impl From<ParserError> for Diagnostic {
    fn from(value: ParserError) -> Self {
        value.into_diagnostic()
    }
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

impl LexerError {
    fn span(&self) -> CodeSpan {
        *match self {
            LexerError::UnterminatedStringLiteral(code_span) => code_span,
            LexerError::InvalidCharacter(_, code_span) => code_span,
            LexerError::UnknownEscapeCharacter(code_span) => code_span,
            LexerError::MultiDecimalPointInNumberLiteral(code_span) => code_span,
            LexerError::UnexpectedCharacterLexingNonDecimalNumberLiteral(code_span) => code_span,
            LexerError::UnknownToken(code_span) => code_span,
        }
    }

    pub fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            message: self.to_string(),
            span: self.span(),
        }
    }
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
        Self::UnexpectedToken {
            msg: msg.into(),
            peeking,
            span,
        }
    }
    pub fn span(&self) -> CodeSpan {
        *match self {
            ParserError::UnexpectedToken {
                msg: _,
                peeking: _,
                span,
            } => span,
        }
    }

    pub fn wrap<T>(self) -> Result<T, ParserError> {
        Err(self)
    }
    pub fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            message: self.to_string(),
            span: self.span(),
        }
    }
}

pub type LexerResult = Result<Token, LexerError>;
pub type ParseResult = Result<Ast, ParserError>;
pub type ExprParseResult = Result<Expr, ParserError>;
