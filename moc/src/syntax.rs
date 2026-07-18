//! I wanted to use the code in here to implement red-green trees with cstree,
//! but I thought: NAH we not doing this. at least not for now. This will stay for now though, maybe it's useful some day bruh

use std::panic;

use cstree::prelude::*;
use num_derive::FromPrimitive;

pub type Mo = SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum SyntaxKind {
    /* Tokens - Operators/Symbols */
    AddAssign,           // +=
    Ampersand,           // &
    Equals,              // =
    At,                  // @
    BitAndAssign,        // &=
    BitOrAssign,         // |=
    Tilde,               // ~
    BitXorAssign,        // ^=
    BitNotAssign,        // ~=
    BitShiftLeft,        // <<
    BitShiftRight,       // >>
    BitShiftLeftAssign,  // <<=
    BitShiftRightAssign, // >>=
    Caret,               // ^
    OpenBrace,           // {
    CloseBrace,          // }
    OpenParen,           // (
    CloseParen,          // )
    OpenBrack,           // [
    CloseBrack,          // ]
    Colon,               // :
    Semicolon,           // ;
    Comma,               // ,
    DeclareAssign,       // :=
    DivAssign,           // /=
    Dot,                 // .
    DoubleEquals,        // ==
    Excl,                // !
    Greater,             // >
    GreaterOrEqual,      // >=
    Ident,
    Less,        // <
    LessOrEqual, // <=
    LineBreak,   // encompassing CRLF and LF in one token.
    Plus,        // +
    Minus,       // -
    Star,        // *
    Percent,     // %
    Pipe,        // |
    ModAssign,   // %=
    MultAssign,  // *=
    ExclEquals,  // !=
    Slash,       // /
    SubAssign,   // -=
    /* Tokens - Number literals */
    DecimalIntegerNumberLiteral,
    DecimalPointNumberLiteral,
    HexadecimalIntegerNumberLiteral,
    OctalIntegerNumberLiteral,
    BinaryIntegerNumberLiteral,
    ScientificDecimalNumberLiteral, // 1e9 = 1,000,000,000 | 1e-9 = 0.000000001 | 0.1e9 = 100,000,000 | 0.1e-9 = 0.0000000001
    ScientificHexNumberLiteral,     // 0x10p10 =  0x10 * 2^10 | 0x10p-10 = 0x10 * 2^-10

    StringLiteral,
    /* Tokens - Keywords */
    Defer,
    True,
    False,
    Fn,
    For,
    If,
    In,
    Impl,
    Break,
    Else,
    Loop,
    Is,
    Next, // keyword, like 'continue' in other languages
    Ret,
    Trait,
    Struct,
    Sum,
    Use,

    /* Nodes */
    Expr,
    Root,
    EndOfFile,
}

impl Syntax for Mo {
    fn static_text(self) -> Option<&'static str> {
        use SyntaxKind::*;
        match self {
            /* Tokens */
            AddAssign => Some("+="),
            Ampersand => Some("&"),
            Equals => Some("="),
            At => Some("@"),
            BitAndAssign => Some("&="),
            BitOrAssign => Some("|="),
            Tilde => Some("~"),
            BitXorAssign => Some("^="),
            BitNotAssign => Some("~="),
            BitShiftLeft => Some("<<"),
            BitShiftRight => Some(">>"),
            BitShiftLeftAssign => Some("<<="),
            BitShiftRightAssign => Some(">>="),
            Caret => Some("^"),
            OpenBrace => Some("{"),
            CloseBrace => Some("}"),
            OpenParen => Some("("),
            CloseParen => Some(")"),
            OpenBrack => Some("["),
            CloseBrack => Some("]"),
            Colon => Some(":"),
            Semicolon => Some(";"),
            Comma => Some(","),
            DeclareAssign => Some(":="),
            DivAssign => Some("/="),
            Dot => Some("."),
            DoubleEquals => Some("=="),
            Excl => Some("!"),
            Greater => Some(">"),
            GreaterOrEqual => Some(">="),
            Less => Some("<"),
            LessOrEqual => Some("<="),
            LineBreak => Some("\n"),
            Plus => Some("+"),
            Minus => Some("-"),
            Star => Some("*"),
            Percent => Some("%"),
            Pipe => Some("|"),
            ModAssign => Some("%="),
            MultAssign => Some("*="),
            ExclEquals => Some("!="),
            Slash => Some("/"),
            SubAssign => Some("-="),

            /* Keywords */
            Defer => Some("defer"),
            True => Some("true"),
            False => Some("false"),
            Fn => Some("fn"),
            For => Some("for"),
            If => Some("if"),
            In => Some("in"),
            Impl => Some("impl"),
            Break => Some("break"),
            Else => Some("else"),
            Loop => Some("loop"),
            Is => Some("is"),
            Next => Some("next"), // keyword, like 'continue' in other languages
            Ret => Some("ret"),
            Trait => Some("trait"),
            Struct => Some("struct"),
            Sum => Some("sum"),
            Use => Some("use"),
            _ => None,
        }
    }

    fn from_raw(raw: RawSyntaxKind) -> Self {
        match num::FromPrimitive::from_u32(raw.0) {
            Some(syntax) => syntax,
            None => panic!("invalid raw syntax kind: {}", raw.0),
        }
    }

    fn into_raw(self) -> RawSyntaxKind {
        RawSyntaxKind(self as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw() {
        let usize = 10;
        let syntax = Mo::from_raw(RawSyntaxKind(usize));
        assert_eq!(syntax, Mo::BitShiftRight);
    }
}
