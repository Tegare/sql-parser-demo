# SQL Parser Demo

A working implementation of SQL parser concepts demonstrating error tracking and zero-copy parsing techniques. 

This demo accompanies the blog post: [Why We Built Our Own SQL Parser From Scratch: A Rust Implementation Story](https://www.databend.com/blog/category-engineering/2025-09-10-query-parser/)

## Error Tracking with RefCell

The parser tracks the furthest error position during backtracking and provides helpful suggestions:

```rust
// Input: "SELCT * FROM users"
// Error at position 0: Expected SELECT, found 'SELCT'
// Did you mean: SELECT

// How it works:
pub struct Backtrace {
    inner: RefCell<Option<BacktraceInner>>,
}

impl Backtrace {
    pub fn track_error(&self, pos: usize, expected: &str, found: Option<&str>, input: &str) {
        // Through immutable &self, we can still track errors!
        let mut inner = self.inner.borrow_mut();
        if pos > inner.furthest_pos {
            // New furthest error - this is what we'll report
            inner.furthest_pos = pos;
            inner.expected = vec![expected];
        }
    }
}
```

## Tech Stack

```toml
[dependencies]
logos = "0.13"         # Fast tokenization
nom = "7.1"            # Parser combinators
strsim = "0.10"        # Error suggestions
```

## Key Demonstrations

### 1. Zero-Copy Tokenization
All tokens are slices of the original input string:
```rust
struct Token<'a> {
    text: &'a str,  // Just a reference!
    kind: TokenKind,
    span: Range<usize>,
}
```

### 2. Error Recovery
Tracks all parse attempts and reports the furthest error:
```rust
// Parser tries: SELECT, INSERT, UPDATE at position 0
// Result: "Expected one of: SELECT, INSERT, UPDATE"
```

### 3. Operator Precedence
Precedence as data, not code structure:
```rust
TokenKind::Or => Precedence(10),
TokenKind::And => Precedence(20),
TokenKind::Plus => Precedence(50),
TokenKind::Star => Precedence(60),
```


## Code Structure

- `src/token.rs` - Zero-copy tokenization using logos
- `src/error.rs` - Error tracking with RefCell and suggestions
- `src/expr.rs` - Expression parser with precedence climbing
- `src/ast.rs` - AST including CTE support
- `src/parser.rs` - Main parser implementation

## Note on Production Implementation

The production Databend parser uses additional tools:
- `pratt` crate for expression parsing  
- `recursive` for stack-safe recursion
- Additional optimizations for performance

This demo focuses on the core concepts to keep the code simple and educational.

## Running the Demo

```bash
# Build the project
cargo build

# Run the demo
cargo run

# Run tests
cargo test
```


## Supported SQL

- Basic SELECT statements
- WHERE clauses with complex expressions
- CTEs (WITH and WITH RECURSIVE)
- UNION queries
- Binary operators with correct precedence

## Examples

```sql
-- Simple SELECT
SELECT * FROM users

-- With WHERE clause
SELECT name, age FROM users WHERE age > 18

-- Complex expression
SELECT * FROM orders 
WHERE total > 100 AND status = 'pending' OR priority = 1

-- CTE
WITH recent AS (
    SELECT * FROM orders WHERE date > '2024-01-01'
) 
SELECT * FROM recent

-- Recursive CTE (parsed, not executed)
WITH RECURSIVE fact(n, f) AS (
    SELECT 1, 1
    UNION ALL
    SELECT n + 1, f * (n + 1) FROM fact WHERE n < 10
)
SELECT * FROM fact
```

## Performance Impact

In Databend production:
- CPU usage: 66% → 20%
- Memory: 5x reduction
- Error messages: Actually helpful with suggestions
