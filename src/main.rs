// How Rust's Type System Saved Our SQL Parser
// Complete working demo from the blog post

mod ast;
mod error;
mod expr;
mod parser;
mod token;

use colored::*;
use parser::{parse_sql, parse_sql_to_string};

fn main() {
    println!("{}", "=".repeat(60).bright_blue());
    println!(
        "{}",
        "How Rust's Type System Saved Our SQL Parser"
            .bright_yellow()
            .bold()
    );
    println!("{}", "Complete Demo from the Blog Post".dimmed());
    println!("{}", "=".repeat(60).bright_blue());

    // Chapter 1: Zero-Copy Tokenization
    demo_zero_copy();

    // Chapter 2: Error Recovery
    demo_error_recovery();

    // Chapter 3: Pratt Parser
    demo_pratt_parser();

    // Chapter 4: CTE Insight
    demo_cte_parsing();

    // Final Demo: Complete SQL Parsing
    demo_complete_parsing();

    println!("\n{}", "=".repeat(60).bright_blue());
    println!("{}", "Key Lessons:".bright_green().bold());
    println!("  1. {} forced zero-copy architecture", "Lifetimes".cyan());
    println!("  2. {} enabled error tracking", "RefCell".cyan());
    println!("  3. {} made precedence elegant", "Pratt parser".cyan());
    println!("  4. {} revealed CTE truth", "Type system".cyan());
    println!("{}", "=".repeat(60).bright_blue());
}

fn demo_zero_copy() {
    println!(
        "\n{}",
        "Chapter 1: Zero-Copy Tokenization".bright_green().bold()
    );
    println!("{}", "-".repeat(40).dimmed());

    let sql = "SELECT name, age FROM users WHERE age > 18";
    token::show_memory_usage(sql);
}

fn demo_error_recovery() {
    println!(
        "\n{}",
        "Chapter 2: Error Recovery with RefCell"
            .bright_green()
            .bold()
    );
    println!("{}", "-".repeat(40).dimmed());

    error::demonstrate_error_tracking();

    // Show real error
    println!("\n{}", "Real Error Example:".yellow());
    let bad_sql = "SELCT * FORM users WHEER age > 18";
    println!("Input: {}", bad_sql.red());

    match parse_sql(bad_sql) {
        Ok(_) => println!("Parsed successfully"),
        Err(e) => println!("\n{}", e),
    }
}

fn demo_pratt_parser() {
    println!("\n{}", "Chapter 3: The Pratt Parser".bright_green().bold());
    println!("{}", "-".repeat(40).dimmed());

    expr::demonstrate_pratt_parser();

    // Show real parsing
    println!("\n{}", "Real Expression Parsing:".yellow());
    let expr_sql = "SELECT * FROM users WHERE age > 18 AND status = 'active' OR admin = 1";

    match parse_sql_to_string(expr_sql) {
        Ok(ast_str) => {
            println!("Input:  {}", expr_sql);
            println!("Parsed: {}", ast_str.green());
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn demo_cte_parsing() {
    println!(
        "\n{}",
        "Chapter 4: CTE Parsing Insight".bright_green().bold()
    );
    println!("{}", "-".repeat(40).dimmed());

    ast::demonstrate_cte_insight();

    // Show real CTE parsing
    println!("\n{}", "Real CTE Parsing:".yellow());

    let cte_examples = vec![
        "WITH t AS (SELECT 1) SELECT * FROM t",
        "WITH RECURSIVE fact(n, f) AS (SELECT 1, 1) SELECT * FROM fact",
    ];

    for sql in cte_examples {
        println!("\nInput: {}", sql.cyan());
        match parse_sql(sql) {
            Ok(_) => println!("Parsed: {}", "✓ Success".green()),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn demo_complete_parsing() {
    println!(
        "\n{}",
        "Final Demo: Complete SQL Parsing".bright_green().bold()
    );
    println!("{}", "-".repeat(40).dimmed());

    let examples = vec![
        ("Simple SELECT", "SELECT * FROM users"),
        ("With WHERE", "SELECT name, age FROM users WHERE age > 18"),
        (
            "Complex expression",
            "SELECT * FROM orders WHERE total > 100 AND status = 'pending' OR priority = 1",
        ),
        (
            "With CTE",
            "WITH recent AS (SELECT * FROM orders WHERE date > '2024-01-01') SELECT * FROM recent",
        ),
        (
            "UNION query",
            "SELECT name FROM users UNION ALL SELECT name FROM customers",
        ),
    ];

    for (desc, sql) in examples {
        println!("\n{}: {}", desc.yellow(), sql);

        match parse_sql_to_string(sql) {
            Ok(ast_str) => {
                println!("  ✓ {}", "Parsed successfully!".green());
                println!("  AST: {}", ast_str.dimmed());
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    }

    // Show memory efficiency
    println!("\n{}", "Memory Efficiency:".yellow());
    let large_sql =
        "SELECT a, b, c, d, e, f, g FROM very_long_table_name WHERE complex_condition = 'value'";
    println!("SQL length: {} bytes", large_sql.len());

    let tokens = token::tokenize(large_sql);
    let token_size = std::mem::size_of::<token::Token>() * tokens.len();
    println!("Token metadata: {} bytes", token_size);
    println!("Ratio: {:.2}x", token_size as f64 / large_sql.len() as f64);
    println!(
        "  {} All strings are references to original input!",
        "→".green()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select() {
        let sql = "SELECT * FROM users";
        assert!(parse_sql(sql).is_ok());
    }

    #[test]
    fn test_parse_where() {
        let sql = "SELECT * FROM users WHERE age > 18";
        assert!(parse_sql(sql).is_ok());
    }

    #[test]
    fn test_parse_cte() {
        let sql = "WITH t AS (SELECT 1) SELECT * FROM t";
        assert!(parse_sql(sql).is_ok());
    }

    #[test]
    fn test_error_recovery() {
        let sql = "SELCT * FROM users";
        let result = parse_sql(sql);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.suggestion.is_some());
            assert_eq!(e.suggestion.unwrap(), "SELECT");
        }
    }

    #[test]
    fn test_parse_to_string() {
        let sql = "SELECT * FROM users";
        let result = parse_sql_to_string(sql);
        assert!(result.is_ok());
        let ast_str = result.unwrap();
        assert!(ast_str.contains("Query"));
        assert!(ast_str.contains("Select"));
    }
}
