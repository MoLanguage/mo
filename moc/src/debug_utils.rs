use crate::token::{Token};

pub fn print_tokens(tokens: &[Token]) {
    for token in tokens {
        println!("{}", token);
    }
}

pub const INDENT: &str = "  ";
pub fn create_indent(depth: usize) -> String {
    format!("\n{}", INDENT.repeat(depth))
}
