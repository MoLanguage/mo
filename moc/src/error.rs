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
