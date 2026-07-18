use std::{collections::VecDeque, usize, vec::IntoIter};

use crate::{
    CodeBlock, CodeSpan, ModulePath, TypedVar,
    ast::Ast,
    decl::{Decl, DeclKind, FnSignature, Variant, VariantData},
    error::{Diagnostic, ExprParseResult, ParserError},
    expr::{Expr, ExprKind, GenericParam, Ident, ImplDeclPart, TypeExpr},
    op::{BinaryOp, UnaryOp},
    stmt::{Stmt, StmtKind},
    token::{Token, TokenKind},
};
use itertools::{PeekNth, peek_nth};
use log::debug;

pub struct Parser {
    token_stream: PeekNth<IntoIter<Token>>,
    current_token: Option<Token>,
    ast: Ast,
    pub diagnostics: Vec<Diagnostic>,
    only_expr: bool, // if true only parses an expression, if false, parses full program structure.
    bracket_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, only_expr: bool) -> Self {
        let parser = Self {
            token_stream: peek_nth(tokens),
            current_token: None,
            ast: Ast::new(),
            diagnostics: Vec::new(),
            only_expr,
            bracket_depth: 0,
        };
        parser
    }

    /// Creates a CodeSpan starting and ending at the current token
    pub fn start_span_at_current(&self) -> CodeSpan {
        self.unwrap_current_token().span
    }

    /// Creates a CodeSpan starting and ending at the next token
    pub fn start_span_at_next(&mut self) -> CodeSpan {
        self.peek().unwrap().span
    }

    /// Sets the given span's end to the end of the current token
    pub fn end_span(&self, span: &mut CodeSpan) {
        span.end = self.unwrap_current_token().span.end;
    }

    pub fn parse(&mut self) -> &Ast {
        if self.only_expr {
            match self.expr(0) {
                Ok(expr) => {
                    let span = expr.span;
                    self.ast.push(Decl::new(DeclKind::LooseExpr(expr), span));
                }
                Err(error) => self.diagnostics.push(error.into()),
            }
        } else {
            if let Err(error) = self.parse_top_level_decls() {
                self.diagnostics.push(error.into());
            }
        }
        &self.ast
    }

    // PRATT PARSING >>>
    fn parse_prefix(&mut self) -> ExprParseResult {
        let token = self.advance().unwrap();
        use TokenKind::*;
        match token.kind {
            DecimalIntegerNumberLiteral
            | DecimalPointNumberLiteral
            | BinaryIntegerNumberLiteral
            | OctalIntegerNumberLiteral
            | HexadecimalIntegerNumberLiteral
            | ScientificDecimalNumberLiteral
            | ScientificHexNumberLiteral => Ok(Expr::new(
                ExprKind::NumberLiteral(
                    token.unwrap_value(),
                    token.kind.try_into().unwrap(), // converts the TokenKind into the NumberLiteralKind. Because we know the TokenKind is a number literal, the conversion shouldn't panic.
                ),
                token.span,
            )),
            Ident => {
                let mut span = self.start_span_at_current();
                let ident = self.parse_path_or_ident();
                self.end_span(&mut span);
                Ok(Expr::new(
                    ExprKind::Variable {
                        ident, // could "evolve into" function call, module identifier or just stay a simple variable.
                    },
                    span,
                ))
            }
            Minus | Excl | Tilde | Ampersand => {
                let operator = UnaryOp::try_from(token.kind).unwrap();
                let bp = operator.prefix_binding_power();
                let start = token.span.start; // taking the start position of the operator.

                let right = self.expr(bp)?; // We call expr recursively to get what this operator applies to
                Ok(Expr::unary(start, operator, right))
            }
            For => return self.parse_for_loop(),
            True => Ok(Expr::new(ExprKind::BoolLiteral(true), token.span)),
            False => Ok(Expr::new(ExprKind::BoolLiteral(false), token.span)),
            StringLiteral => Ok(Expr::new(
                ExprKind::StringLiteral(token.unwrap_value()),
                token.span,
            )),
            OpenParen => {
                let mut span = self.start_span_at_current();
                let inner = self.expr(0)?;
                self.try_consume_token(CloseParen, "Expected ')'")?;
                self.end_span(&mut span);
                Ok(Expr::new(ExprKind::Grouping(inner.boxed()), span))
            }
            OpenBrack => self.parse_array_literal(),
            If => self.parse_if_else_expr(),
            _ => Err(ParserError::UnexpectedToken {
                msg: "Expected an expression".into(),
                peeking: Some(token.clone()),
                span: token.span,
            }),
        }
    }

    fn expr(&mut self, min_bp: u8) -> ExprParseResult {
        // 2 + 2 * 3
        // * (3,4) // higher precedence means the (sub)expression is evaluated before the lower precedence operators. That means it's nested deeper in the AST.
        // + (1,2)

        let mut span = self.start_span_at_next();
        let mut left = self.parse_prefix()?; // handles literals, (groups), and prefix ! - *

        loop {
            let op_token = match self.peek() {
                Some(t) => t.clone(),
                None => break,
            };
            let (l_bp, r_bp) = match op_token.infix_binding_power() {
                Some(bp) => bp,
                None => break,
            };
            if l_bp < min_bp {
                break;
            }

            self.advance();

            if op_token.is_assignment_operator() {
                let right = self.expr(r_bp)?;
                span.merge(right.span);
                left = Expr::new(
                    ExprKind::Assign {
                        assignee: Box::new(left),
                        operator: op_token.kind,
                        value: Box::new(right),
                    },
                    span,
                );
                continue;
            }

            match op_token.kind {
                TokenKind::OpenParen => {
                    let args = self.parse_fn_call_args()?;
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::FnCall {
                            callee: Box::new(left),
                            args,
                        },
                        span,
                    );
                    continue;
                }
                TokenKind::Dot => {
                    // Zig-style postfix deref operator *
                    // Example: <expr>.*
                    if self.matches_advance(TokenKind::Star) {
                        self.end_span(&mut span);
                        left = Expr::new(
                            ExprKind::Unary {
                                operator: UnaryOp::Deref,
                                expr: Box::new(left),
                            },
                            span,
                        );
                        continue;
                    }

                    self.skip_line_terminators();

                    self.try_consume_token(
                        TokenKind::Ident,
                        "Expected identifier or '*' after '.'",
                    )?;
                    let member_ident = self.parse_path_or_ident();
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::FieldAccess {
                            called_on: Box::new(left),
                            member_ident,
                        },
                        span,
                    );
                }
                TokenKind::OpenBrack => {
                    let index_expr = self.expr(0)?;

                    self.try_consume_token(
                        TokenKind::CloseBrack,
                        "Expected ']' after array index",
                    )?;
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::ArrayAccessor {
                            array: Box::new(left),
                            index: Box::new(index_expr),
                        },
                        span,
                    );
                    continue;
                }
                // Explicit Generic Function Call (e.g., foo:[T])
                TokenKind::Colon => {
                    self.try_consume_token(
                        TokenKind::OpenBrack,
                        "Expected '[' after ':' for generic call",
                    )?;

                    let mut type_args = Vec::new();
                    if !self.matches_advance(TokenKind::CloseBrack) {
                        loop {
                            self.skip_line_terminators();
                            type_args.push(self.parse_type_expr()?); // Pure TypeExpr parsing!
                            self.skip_line_terminators();

                            if self.matches_advance(TokenKind::CloseBrack) {
                                break;
                            }
                            if self.matches_advance(TokenKind::Comma) {
                                continue;
                            }

                            return ParserError::unexpected_token(
                                "Expected ',' or ']'",
                                self.peek().cloned(),
                                span,
                            )
                            .wrap();
                        }
                    }
                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::GenericFnCallPart {
                            callee: Box::new(left),
                            type_args: Some(type_args),
                        },
                        span,
                    );
                    continue;
                }
                _ => {
                    // Try to turn the token into a BinaryOp
                    let op = BinaryOp::try_from(op_token.kind)
                        .expect("should've been handled in infix_binding_power");
                    // We can unwrap/expect because infix_binding_power returns None and breaks the loop before we get here.
                    self.skip_line_terminators(); // allows to have binary expressions span across multiple lines
                    let right = self.expr(r_bp)?;

                    self.end_span(&mut span);
                    left = Expr::new(
                        ExprKind::Binary {
                            left_expr: Box::new(left),
                            operator: op,
                            right_expr: Box::new(right),
                        },
                        span,
                    );
                }
            }
        }

        Ok(left)
    }

    // <<< PRATT PARSING END

    /// Parses the top level declarations within .mo files that declare items.
    fn parse_top_level_decls(&mut self) -> Result<(), ParserError> {
        loop {
            if let Some(token) = self.peek().cloned() {
                match token.kind {
                    TokenKind::Use => {
                        let use_decl = self.parse_use_decl()?;
                        self.ast.push(use_decl);
                    }
                    TokenKind::Struct => {
                        let struct_decl = self.parse_struct_decl()?;
                        self.ast.push(struct_decl);
                    }
                    TokenKind::Sum => {
                        let sum_decl = self.parse_sum_decl()?;
                        self.ast.push(sum_decl);
                    }
                    TokenKind::Fn => {
                        let fn_decl = self.parse_fn_decl()?;
                        self.ast.push(fn_decl);
                    }
                    TokenKind::Trait => {
                        let trait_decl = self.parse_trait_decl()?;
                        self.ast.push(trait_decl);
                    }
                    TokenKind::LineBreak => {
                        self.advance();
                        continue; // continue for now
                    }
                    TokenKind::EndOfFile => {
                        self.advance();
                        break;
                    }
                    TokenKind::Impl => {
                        let impl_decl = self.parse_impl_decl()?;
                        self.ast.push(impl_decl);
                    }
                    _ => {
                        // Just add error, don't stop the loop...
                        self.diagnostics.push(ParserError::unexpected_token(
                            "Unexpected token parsing top-level declarations",
                            Some(token.clone()),
                            token.span,
                        ).into());
                        self.synchronize_top_level();
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn synchronize_top_level(&mut self) {
        // Consume the offending token that triggered the error.
        // If you don't do this, you might end up in an infinite loop
        // re-evaluating the same bad token.
        self.advance();

        // Skip tokens until we find something that looks like
        // the start of a valid top-level declaration.

        // TODO: We need to rewrite all of this if we start to allow use, function, struct and sum declarations inside of functions ...
        self.skip_tokens_if_predicate(|parser| {
            let is_unambiguous_decl = parser.is_next_of_kinds(&[
                TokenKind::Fn,
                TokenKind::Struct,
                TokenKind::Use,
                TokenKind::Sum,
                TokenKind::Trait,
            ]);

            let is_top_lvl_impl_decl =
                parser.is_next_of_kind(TokenKind::Impl) && parser.bracket_depth == 0;
            let keep_skipping = !(is_unambiguous_decl || is_top_lvl_impl_decl);
            // we only stop skipping if we find an unambiguous declaration (which is indicated by the tokens Fn, Struct Use, Sum and Trait)
            // or if we find an impl token that isnt inside of square brackets
            keep_skipping
        });
    }

    /// Parses use declaration assuming to peek 'use' token
    /// use_decl <- 'use' ident(':'ident)* string_literal?
    ///
    /// # Examples:
    /// use io | use io "foo"
    /// use io:print | use io:print "printie"
    fn parse_use_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance(); // consume 'use'
        let mut span = self.start_span_at_current();

        self.try_consume_token(TokenKind::Ident, "Expected identifier")?;
        let path = self.parse_path_or_ident();

        let alias = if self.matches_advance(TokenKind::StringLiteral) {
            Some(self.unwrap_current_token().unwrap_value())
        } else {
            None
        };
        self.end_span(&mut span);
        Ok(Decl::new(DeclKind::Use { path, alias }, span))
    }

    /// fn_signature <- 'fn' ident generic_type_params? '(' ( fn_param ( ',' fn_param )* )* ')' type_expr?
    /// generic_type_params <- '['(ident(',' ident)*)?']'
    /// fn_param <- type_expr ident
    fn parse_fn_signature(&mut self) -> Result<FnSignature, ParserError> {
        self.advance(); // consume 'fn'

        self.try_consume_token(TokenKind::Ident, "Expected function identifier")?;
        let ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;

        self.try_consume_token(TokenKind::OpenParen, "Expected open parenthesis")?;

        // Parse parameters
        let mut params = Vec::new();
        if !self.matches_advance(TokenKind::CloseParen) {
            loop {
                self.try_consume_token(TokenKind::Ident, "Expected variable identifier")?;
                let var_ident = self.unwrap_current_token().unwrap_value();

                let type_expr = self.parse_type_expr()?;
                params.push(TypedVar::new(var_ident, type_expr));

                if self.matches_advance(TokenKind::CloseParen) {
                    break;
                }
                if self.matches_advance(TokenKind::Comma) {
                    continue;
                }
                let token = self.peek().cloned();
                self.diagnostics.push(ParserError::unexpected_token(
                    "Expected argument delimiter ',' or closed parenthesis ')'",
                    token.clone(),
                    token.unwrap().span,
                ).into());
                self.advance();
            }
        }
        // Parse return type
        let mut return_type = None;
        if !self.matches_any(&[TokenKind::LineBreak, TokenKind::OpenBrace]) {
            let type_expr = self.parse_type_expr()?;
            return_type = Some(type_expr);
        }
        Ok(FnSignature {
            ident,
            generics,
            params,
            return_type,
        })
    }

    /// fn_decl <- fn_signature code_block
    fn parse_fn_decl(&mut self) -> Result<Decl, ParserError> {
        let mut span = self.start_span_at_next();

        let signature = self.parse_fn_signature()?;
        self.skip_tokens(TokenKind::LineBreak);

        let body = self.parse_code_block()?;

        self.end_span(&mut span);
        Ok(Decl::new(DeclKind::Fn { signature, body }, span))
    }

    /// code_block <- '{' (stmt lt)* '}'
    /// lt <- line_break | ';'
    fn parse_code_block(&mut self) -> Result<CodeBlock, ParserError> {
        debug!("parsing code block");
        self.try_consume_token(TokenKind::OpenBrace, "Expected open brace")?;

        self.skip_tokens(TokenKind::LineBreak); // move on if there's a linebreak.

        let mut code_block = CodeBlock::new();
        loop {
            if self.matches_advance(TokenKind::CloseBrace) {
                debug!("matched closebrace, returning codeblock");
                return Ok(code_block);
            }
            let stmt = self.parse_stmt()?;
            code_block.stmts.push(stmt);
            self.skip_tokens(TokenKind::LineBreak);
        }
    }

    /// Parses optional generic parameters with optional trait bounds
    /// generic_params <- '[' ( generic_param ( ',' generic_param ','? )* )* ']'
    /// generic_param <- ident ('impl' (ident)+)*
    ///
    /// Examples:
    /// '[T, U]'
    /// '[T impl Trait1 Trait2]
    /// '[T impl Trait1 Trait2, U impl Trait1 Trait2]
    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, ParserError> {
        let mut generics = Vec::new();
        // Check if there is an open paren. If not, return empty vec!
        if self.matches_advance(TokenKind::OpenBrack) {
            if !self.matches(TokenKind::CloseBrack) {
                loop {
                    self.try_consume_token(TokenKind::Ident, "Expected generic type identifier")?;
                    let generic_ident = self.unwrap_current_token().unwrap_value();

                    let mut bounds = Vec::new();

                    // If we see an impl, parse the trait bounds!
                    if self.matches_advance(TokenKind::Impl) {
                        loop {
                            self.try_consume_token(TokenKind::Ident, "Expected trait")?;
                            bounds.push(self.parse_path_or_ident());

                            if !self.matches(TokenKind::Ident) {
                                break;
                            }
                        }
                    }

                    // Add the generic_ident and its bounds to your list
                    generics.push(GenericParam {
                        ident: generic_ident,
                        bounds: Some(bounds),
                    });

                    if self.matches_advance(TokenKind::CloseBrack) {
                        break;
                    }
                    // Allow trailing commas!
                    if self.matches_advance(TokenKind::Comma) {
                        self.skip_line_terminators();
                        if self.matches_advance(TokenKind::CloseBrack) {
                            break;
                        }
                        continue;
                    }
                    let token = self.peek();
                    return Err(ParserError::unexpected_token(
                        "Expected ',' or ')'",
                        token.cloned(),
                        token.unwrap().span,
                    ));
                }
            }
        }
        Ok(generics)
    }

    /// Parses optional trait implementations: 'impl Trait1, Trait2' or 'impl Trait1[i32], Trait2[string]
    fn parse_impl_decl(&mut self) -> Result<Decl, ParserError> {
        let mut traits = Vec::new();
        let mut span = self.start_span_at_next();
        if self.matches_advance(TokenKind::Impl) {
            loop {
                self.try_consume_token(TokenKind::Ident, "Expected trait")?;
                let ident = self.parse_path_or_ident();

                let mut args = Vec::new();
                if self.matches_advance(TokenKind::OpenBrack) {
                    if !self.matches_advance(TokenKind::CloseBrack) {
                        loop {
                            self.skip_line_terminators();

                            // Trait arguments are full type expressions
                            args.push(self.parse_type_expr()?);

                            self.skip_line_terminators();

                            if self.matches_advance(TokenKind::CloseBrack) {
                                break;
                            }
                            if self.matches_advance(TokenKind::Comma) {
                                continue;
                            }
                            let token = self.peek();
                            return Err(ParserError::unexpected_token(
                                "Expected ',' or ']'",
                                token.cloned(),
                                token.unwrap().span,
                            ));
                        }
                    }
                }

                traits.push(ImplDeclPart { ident, args });
                if !self.matches_advance(TokenKind::Comma) {
                    break;
                }
                self.skip_line_terminators();
            }
        }
        self.end_span(&mut span);
        Ok(Decl {
            span,
            kind: DeclKind::Impl { parts: traits },
        })
    }

    /// stmt <- ret_stmt | defer_stmt | break_stmt | next_stmt | code_block | var_decl | expr
    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        let mut span = self.start_span_at_next(); // this span is only used when no statement could be parsed
        if let Some(token) = self.peek().cloned() {
            match token.kind {
                TokenKind::Ret => return self.parse_ret_stmt(),
                TokenKind::Defer => return self.parse_defer_stmt(),
                TokenKind::Break => return self.parse_break_stmt(),
                TokenKind::Next => {
                    self.advance();
                    return Ok(Stmt::new(StmtKind::Next, token.span));
                }
                TokenKind::OpenBrace => {
                    let mut span = self.start_span_at_next();
                    let code_block = self.parse_code_block()?;
                    self.end_span(&mut span);
                    return Ok(Stmt::new(StmtKind::CodeBlock(code_block), span));
                }
                TokenKind::Ident => {
                    // Parse variable declarations
                    if let Some(var_decl) = self.parse_var_decl()? {
                        // Make sure to consume the possible line break / semicolon here
                        self.skip_line_terminators();
                        return Ok(var_decl);
                    }
                }
                _ => {}
            }

            // If it wasn't a dedicated statement or a variable declaration,
            // it MUST be an expression statement (e.g. `foo()` or `a = 10` or `a.b += 5`)

            let mut span = self.start_span_at_next();
            let expr = self.parse_expression()?;

            // If the next token isn't a valid terminator or the end of a block, throw an error!
            if !self.matches_any(&[
                TokenKind::LineBreak,
                TokenKind::Semicolon,
                TokenKind::CloseBrace,
            ]) {
                let token = self.peek();
                return Err(ParserError::unexpected_token(
                    "Expected newline or semicolon after expression statement",
                    token.cloned(),
                    token.unwrap().span,
                ));
            }

            self.end_span(&mut span);
            self.skip_line_terminators();
            return Ok(Stmt::new(StmtKind::Expr(expr), span));
        }

        self.end_span(&mut span);
        Err(ParserError::unexpected_token(
            "Couldn't parse statement",
            self.peek().cloned(),
            span,
        ))
    }

    // var_decl <- ident type_expr '=' expr | ident ':=' expr
    //
    // Examples:
    // a i32 = 10    | DeclAssignmt
    // a := 10       | DeclAssignmt (no type identifier and ':=', type to be inferred)
    fn parse_var_decl(&mut self) -> Result<Option<Stmt>, ParserError> {
        let mut span = self.start_span_at_current();

        // <ident> := <expr>
        if self
            .peek_nth(1)
            .is_some_and(|t| t.is_of_kind(TokenKind::ColonEquals))
        {
            self.advance_n(2);
            let ident = self.unwrap_current_token().unwrap_value();
            let expr = self.parse_expression()?;
            self.end_span(&mut span);
            let stmt = Stmt::new(
                StmtKind::LocalVarDeclAssign {
                    ident,
                    type_expr: None,
                    value: expr,
                },
                span,
            );
            return Ok(Some(stmt));
        }

        // <ident> <type> = <expr>

        if self.matches_nth(1, TokenKind::Ident) {
            if self.matches_nth_any(2, &[TokenKind::Ident, TokenKind::Ampersand]) {
                // alright, we have a type expression here.
                let ident = self.advance().unwrap().unwrap_value();
                let type_expr = self.parse_type_expr();
                let peeking = self.peek().cloned();
                match type_expr {
                    Ok(type_expr) => {
                        let value = self.parse_expression()?;
                        return Ok(Some(Stmt::new(
                            StmtKind::LocalVarDeclAssign {
                                ident,
                                type_expr: Some(type_expr),
                                value,
                            },
                            span,
                        )));
                    }
                    Err(_) => self.diagnostics.push(ParserError::UnexpectedToken {
                        msg: "expected type expr".into(),
                        peeking,
                        span,
                    }.into()),
                }; // horrible error handling, missing synchronization etc.... really need to tackle this.
            }
        }
        Ok(None)
    }

    // array_literal <- '[' ( expr ( ',' expr )* )* ']'
    // Example: '[1,2,3,4,5,6,7,8]'
    fn parse_array_literal(&mut self) -> Result<Expr, ParserError> {
        let mut elements = Vec::new();
        let mut span = self.start_span_at_current();
        if !self.matches_advance(TokenKind::CloseBrack) {
            loop {
                let element = self.parse_expression()?;
                elements.push(element);

                self.skip_line_terminators();

                if self.matches_advance(TokenKind::CloseBrack) {
                    break;
                }
                if self.matches_advance(TokenKind::Comma) {
                    self.skip_line_terminators();

                    if self.matches_advance(TokenKind::CloseBrack) {
                        break;
                    }
                    continue;
                }

                let token = self.peek();
                return ParserError::unexpected_token(
                    "Expected comma ',' or closed bracket ']'",
                    token.cloned(),
                    token.unwrap().span,
                )
                .wrap();
            }
        }
        self.end_span(&mut span);
        return Ok(Expr::new(ExprKind::ArrayLiteral { elements }, span));
    }

    // starts with Ampersand or Ident
    // type_expr <- pointer_type | ident_type
    #[track_caller]
    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParserError> {
        debug!("parsing type expr from: {}", std::panic::Location::caller());
        let mut span = self.start_span_at_next();

        /*
        // array_type <- '[' decimal_integer_number_literal? ']' type_expr
        let mut array_length = None;
        if self.matches_advance(TokenKind::OpenBrack) {
            if self.matches_any_advance(&[TokenKind::DecimalIntegerNumberLiteral]) {
                let length: usize = self
                    .unwrap_current_token()
                    .unwrap_value()
                    .parse()
                    .expect("If this panics, there is a lexer bug");
                array_length = Some(length);
            }

            self.try_consume_token(TokenKind::CloseBrack, "Expected closed bracket ']'")?;
            return Ok(TypeExpr::Array {
                length: array_length,
                type_expr: Box::new(self.parse_type_expr()?),
            });
        }
         */

        // reference_type <- '&' type_expr
        if self.matches_advance(TokenKind::Ampersand) {
            return Ok(TypeExpr::reference(self.parse_type_expr()?));
        }

        // identifier with optional module path prefix
        // module_prefix <- (ident ':')+
        // generic_args <- '[' (type_expr','*)* ']'
        //
        // ident_type <- module_prefix? ident generic_args?
        if self.matches_any_advance(&[TokenKind::Ident]) {
            let base_ident = self.parse_path_or_ident();

            // generic type
            if self.matches_advance(TokenKind::OpenBrack) {
                let mut generic_args = Vec::new();
                if !self.matches(TokenKind::CloseBrack) {
                    loop {
                        self.skip_line_terminators();

                        generic_args.push(self.parse_type_expr()?);
                        self.skip_line_terminators();

                        if self.matches_advance(TokenKind::CloseBrack) {
                            break;
                        }
                        if self.matches_advance(TokenKind::Comma) {
                            continue;
                        }
                        let token = self.peek();
                        return ParserError::unexpected_token(
                            "Expected ',' or ']' after generic type argument",
                            token.cloned(),
                            token.unwrap().span,
                        )
                        .wrap();
                    }
                }
                return Ok(TypeExpr::Generic {
                    ident: base_ident,
                    args: generic_args,
                });
            }

            // not generic, just simple ident.
            return Ok(TypeExpr::Ident(base_ident));
        }
        self.end_span(&mut span);
        ParserError::unexpected_token("Expected type expression", self.peek().cloned(), span).wrap()
    }

    #[allow(dead_code)]
    fn consume_line_terminator(&mut self) -> Result<(), ParserError> {
        self.try_consume_token2(
            &[TokenKind::LineBreak, TokenKind::Semicolon],
            "Expected line break or ;",
        )
    }

    // fn_call_args <- '(' (expr ','*)* ')'
    fn parse_fn_call_args(&mut self) -> Result<Vec<Expr>, ParserError> {
        // parse arguments
        let mut args = Vec::new();

        if !self.matches_advance(TokenKind::CloseParen) {
            loop {
                let expr = self.parse_expression()?;
                args.push(expr);

                self.skip_line_terminators();

                if self.matches_advance(TokenKind::CloseParen) {
                    break;
                }
                if self.matches_advance(TokenKind::Comma) {
                    self.skip_line_terminators(); // added this

                    // trailing commas
                    // If we skipped the comma/newlines and hit a ')', we are done.
                    if self.matches_advance(TokenKind::CloseParen) {
                        break;
                    }
                    continue;
                }
                let token = self.peek();
                return ParserError::unexpected_token(
                    "Expected argument delimiter ',' or closed parenthesis ')'",
                    token.cloned(),
                    token.unwrap().span,
                )
                .wrap();
            }
        }
        Ok(args)
    }

    // TODO:
    // for-in loop for collections, for loop without condition (like loop keyword in Rust)
    // TODO: Make it be an expression for break with value.
    //
    // Parsing:
    // for_loop <- 'for' expr? code_block
    fn parse_for_loop(&mut self) -> Result<Expr, ParserError> {
        let mut span = self.start_span_at_current();
        if let Some(token) = self.peek()
            && token.is_of_kind(TokenKind::OpenBrace)
        {
            let code_block = self.parse_code_block()?;
            self.end_span(&mut span);
            return Ok(Expr::new(
                ExprKind::ForLoop {
                    condition: None,
                    code_block,
                },
                span,
            ));
        }
        let condition = Some(self.parse_expression()?.boxed());
        let code_block = self.parse_code_block()?;
        self.end_span(&mut span);
        let stmt = Expr::new(
            ExprKind::ForLoop {
                condition,
                code_block,
            },
            span,
        );
        Ok(stmt)
    }

    // TODO:
    // if is expr (like switch or match)
    //
    // if_else_expr <- 'if' code_block ('else' code_block)?
    fn parse_if_else_expr(&mut self) -> Result<Expr, ParserError> {
        let mut span = self.start_span_at_current();
        let condition = self.parse_expression()?.boxed();
        let if_block = self.parse_code_block()?;
        let else_block = if self.matches_advance(TokenKind::Else) {
            Some(self.parse_code_block()?)
        } else {
            None
        };

        self.end_span(&mut span);
        Ok(Expr::new(
            ExprKind::If {
                condition,
                if_block,
                else_block,
            },
            span,
        ))
    }

    /// defer_stmt <- 'defer' stmt
    fn parse_defer_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        let stmt = self.parse_stmt()?;
        self.end_span(&mut span);
        Ok(Stmt::new(StmtKind::Defer(Box::new(stmt)), span))
    }

    /// break_stmt <- 'break' expr?
    fn parse_break_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        if self.matches_any(&[TokenKind::LineBreak, TokenKind::Semicolon]) {
            self.end_span(&mut span);
            return Ok(Stmt::new(StmtKind::Break { value: None }, span));
        }
        let expr = self.parse_expression()?;
        self.end_span(&mut span);
        Ok(Stmt::new(StmtKind::Break { value: Some(expr) }, span))
    }

    /// ret_stmt <- 'ret' expr?
    fn parse_ret_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        if self.matches_advance(TokenKind::LineBreak) {
            self.end_span(&mut span);
            Ok(Stmt::new(StmtKind::Ret(None), span))
        } else {
            let expr = self.parse_expression()?;
            Ok(Stmt::new(StmtKind::Ret(Some(expr)), span))
        }
    }

    /*
    Parse struct declaration of form:
    struct Foo {
        a int
        b int
    }
    struct Foo[T] {
        a T
        b T
    }
    unit struct:
    struct Foo {}
     */
    ///
    /// struct_decl <- 'struct' ident generic_params?
    fn parse_struct_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance();
        let mut span = self.start_span_at_current();
        self.try_consume_token(TokenKind::Ident, "Expected struct identifier")?;
        let struct_ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;
        //let impl_traits = self.parse_impl_traits()?;

        self.skip_tokens(TokenKind::LineBreak);
        self.try_consume_token(TokenKind::OpenBrace, "Expected open brace or impl")?; // this error message is weird
        let mut fields = Vec::new();
        loop {
            self.skip_tokens(TokenKind::LineBreak);
            if self.matches_advance(TokenKind::CloseBrace) {
                break;
            }
            self.try_consume_token(TokenKind::Ident, "Expected variable identifier")?;
            let var_ident = self.unwrap_current_token().unwrap_value();

            let type_expr = self.parse_type_expr()?;
            fields.push(TypedVar::new(var_ident, type_expr));
            if !self
                .peek()
                .is_some_and(|t| t.is_of_kind(TokenKind::CloseBrace))
            {
                // If next isn't token CloseBrace, means we are expecting next struct field declaration
                self.try_consume_token2(
                    &[TokenKind::LineBreak, TokenKind::Comma],
                    "Expected comma ',' or linebreak",
                )?;
            }
        }
        self.end_span(&mut span);
        Ok(Decl::new(
            DeclKind::Struct {
                ident: struct_ident,
                fields,
                generics,
            },
            span,
        ))
    }

    ///
    /// sum_decl <- 'sum' ident
    fn parse_sum_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance(); // consume 'sum' (assuming TokenKind::Sum)
        let mut span = self.start_span_at_current();

        self.try_consume_token(TokenKind::Ident, "Expected sum type identifier")?;
        let sum_ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;

        self.skip_line_terminators();
        self.try_consume_token(TokenKind::OpenBrace, "Expected '{'")?;

        debug!("matched closebrace, returning codeblock");
        let mut variants = Vec::new();

        loop {
            self.skip_line_terminators();
            if self.matches_advance(TokenKind::CloseBrace) {
                break;
            }

            self.try_consume_token(TokenKind::Ident, "Expected variant identifier")?;
            let variant_ident = self.unwrap_current_token().unwrap_value();

            // Figure out which kind of variant this is
            let data = if self.matches_advance(TokenKind::OpenParen) {
                // 1. Tuple Variant
                let mut types = Vec::new();
                if !self.matches_advance(TokenKind::CloseParen) {
                    loop {
                        self.skip_line_terminators();
                        types.push(self.parse_type_expr()?);
                        self.skip_line_terminators();

                        if self.matches_advance(TokenKind::CloseParen) {
                            break;
                        }
                        self.try_consume_token(TokenKind::Comma, "Expected ',' or ')'")?;
                    }
                }
                VariantData::Tuple(types)
            } else if self.matches_advance(TokenKind::OpenBrace) {
                // 2. Struct Variant
                let mut fields = Vec::new();
                loop {
                    self.skip_line_terminators();
                    if self.matches_advance(TokenKind::CloseBrace) {
                        break;
                    }

                    self.try_consume_token(TokenKind::Ident, "Expected field identifier")?;
                    let field_ident = self.unwrap_current_token().unwrap_value();
                    let type_expr = self.parse_type_expr()?;

                    fields.push(TypedVar::new(field_ident, type_expr));

                    // Fields can be separated by commas or linebreaks
                    self.matches_any_advance(&[TokenKind::Comma, TokenKind::LineBreak]);
                }
                VariantData::Struct(fields)
            } else {
                // 3. Unit Variant
                VariantData::Unit
            };

            variants.push(Variant {
                ident: variant_ident,
                data,
            });

            // Variants themselves must be separated by commas or linebreaks
            if !self
                .peek()
                .is_some_and(|t| t.is_of_kind(TokenKind::CloseBrace))
            {
                self.try_consume_token2(
                    &[TokenKind::LineBreak, TokenKind::Comma],
                    "Expected ',', or linebreak between variants",
                )?;
            }
        }

        self.end_span(&mut span);

        Ok(Decl::new(
            DeclKind::Sum {
                ident: sum_ident,
                generics,
                variants,
            },
            span,
        ))
    }

    fn parse_trait_decl(&mut self) -> Result<Decl, ParserError> {
        self.advance(); // Consume 'trait' keyword
        let mut span = self.start_span_at_current();

        self.try_consume_token(TokenKind::Ident, "Expected trait identifier")?;
        let ident = self.unwrap_current_token().unwrap_value();

        let generics = self.parse_generic_params()?;

        self.skip_line_terminators();
        self.try_consume_token(TokenKind::OpenBrace, "Expected '{'")?;

        let mut methods = Vec::new();

        loop {
            self.skip_line_terminators();
            if self.matches_advance(TokenKind::CloseBrace) {
                break;
            }

            methods.push(self.parse_fn_signature()?);

            // Allow (but don't strictly require) linebreaks/semicolons between methods
            self.skip_line_terminators();
        }

        self.end_span(&mut span);

        Ok(Decl::new(
            DeclKind::Trait {
                ident,
                generics,
                methods,
            },
            span,
        ))
    }

    // :Expressions

    fn parse_expression(&mut self) -> ExprParseResult {
        debug!("parsing expression {}", self.parser_state_dbg_info());
        self.expr(0)
    }

    // :Utils

    /// Parses a path or simple ident.
    /// Assumes the first identifier has just been consumed.
    fn parse_path_or_ident(&mut self) -> Ident {
        let first_ident = self.unwrap_current_token().unwrap_value();
        let mut path_parts = VecDeque::new();
        path_parts.push_back(first_ident);

        while self.is_next_of_kind(TokenKind::Colon) {
            if let Some(next_token) = self.peek_nth(1) {
                if next_token.kind == TokenKind::Ident {
                    self.advance(); // consume ':'
                    self.advance(); // consume Ident
                    path_parts.push_back(self.unwrap_current_token().unwrap_value());
                    continue;
                }
            }
            break; // Break if it's not followed by an Ident (e.g. `:[` generic call)
        }

        if path_parts.len() > 1 {
            let ident_name = path_parts.pop_back().unwrap();
            let module_path = ModulePath { path: path_parts };
            Ident::WithModulePrefix(module_path, ident_name)
        } else {
            Ident::Simple(path_parts.pop_front().unwrap())
        }
    }

    /// Gets and clones current token.
    /// # Panics
    /// If current token is None
    fn unwrap_current_token(&self) -> Token {
        self.current_token.clone().unwrap()
    }

    fn current_token(&self) -> Option<Token> {
        self.current_token.clone()
    }

    #[track_caller]
    fn advance(&mut self) -> Option<Token> {
        self.current_token = self.token_stream.next();

        if let Some(current_token) = &self.current_token() {
            match current_token.kind {
                TokenKind::OpenBrack => self.bracket_depth += 1,
                TokenKind::CloseBrack => self.bracket_depth = self.bracket_depth.saturating_sub(1),
                _ => {}
            }
        }

        // Debug info:
        if let Some(current_token) = &self.current_token() {
            if let Some(peeked_token) = self.token_stream.peek().cloned() {
                debug!(
                    "{} advanced from {}, peeking {}",
                    std::panic::Location::caller(),
                    current_token,
                    peeked_token
                );
            }
        }
        return self.current_token();
    }

    /// Advances n times.
    #[allow(dead_code)]
    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    /// Skips tokens that are of the same kind as given.
    fn skip_tokens(&mut self, token_kind: TokenKind) {
        loop {
            if let Some(token) = self.peek() {
                if token.is_of_kind(token_kind) {
                    let token = token.clone();
                    self.advance();
                    debug!("Skipped token: {}", token);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Skips tokens that are of any of the kinds given
    #[allow(dead_code)]
    fn skip_tokens_of_kinds(&mut self, token_kinds: &[TokenKind]) {
        loop {
            if let Some(token) = self.peek() {
                if token.is_of_any_kinds(token_kinds) {
                    debug!("Peeking token: {}, skipping", token);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Skips all tokens that are not of the given kinds until it hits a token of the kinds given
    #[allow(dead_code)]
    fn skip_tokens_unless_of_kinds(&mut self, token_kinds: &[TokenKind]) {
        self.skip_tokens_if_predicate(|parser| !parser.is_next_of_kinds(token_kinds));
    }

    /// Skips all tokens if the predicate evaluates to true until it evaluates to false
    fn skip_tokens_if_predicate<F>(&mut self, predicate: F)
    where
        F: Fn(&mut Self) -> bool,
    {
        loop {
            if let Some(token) = self.peek().cloned() {
                if predicate(self) {
                    let token = token.clone();
                    self.advance();
                    debug!("Skipped token: {}", token);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn skip_line_terminators(&mut self) {
        self.skip_tokens_of_kinds(&[TokenKind::LineBreak, TokenKind::Semicolon]);
    }

    fn peek(&mut self) -> Option<&Token> {
        self.token_stream.peek()
    }

    #[allow(dead_code)]
    fn peek_nth(&mut self, n: usize) -> Option<&Token> {
        self.token_stream.peek_nth(n) // Note: maybe check if this causes problems at the end of files...
    }

    /// checks if next token is of one of the given types, then moves on to that token
    fn matches_any_advance(&mut self, tokens: &[TokenKind]) -> bool {
        if self.is_next_of_kinds(tokens) {
            self.advance();
            return true;
        }
        false
    }

    #[track_caller]
    fn matches_advance(&mut self, token_kind: TokenKind) -> bool {
        debug!("{}", std::panic::Location::caller());
        if self.is_next_of_kind(token_kind) {
            self.advance();
            return true;
        }
        false
    }

    /// checks if next token is of one of the given types
    fn matches_any(&mut self, token_kinds: &[TokenKind]) -> bool {
        if self.is_next_of_kinds(token_kinds) {
            return true;
        }
        false
    }

    #[allow(dead_code)]
    #[track_caller]
    fn matches(&mut self, token_kind: TokenKind) -> bool {
        debug!("{}", std::panic::Location::caller());
        if self.is_next_of_kind(token_kind) {
            return true;
        }
        false
    }

    /// Checks if token at nth position
    fn matches_nth(&mut self, n: usize, token_kind: TokenKind) -> bool {
        if let Some(token) = self.peek_nth(n) {
            return token.is_of_kind(token_kind);
        }
        false
    }

    /// Checks if any of the given token kinds are at nth position
    fn matches_nth_any(&mut self, n: usize, token_kinds: &[TokenKind]) -> bool {
        if let Some(token) = self.peek_nth(n) {
            return token.is_of_any_kinds(token_kinds);
        }
        false
    }

    /// peeks multiple tokens to see if they match the given token types in row.
    #[allow(dead_code)]
    fn matches_in_row(&mut self, token_kinds: &[TokenKind]) -> bool {
        for (n, token_kind) in token_kinds.iter().enumerate() {
            if let Some(peeked_token) = self.peek_nth(n) {
                if &peeked_token.kind == token_kind {
                    continue;
                } else {
                    return false;
                }
            }
        }
        true
    }

    #[allow(dead_code)]
    fn matches_predicate<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(&mut Parser) -> bool,
    {
        if predicate(self) {
            self.advance();
            return true;
        }
        false
    }

    // checks if the following token is of the given type.
    fn is_next_of_kind(&mut self, token_kind: TokenKind) -> bool {
        if let Some(current_token) = self.peek() {
            if token_kind == current_token.kind && token_kind != TokenKind::EndOfFile {
                // TODO: Check if EndOfFile check is necessary
                return true;
            }
        }
        false
    }

    // checks if the following token is of one of the given types.
    fn is_next_of_kinds(&mut self, token_kinds: &[TokenKind]) -> bool {
        for token in token_kinds {
            if self.is_next_of_kind(*token) {
                return true;
            }
        }
        false
    }

    // if next token is of given type, advances. If not, return an error with given message.
    fn try_consume_token(&mut self, token_kind: TokenKind, msg: &str) -> Result<(), ParserError> {
        if self.is_next_of_kind(token_kind) {
            self.advance();
            return Ok(());
        }
        let token = self.peek();
        Err(ParserError::unexpected_token(
            msg,
            token.cloned(),
            token.unwrap().span,
        ))
    }

    /// If next token is of any of given types, advances. If not, return an error with given message.
    fn try_consume_token2(
        &mut self,
        token_kinds: &[TokenKind],
        msg: &str,
    ) -> Result<(), ParserError> {
        if self.is_next_of_kinds(token_kinds) {
            self.advance();
            return Ok(());
        }
        let token = self.peek();
        Err(ParserError::unexpected_token(
            msg,
            token.cloned(),
            token.unwrap().span,
        ))
    }

    /// Prints some debug info about the current state of the parser
    fn parser_state_dbg_info(&mut self) -> String {
        let current = self.unwrap_current_token();
        let line = current.span.start.line;
        let col = current.span.end.column;
        let r#type = current.kind;
        let mut str = String::new();

        str.push_str(&format!("Current: \"{}\", loc: {}:{}", r#type, line, col));

        if let Some(next) = self.peek() {
            let line = next.span.end.line;
            let col = next.span.end.column;
            let r#type = next.kind;
            str.push_str(&format!(" --- Next: \"{}\", loc: {}:{}", r#type, line, col));
        }
        str
    }
}
