use std::str::Chars;

use itertools::{PeekNth, peek_nth};
use log::debug;
use crate::error::{LexerError, LexerResult};
use crate::token::{NumberLiteralKind, Token, TokenKind, TokenKind::*};
use crate::{CodeLocation, CodeSpan};

#[derive(Clone)]
pub struct Lexer<'a> {
    chars: PeekNth<Chars<'a>>,
    last_token_end: CodeLocation,
    location: CodeLocation,
    pub errors: Vec<LexerError>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: peek_nth(input.chars()),
            location: CodeLocation::default(),
            last_token_end: CodeLocation::default(),
            errors: Vec::new(),
        }
    }

    pub fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::with_capacity(256);
        loop {
            match self.next_token() {
                Ok(token) => if token.kind == EndOfFile {
                    break;
                } else {
                    tokens.push(token);
                },
                Err(error) => self.errors.push(error),
            }
        }
        tokens
    }

    fn current_span(&self) -> CodeSpan {
        CodeSpan::from((self.last_token_end, self.location))
    }

    fn new_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.current_span())
    }

    fn count_line_break(&mut self) {
        self.location.column = 1;
        self.location.line += 1;
    }

    fn update_span(&mut self) {
        self.last_token_end = self.location;
    }

    fn advance(&mut self) -> Option<char> {
        self.location.column += 1;
        let char = self.chars.next();
        debug!("lexing char: {}", char.unwrap_or('E').escape_default());
        char
    }

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().cloned()
    }
    fn peek_nth_char(&mut self, n: usize) -> Option<char> {
        self.chars.peek_nth(n).cloned()
    }

    fn lex_linebreak(&mut self, ch: char) -> LexerResult {
        if ch == '\n' {
            return Ok(self.lex_lf());
        }
        self.lex_crlf()
    }

    fn lex_crlf(&mut self) -> Result<Token, LexerError> {
        self.advance();
        if self.peek_char() == Some('\n') {
            self.advance();
            self.update_span();
            self.count_line_break();
            return Ok(self.new_token(LineBreak));
        }
        // Only single \r found. Maybe emit warning in the future
        Err(LexerError::UnknownToken(self.current_span()))
    }

    fn lex_lf(&mut self) -> Token {
        self.advance();
        self.update_span();
        self.count_line_break();
        self.new_token(LineBreak)
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.update_span();
        let lexer_result: LexerResult = loop {
            if let Some(ch) = self.peek_char() {
                match ch {
                    '.' => {
                        self.advance();
                        break Ok(self.new_token(Dot));
                    }
                    '!' => {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            break Ok(self.new_token(ExclEquals));
                        }
                        break Ok(self.new_token(Excl));
                    }
                    '@' => {
                        self.advance();
                        break Ok(self.new_token(At));
                    }
                    ',' => {
                        self.advance();
                        break Ok(self.new_token(Comma));
                    }
                    '/' => {
                        self.advance();
                        if self.peek_char() == Some('/') {
                            // two slashes means a comment
                            if let Some(token) = self.skip_comment() {
                                return token;
                            } else {
                                continue;
                            }
                        }
                        break self.lex_operator(ch); // if we have a single slash, it's a divide operator
                    }
                    ' ' | '\t' => {
                        self.advance();
                        self.update_span(); // for every 'skipped' chararacter that doesnt result a token, we need to move the "last_token_end" cursor to the lexer's position.
                        continue;
                    } // Skip non-relevant whitespace
                    '\r' | '\n' => {
                        return self.lex_linebreak(ch);
                    }
                    '{' => {
                        self.advance();
                        break Ok(self.new_token(OpenBrace));
                    }
                    '}' => {
                        self.advance();
                        break Ok(self.new_token(CloseBrace));
                    }
                    '[' => {
                        self.advance();
                        break Ok(self.new_token(OpenBrack));
                    }
                    ']' => {
                        self.advance();
                        break Ok(self.new_token(CloseBrack));
                    }
                    '(' => {
                        self.advance();
                        break Ok(self.new_token(OpenParen));
                    }
                    ')' => {
                        self.advance();
                        break Ok(self.new_token(CloseParen));
                    }
                    ':' => {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            break Ok(self.new_token(DeclareAssign));
                        }
                        break Ok(self.new_token(Colon));
                    }
                    ';' => {
                        self.advance();
                        break Ok(self.new_token(Semicolon));
                    }
                    '=' => {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            break Ok(self.new_token(DoubleEquals));
                        }
                        break Ok(self.new_token(Equals));
                    }
                    '+' | '-' | '*' | '%' | '&' | '|' | '~' | '^' | '<' | '>' => {
                        break self.lex_operator(ch);
                    }
                    '0'..='9' => break self.lex_number(),
                    'a'..='z' | 'A'..='Z' | '_' => break Ok(self.lex_keyword_or_ident()),
                    '\"' => break self.lex_string_literal(),
                    _ => {
                        self.errors.push(LexerError::InvalidCharacter(ch, self.current_span()));
                    }
                }
            } else {
                return Ok(self.new_token(EndOfFile));
            }
        };

        return lexer_result;
    }

    fn lex_operator(&mut self, ch: char) -> LexerResult {
        self.advance();
        if ch == '<' {
            if self.peek_char() == Some('<') {
                // bit-shift left
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(self.new_token(BitShiftLeftAssign));
                }
                return Ok(self.new_token(BitShiftLeft));
            }
        }
        if ch == '>' {
            if self.peek_char() == Some('>') {
                // bit-shift right
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(self.new_token(BitShiftRightAssign));
                }
                return Ok(self.new_token(BitShiftRight));
            }
        }
        if self.peek_char() == Some('=') {
            let token_kind = match ch {
                '+' => AddAssign,
                '-' => SubAssign,
                '*' => MultAssign,
                '/' => DivAssign,
                '%' => ModAssign,
                '&' => BitAndAssign,
                '|' => BitOrAssign,
                '^' => BitXorAssign,
                '<' => LessOrEqual,
                '>' => GreaterOrEqual,
                '~' => BitNotAssign,
                _ => unreachable!(),
            };

            // is a 2 character operator
            self.advance();
            return Ok(self.new_token(token_kind));
        } else {
            let token_kind = match ch {
                '+' => Plus,
                '-' => Minus,
                '*' => Star,
                '/' => Slash,
                '%' => Percent,
                '&' => Ampersand,
                '|' => Pipe,
                '^' => Caret,
                '<' => Less,
                '>' => Greater,
                '~' => Tilde,
                _ => unreachable!(),
            };
            return Ok(self.new_token(token_kind));
        }
    }

    fn lex_number(&mut self) -> LexerResult {
        let mut num = String::new();

        let mut has_decimal_point = false;
        let mut has_exponent = false;
        let mut first_iter = true;

        while let Some(ch) = self.peek_char() {
            if first_iter && ch == '0' {
                if let Some(lex_result) = self.lex_non_decimal_literal(&mut num) {
                    return lex_result;
                } else {
                    num.push(ch);
                    self.advance();
                }
            } else if ch.is_digit(10) {
                num.push(ch);
                self.advance();
            } else if ch == '.' {
                if !has_decimal_point && !has_exponent {
                    has_decimal_point = true;
                    num.push(ch);
                    self.advance();
                } else {
                    return Err(LexerError::MultiDecimalPointInNumberLiteral(self.current_span()));
                }
            } else if ch == 'e' || ch == 'E' {
                if !has_exponent {
                    has_exponent = true;
                    num.push(ch);
                    self.advance();

                    // Exponents can optionally have a +/- sign
                    if let Some(next_ch) = self.peek_char() {
                        if next_ch == '+' || next_ch == '-' {
                            num.push(next_ch);
                            self.advance();
                        }
                    }
                } else {
                    break;
                }
            } else if ch == '_' {
                self.advance();
            } else {
                break;
            }
            first_iter = false;
        }

        // Determine the final token kind based on what we found
        let kind = if has_exponent {
            NumberLiteralKind::ScientificDecimal
        } else if has_decimal_point {
            NumberLiteralKind::DecimalPoint
        } else {
            // Assuming you have a standard integer kind, update this if your enum uses a different name
            NumberLiteralKind::DecimalInteger
        };

        Ok(Token::number_literal(
            num,
            kind,
            self.current_span(),
        ))
    }

    fn lex_non_decimal_literal(&mut self, num: &mut String) -> Option<Result<Token, LexerError>> {
        let prefix = self.peek_nth_char(1);
        if let Some(prefix) = prefix {
            match prefix {
                'x' => {
                    return Some(self.lex_non_decimal_literal_value(
                        num,
                        NumberLiteralKind::HexadecimalInteger,
                    ));
                }
                'o' => {
                    return Some(
                        self.lex_non_decimal_literal_value(num, NumberLiteralKind::OctalInteger),
                    );
                }
                'b' => {
                    return Some(
                        self.lex_non_decimal_literal_value(num, NumberLiteralKind::BinaryInteger),
                    );
                }
                '.' => return None,
                _ => {
                    if let Some(ch) = self.peek_nth_char(2) {
                        if ch.is_digit(16) {
                            return Some(Err(
                                LexerError::UnexpectedCharacterLexingNonDecimalNumberLiteral(
                                    self.current_span()
                                ),
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    fn lex_non_decimal_literal_value(
        &mut self,
        num: &mut String,
        literal_kind: NumberLiteralKind,
    ) -> LexerResult {
        // We capture the starting kind, but allow it to "upgrade" to ScientificHex
        let mut final_kind = literal_kind;
        let mut has_decimal = false;
        let mut has_exponent = false;

        self.advance_n(2);
        let mut first = true;

        while let Some(ch) = self.peek_char() {
            if ch == '_' {
                self.advance();
                continue;
            }

            if literal_kind == NumberLiteralKind::HexadecimalInteger {
                // Hex numbers can have A-F, a decimal '.', and an exponent 'p'/'P'
                if ch.is_digit(16) {
                    num.push(ch);
                    self.advance();
                } else if ch == '.' && !has_decimal && !has_exponent {
                    has_decimal = true;
                    final_kind = NumberLiteralKind::ScientificHex;
                    num.push(ch);
                    self.advance();
                } else if (ch == 'p' || ch == 'P') && !has_exponent {
                    has_exponent = true;
                    final_kind = NumberLiteralKind::ScientificHex;
                    num.push(ch);
                    self.advance();

                    // Hex exponents can also have a +/- sign
                    if let Some(next_ch) = self.peek_char() {
                        if next_ch == '+' || next_ch == '-' {
                            num.push(next_ch);
                            self.advance();
                        }
                    }
                } else {
                    break;
                }
            } else {
                // Binary and Octal fall back to the standard strict logic
                if ch.is_digit(literal_kind.get_radix()) {
                    num.push(ch);
                    self.advance();
                } else if ch.is_digit(16) || first {
                    return Err(
                        LexerError::UnexpectedCharacterLexingNonDecimalNumberLiteral(self.current_span()),
                    );
                } else {
                    break;
                }
            }
            first = false;
        }

        Ok(Token::number_literal(
            num.clone(),
            final_kind,
            (self.last_token_end, self.location).into(),
        ))
    }

    fn lex_keyword_or_ident(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        match ident.as_str() {
            "break" => self.new_token(Break),
            "defer" => self.new_token(Defer),
            "else" => self.new_token(Else),
            "false" => self.new_token(False),
            "fn" => self.new_token(Fn),
            "for" => self.new_token(For),
            "if" => self.new_token(If),
            "impl" => self.new_token(Impl),
            "in" => self.new_token(In),
            "is" => self.new_token(Is),
            "loop" => self.new_token(Loop),
            "next" => self.new_token(Next), // like continue
            "ret" => self.new_token(Ret),
            "struct" => self.new_token(Struct),
            "sum" => self.new_token(Sum),
            "true" => self.new_token(True),
            "trait" => self.new_token(Trait),
            "use" => self.new_token(Use),
            _ => Token::ident(ident, (self.last_token_end, self.location).into()),
        }
    }

    // Skips comment and returns a linebreak token if the comment ends with one.
    // maybe this is over-engineered because comments always end with a linebreak unless they are the last thing in a source file. But in that case it's irrelevant, maybe?
    fn skip_comment(&mut self) -> Option<Result<Token, LexerError>> {
        let mut line_break = None;
        while let Some(ch) = self.peek_char() {
            if ch == '\r' || ch == '\n' {
                line_break = Some(self.lex_linebreak(ch));
                break;
            } else {
                self.advance();
            }
        }
        self.update_span();
        line_break
    }

    // still need to add escaping of special characters. could do that later tho.
    fn lex_string_literal(&mut self) -> Result<Token, LexerError> {
        self.advance();
        let mut value = String::new();
        let mut ends_by_quote = false; // valid if string literal ends with another quote symbol "
        while let Some(ch) = self.peek_char() {
            self.advance();
            match ch {
                '\\' => {
                    self.handle_escape_character(&mut value)?;
                }
                '\"' => {
                    ends_by_quote = true;
                    break;
                }
                _ => {
                    value.push(ch);
                }
            }
        }
        if ends_by_quote {
            Ok(Token::string_literal(
                value,
                self.current_span(),
            ))
        } else {
            Err(LexerError::UnterminatedStringLiteral(self.current_span()))
        }
    }

    fn handle_escape_character(&mut self, literal: &mut String) -> Result<(), LexerError> {
        if let Some(ch) = self.peek_char() {
            let replacement = match ch {
                '\\' => Some('\\'),
                '\"' => Some('\"'),
                'r' => Some('\r'),
                'n' => Some('\n'),
                '0' => Some('\0'),
                't' => Some('\t'),
                '\'' => Some('\''),
                _ => None, // replace with proper error
            };
            if let Some(replacement) = replacement {
                self.advance();
                literal.push(replacement);
                return Ok(());
            }
        }
        Err(LexerError::UnknownEscapeCharacter(self.current_span()))
    }
}
