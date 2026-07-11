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
