// Chapter 3: The Pratt Parser - Precedence as Data
// This shows how to handle operator precedence elegantly

use crate::parser::{ParseResult, Parser};
use crate::token::TokenKind;
use colored::*;

/// Expression AST with zero-copy strings
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    /// Column reference
    Column(&'a str),

    /// Literal value
    Literal(Literal<'a>),

    /// Binary operation
    Binary {
        left: Box<Expr<'a>>,
        op: BinaryOp,
        right: Box<Expr<'a>>,
    },

    /// Parenthesized expression
    Paren(Box<Expr<'a>>),

    /// Star (for SELECT *)
    Star,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'a> {
    Number(i64),
    Float(f64),
    String(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Logical
    And,
    Or,

    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // Arithmetic
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl BinaryOp {
    /// Convert token to binary operator
    pub fn from_token(token: TokenKind) -> Option<Self> {
        match token {
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Or => Some(BinaryOp::Or),
            TokenKind::Equal => Some(BinaryOp::Equal),
            TokenKind::NotEqual => Some(BinaryOp::NotEqual),
            TokenKind::Less => Some(BinaryOp::Less),
            TokenKind::Greater => Some(BinaryOp::Greater),
            TokenKind::LessEqual => Some(BinaryOp::LessEqual),
            TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
            TokenKind::Plus => Some(BinaryOp::Plus),
            TokenKind::Minus => Some(BinaryOp::Minus),
            TokenKind::Star => Some(BinaryOp::Multiply),
            TokenKind::Slash => Some(BinaryOp::Divide),
            _ => None,
        }
    }
}

/// Get operator precedence
fn get_precedence(token: TokenKind) -> Option<(u8, bool)> {
    // Return (precedence, is_left_associative)
    match token {
        TokenKind::Or => Some((10, true)),
        TokenKind::And => Some((20, true)),
        TokenKind::Equal | TokenKind::NotEqual => Some((30, true)),
        TokenKind::Less | TokenKind::Greater | TokenKind::LessEqual | TokenKind::GreaterEqual => {
            Some((40, true))
        }
        TokenKind::Plus | TokenKind::Minus => Some((50, true)),
        TokenKind::Star | TokenKind::Slash => Some((60, true)),
        _ => None,
    }
}

impl<'a> Parser<'a> {
    /// Parse an expression using precedence climbing
    pub fn parse_expr(&mut self) -> ParseResult<Expr<'a>> {
        self.parse_expr_with_precedence(0)
    }

    /// Parse expression with minimum precedence
    fn parse_expr_with_precedence(&mut self, min_prec: u8) -> ParseResult<Expr<'a>> {
        let mut left = self.parse_primary()?;

        while let Some(token) = self.current() {
            if let Some((prec, is_left)) = get_precedence(token.kind) {
                if prec < min_prec {
                    break;
                }

                let op_kind = token.kind;
                self.advance();
                let next_min_prec = if is_left { prec + 1 } else { prec };
                let right = self.parse_expr_with_precedence(next_min_prec)?;

                if let Some(op) = BinaryOp::from_token(op_kind) {
                    left = Expr::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    };
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse primary expression
    fn parse_primary(&mut self) -> ParseResult<Expr<'a>> {
        match self.current() {
            Some(token) => {
                match token.kind {
                    TokenKind::Number => {
                        let text = token.text;
                        self.advance();
                        let n = text
                            .parse::<i64>()
                            .map_err(|_| self.error_at_current("Invalid number"))?;
                        Ok(Expr::Literal(Literal::Number(n)))
                    }
                    TokenKind::Float => {
                        let text = token.text;
                        self.advance();
                        let f = text
                            .parse::<f64>()
                            .map_err(|_| self.error_at_current("Invalid float"))?;
                        Ok(Expr::Literal(Literal::Float(f)))
                    }
                    TokenKind::String => {
                        let text = token.text;
                        self.advance();
                        // Remove quotes
                        let s = &text[1..text.len() - 1];
                        Ok(Expr::Literal(Literal::String(s)))
                    }
                    TokenKind::Identifier => {
                        let text = token.text;
                        self.advance();
                        Ok(Expr::Column(text))
                    }
                    TokenKind::Star => {
                        self.advance();
                        Ok(Expr::Star)
                    }
                    TokenKind::LeftParen => {
                        self.advance();
                        let expr = self.parse_expr()?;
                        self.expect(TokenKind::RightParen)?;
                        Ok(Expr::Paren(Box::new(expr)))
                    }
                    _ => Err(self.error_at_current("Expected expression")),
                }
            }
            None => Err(self.error_at_current("Unexpected end of input")),
        }
    }
}

/// Demonstrate the Pratt parser
pub fn demonstrate_pratt_parser() {
    println!("\n=== Pratt Parser Demo ===\n");

    println!("{}", "The Problem:".yellow());
    println!("  Parse: 2 + 3 * 4 - 5 / 2 AND true OR false");
    println!("  Need:  ((2 + (3 * 4)) - (5 / 2)) AND true) OR false");

    println!("\n{}", "The Traditional Solution:".yellow());
    println!("  15+ functions, one for each precedence level:");
    println!("  parse_or() -> parse_and() -> parse_compare() -> ...");

    println!("\n{}", "The Pratt Solution:".green().bold());
    println!("  Precedence as data:");

    let precedences = vec![
        ("OR", 10, "Lowest precedence"),
        ("AND", 20, ""),
        ("=, !=", 30, ""),
        ("<, >", 40, ""),
        ("+, -", 50, ""),
        ("*, /", 60, "Highest precedence"),
    ];

    for (ops, prec, note) in precedences {
        println!(
            "    {} => Precedence({}) {}",
            ops.cyan(),
            prec.to_string().yellow(),
            note.dimmed()
        );
    }

    println!("\n{}", "Benefits:".green());
    println!("  ✓ Adding operators = adding data, not functions");
    println!("  ✓ Clear precedence table");
    println!("  ✓ One algorithm handles everything");

    // Show example parsing
    println!("\n{}", "Example Parse:".cyan());
    let input = "age > 18 AND status = 'active' OR admin = true";
    println!("  Input:  {}", input);
    println!("  Result: ((age > 18 AND status = 'active') OR admin = true)");

    println!(
        "\n💡 {} Precedence is data, not code structure!",
        "Key insight:".cyan()
    );
}

impl<'a> std::fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Column(name) => write!(f, "{}", name),
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Binary { left, op, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::Paren(expr) => write!(f, "({})", expr),
            Expr::Star => write!(f, "*"),
        }
    }
}

impl<'a> std::fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::String(s) => write!(f, "'{}'", s),
        }
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOp::And => "AND",
            BinaryOp::Or => "OR",
            BinaryOp::Equal => "=",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEqual => "<=",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Backtrace;
    use crate::token::tokenize;

    #[test]
    fn test_precedence() {
        let input = "2 + 3 * 4";
        let tokens = tokenize(input);
        let backtrace = Backtrace::new();
        let mut parser = Parser::new(&tokens, &backtrace, input);

        let expr = parser.parse_expr().unwrap();

        // Should parse as (2 + (3 * 4))
        assert_eq!(format!("{}", expr), "(2 + (3 * 4))");
    }

    #[test]
    fn test_complex_expression() {
        let input = "age > 18 AND status = 'active' OR admin = 1";
        let tokens = tokenize(input);
        let backtrace = Backtrace::new();
        let mut parser = Parser::new(&tokens, &backtrace, input);

        let expr = parser.parse_expr().unwrap();

        // Should parse with correct precedence
        let expected = "(((age > 18) AND (status = 'active')) OR (admin = 1))";
        assert_eq!(format!("{}", expr), expected);
    }
}
