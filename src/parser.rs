// The main parser module that combines all concepts

use crate::ast::{Query, SelectStmt, Statement, TableRef, With, CTE};
use crate::error::{Backtrace, ParseError};
use crate::expr::Expr;
use crate::token::{Token, TokenKind};

pub type ParseResult<T> = Result<T, ParseError>;

/// The parser structure with error tracking
pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    pos: usize,
    backtrace: &'a Backtrace,
    input: &'a str, // Original input for error messages
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>], backtrace: &'a Backtrace, input: &'a str) -> Self {
        Parser {
            tokens,
            pos: 0,
            backtrace,
            input,
        }
    }

    /// Current token
    pub fn current(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    /// Advance to next token
    pub fn advance(&mut self) -> &Token<'a> {
        let token = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    /// Expect a specific token kind
    pub fn expect(&mut self, expected: TokenKind) -> ParseResult<&Token<'a>> {
        match self.current() {
            Some(token) if token.kind == expected => Ok(self.advance()),
            Some(token) => {
                self.backtrace.track_error(
                    token.span.start,
                    &format!("{:?}", expected),
                    Some(token.text),
                    self.input,
                );
                Err(self.backtrace.get_error(self.input))
            }
            None => {
                let pos = if self.pos > 0 && !self.tokens.is_empty() {
                    self.tokens[self.pos - 1].span.end
                } else {
                    0
                };
                self.backtrace
                    .track_error(pos, &format!("{:?}", expected), None, self.input);
                Err(self.backtrace.get_error(self.input))
            }
        }
    }

    /// Try to consume a token
    pub fn try_consume(&mut self, kind: TokenKind) -> bool {
        if self.current().map(|t| t.kind) == Some(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Parse identifier
    pub fn parse_identifier(&mut self) -> ParseResult<&'a str> {
        match self.current() {
            Some(token) if token.kind == TokenKind::Identifier => Ok(self.advance().text),
            Some(token) => {
                self.backtrace.track_error(
                    token.span.start,
                    "identifier",
                    Some(token.text),
                    self.input,
                );
                Err(self.backtrace.get_error(self.input))
            }
            None => {
                let pos = if self.pos > 0 && !self.tokens.is_empty() {
                    self.tokens[self.pos - 1].span.end
                } else {
                    0
                };
                self.backtrace
                    .track_error(pos, "identifier", None, self.input);
                Err(self.backtrace.get_error(self.input))
            }
        }
    }

    /// Create error at current position
    pub fn error_at_current(&self, msg: &str) -> ParseError {
        let mut error = self.backtrace.get_error(self.input);
        error.message = msg.to_string();
        error
    }

    /// Check if current token might be a typo for the expected keyword
    fn check_for_keyword_typo(
        &mut self,
        expected_keyword: &str,
        starts_with_chars: &[char],
    ) -> bool {
        if let Some(token) = self.current() {
            if token.kind == TokenKind::Identifier {
                let text_upper = token.text.to_uppercase();
                for &ch in starts_with_chars {
                    if text_upper.starts_with(ch) {
                        self.backtrace.track_error(
                            token.span.start,
                            expected_keyword,
                            Some(token.text),
                            self.input,
                        );
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if current token might be a typo for WHERE keyword (with substring check)
    fn check_for_where_typo(&mut self) -> bool {
        if let Some(token) = self.current() {
            if token.kind == TokenKind::Identifier {
                let text_upper = token.text.to_uppercase();
                if text_upper.starts_with('W') || text_upper.contains("HER") {
                    self.backtrace.track_error(
                        token.span.start,
                        "WHERE",
                        Some(token.text),
                        self.input,
                    );
                    return true;
                }
            }
        }
        false
    }

    /// Check if current token is a specific WHERE typo pattern
    fn check_for_specific_where_typos(&mut self) -> bool {
        if let Some(token) = self.current() {
            if token.kind == TokenKind::Identifier {
                let text = token.text.to_uppercase();
                if text.starts_with("WHEER")
                    || text.starts_with("WHER")
                    || text.starts_with("WHRE")
                    || text == "WHEER"
                {
                    self.backtrace.track_error(
                        token.span.start,
                        "WHERE",
                        Some(token.text),
                        self.input,
                    );
                    return true;
                }
            }
        }
        false
    }

    /// Parse a complete SQL statement  
    pub fn parse_statement(&mut self) -> ParseResult<Statement<'a>> {
        let start_pos = self.pos;

        // Try WITH clause first
        if self.current().map(|t| t.kind) == Some(TokenKind::With) {
            match self.parse_with() {
                Ok(with) => {
                    match self.parse_query() {
                        Ok(query) => {
                            return Ok(Statement::Query(Query::With {
                                with,
                                query: Box::new(query),
                            }))
                        }
                        Err(_) => {
                            // Reset position and try other statement types
                            self.pos = start_pos;
                        }
                    }
                }
                Err(_) => {
                    // Reset position and try other statement types
                    self.pos = start_pos;
                }
            }
        }

        // Try SELECT statement with lenient parsing for error tracking
        self.pos = start_pos;
        match self.parse_select() {
            Ok(stmt) => return Ok(Statement::Query(Query::Select(Box::new(stmt)))),
            Err(_) => {
                // This was the furthest we could get
            }
        }

        // Try other statement types and track their errors
        self.pos = start_pos;
        if let Some(token) = self.current() {
            // Track errors for other statement types to show alternatives
            self.backtrace
                .track_error(token.span.start, "INSERT", Some(token.text), self.input);
            self.backtrace
                .track_error(token.span.start, "UPDATE", Some(token.text), self.input);
            self.backtrace
                .track_error(token.span.start, "DELETE", Some(token.text), self.input);
            self.backtrace
                .track_error(token.span.start, "WITH", Some(token.text), self.input);
        }

        // If all fail, return the furthest error
        Err(self.backtrace.get_error(self.input))
    }

    /// Parse SELECT statement with lenient keyword matching
    pub fn parse_select(&mut self) -> ParseResult<SelectStmt<'a>> {
        let mut had_errors = false;

        // Try to parse SELECT, but be lenient about typos
        match self.current() {
            Some(token) if token.kind == TokenKind::Select => {
                self.advance();
            }
            Some(token) if token.kind == TokenKind::Identifier => {
                // Track this as an error
                self.backtrace.track_error(
                    token.span.start,
                    "SELECT",
                    Some(token.text),
                    self.input,
                );
                had_errors = true;

                // Check if this looks like a SELECT typo
                let text = token.text.to_uppercase();
                if text.starts_with("SEL") && text.len() >= 4 {
                    // Could be a SELECT typo, continue to see how far we get
                    self.advance();
                } else {
                    return Err(self.backtrace.get_error(self.input));
                }
            }
            Some(token) => {
                self.backtrace.track_error(
                    token.span.start,
                    "SELECT",
                    Some(token.text),
                    self.input,
                );
                return Err(self.backtrace.get_error(self.input));
            }
            None => {
                let pos = if self.pos > 0 && !self.tokens.is_empty() {
                    self.tokens[self.pos - 1].span.end
                } else {
                    0
                };
                self.backtrace.track_error(pos, "SELECT", None, self.input);
                return Err(self.backtrace.get_error(self.input));
            }
        }

        // Parse projection
        let projection = if self.try_consume(TokenKind::Star) {
            vec![Expr::Star]
        } else {
            self.parse_expr_list()?
        };

        // Parse FROM clause
        let from = if self.try_consume(TokenKind::From) {
            Some(self.parse_table_ref()?)
        } else {
            // Check if there's an identifier that might be a misspelled FROM
            if self.check_for_keyword_typo("FROM", &['F']) {
                return Err(self.backtrace.get_error(self.input));
            }
            None
        };

        // Parse WHERE clause (optional)
        let where_clause = if self.try_consume(TokenKind::Where) {
            Some(self.parse_expr()?)
        } else {
            // Check if there's an identifier that might be a misspelled WHERE
            if self.check_for_where_typo() {
                return Err(self.backtrace.get_error(self.input));
            }
            None
        };

        // If we encountered errors during parsing, return the error
        if had_errors {
            return Err(self.backtrace.get_error(self.input));
        }

        Ok(SelectStmt {
            projection,
            from,
            where_clause,
        })
    }

    /// Parse table reference
    fn parse_table_ref(&mut self) -> ParseResult<TableRef<'a>> {
        let name = self.parse_identifier()?;

        // Check for alias (but not common SQL keywords that are likely typos)
        let alias = if self.try_consume(TokenKind::As) {
            Some(self.parse_identifier()?)
        } else if let Some(token) = self.current() {
            if token.kind == TokenKind::Identifier {
                // Check if this looks like a WHERE typo, not an alias
                if self.check_for_specific_where_typos() {
                    return Err(self.backtrace.get_error(self.input));
                }
                Some(self.parse_identifier()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(TableRef { name, alias })
    }

    /// Parse comma-separated expression list
    fn parse_expr_list(&mut self) -> ParseResult<Vec<Expr<'a>>> {
        let mut exprs = vec![self.parse_expr()?];

        while self.try_consume(TokenKind::Comma) {
            exprs.push(self.parse_expr()?);
        }

        Ok(exprs)
    }

    /// Parse WITH clause (including CTEs)
    pub fn parse_with(&mut self) -> ParseResult<With<'a>> {
        self.expect(TokenKind::With)?;

        // Check for RECURSIVE
        let recursive = self.try_consume(TokenKind::Recursive);

        // Parse CTEs
        let mut ctes = vec![self.parse_cte()?];

        while self.try_consume(TokenKind::Comma) {
            ctes.push(self.parse_cte()?);
        }

        Ok(With { recursive, ctes })
    }

    /// Parse a single CTE
    fn parse_cte(&mut self) -> ParseResult<CTE<'a>> {
        let name = self.parse_identifier()?;

        // Optional column list
        let columns = if self.current().map(|t| t.kind) == Some(TokenKind::LeftParen) {
            self.advance();
            let cols = self.parse_identifier_list()?;
            self.expect(TokenKind::RightParen)?;
            Some(cols)
        } else {
            None
        };

        self.expect(TokenKind::As)?;
        self.expect(TokenKind::LeftParen)?;
        let query = Box::new(self.parse_query()?);
        self.expect(TokenKind::RightParen)?;

        Ok(CTE {
            name,
            columns,
            query,
        })
    }

    /// Parse identifier list
    fn parse_identifier_list(&mut self) -> ParseResult<Vec<&'a str>> {
        let mut idents = vec![self.parse_identifier()?];

        while self.try_consume(TokenKind::Comma) {
            idents.push(self.parse_identifier()?);
        }

        Ok(idents)
    }

    /// Parse a query (can be SELECT or WITH)
    pub fn parse_query(&mut self) -> ParseResult<Query<'a>> {
        if self.current().map(|t| t.kind) == Some(TokenKind::With) {
            let with = self.parse_with()?;
            let query = Box::new(self.parse_query()?);
            Ok(Query::With { with, query })
        } else {
            let select = self.parse_select()?;

            // Check for UNION
            if self.try_consume(TokenKind::Union) {
                let all = self.try_consume(TokenKind::All);
                let right = Box::new(self.parse_query()?);
                Ok(Query::Union {
                    left: Box::new(Query::Select(Box::new(select))),
                    all,
                    right,
                })
            } else {
                Ok(Query::Select(Box::new(select)))
            }
        }
    }
}

/// Parse SQL from string
/// Note: This demonstrates the concepts, but in production Databend
/// uses an owned string approach to avoid lifetime complexity
pub fn parse_sql(sql: &str) -> Result<(), ParseError> {
    use crate::token::tokenize;

    let tokens = tokenize(sql);
    let backtrace = Backtrace::new();
    let mut parser = Parser::new(&tokens, &backtrace, sql);

    // Parse and validate
    let _stmt = parser.parse_statement()?;

    Ok(())
}

/// Parse SQL and return an owned representation (for testing)
pub fn parse_sql_to_string(sql: &str) -> Result<String, ParseError> {
    use crate::token::tokenize;

    let tokens = tokenize(sql);
    let backtrace = Backtrace::new();
    let mut parser = Parser::new(&tokens, &backtrace, sql);

    let stmt = parser.parse_statement()?;
    Ok(format!("{:?}", stmt))
}
