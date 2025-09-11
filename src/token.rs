// Chapter 1: Zero-Copy Tokenization with logos
// This shows how Rust's lifetimes enable zero-copy parsing

use logos::Logos;
use std::fmt;
use std::ops::Range;

/// The token structure - notice the lifetime 'a
/// This means the token doesn't own the string, just references it
#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a> {
    pub text: &'a str, // Zero-copy reference to original input!
    pub kind: TokenKind,
    pub span: Range<usize>,
}

impl<'a> Token<'a> {
    pub fn new(text: &'a str, kind: TokenKind, span: Range<usize>) -> Self {
        Token { text, kind, span }
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({})", self.kind, self.text)
    }
}

/// Token types using logos for fast tokenization
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n\f]+")] // Skip whitespace
#[logos(skip r"--[^\n]*")] // Skip SQL comments
pub enum TokenKind {
    // Keywords (case-insensitive)
    #[regex("(?i)SELECT")]
    Select,

    #[regex("(?i)FROM")]
    From,

    #[regex("(?i)WHERE")]
    Where,

    #[regex("(?i)WITH")]
    With,

    #[regex("(?i)RECURSIVE")]
    Recursive,

    #[regex("(?i)AS")]
    As,

    #[regex("(?i)UNION")]
    Union,

    #[regex("(?i)ALL")]
    All,

    #[regex("(?i)AND")]
    And,

    #[regex("(?i)OR")]
    Or,

    #[regex("(?i)INSERT")]
    Insert,

    #[regex("(?i)UPDATE")]
    Update,

    #[regex("(?i)DELETE")]
    Delete,

    // Identifiers and literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"'([^'\\]|\\.)*'")]
    String,

    #[regex(r"-?[0-9]+")]
    Number,

    #[regex(r"-?[0-9]+\.[0-9]+")]
    Float,

    // Operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("=")]
    Equal,

    #[token("!=")]
    #[token("<>")]
    NotEqual,

    #[token("<")]
    Less,

    #[token(">")]
    Greater,

    #[token("<=")]
    LessEqual,

    #[token(">=")]
    GreaterEqual,

    // Delimiters
    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    // End of input
    Eof,
}

/// Zero-copy tokenization function
/// The key insight: we return tokens that reference the original input
/// No string copying happens here!
pub fn tokenize(input: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut lexer = TokenKind::lexer(input);

    while let Some(result) = lexer.next() {
        if let Ok(kind) = result {
            let span = lexer.span();
            let text = &input[span.clone()];
            tokens.push(Token::new(text, kind, span));
        }
    }

    // Add EOF token
    let len = input.len();
    tokens.push(Token::new("", TokenKind::Eof, len..len));

    tokens
}

/// Demonstrate memory efficiency
pub fn show_memory_usage(sql: &str) {
    println!("\n=== Zero-Copy Memory Demo ===");
    println!("Input SQL: {} bytes", sql.len());

    let tokens = tokenize(sql);

    // Calculate metadata size (not the strings!)
    let metadata_size = tokens.len() * std::mem::size_of::<Token>();
    println!("Token count: {}", tokens.len());
    println!("Token metadata size: {} bytes", metadata_size);

    // The key point: tokens don't own strings
    println!("\nToken text pointers:");
    for (i, token) in tokens.iter().take(3).enumerate() {
        let ptr = token.text.as_ptr() as usize;
        let input_ptr = sql.as_ptr() as usize;
        if ptr >= input_ptr && ptr < input_ptr + sql.len() {
            println!("  Token {}: Points into original input ✓", i);
        }
    }

    println!("\n💡 Key insight: All token strings are slices of the original input!");
    println!("   No string copying = massive memory savings");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy() {
        let sql = "SELECT * FROM users";
        let tokens = tokenize(sql);

        // Verify tokens point into original string
        for token in &tokens {
            if !token.text.is_empty() {
                let token_ptr = token.text.as_ptr();
                let sql_start = sql.as_ptr();
                let sql_end = unsafe { sql_start.add(sql.len()) };

                assert!(
                    token_ptr >= sql_start && token_ptr < sql_end,
                    "Token should reference original input"
                );
            }
        }
    }

    #[test]
    fn test_tokenize_basic() {
        let sql = "SELECT name FROM users WHERE age > 18";
        let tokens = tokenize(sql);

        assert_eq!(tokens[0].kind, TokenKind::Select);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].text, "name");
        assert_eq!(tokens[2].kind, TokenKind::From);
    }
}
