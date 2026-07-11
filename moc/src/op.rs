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
