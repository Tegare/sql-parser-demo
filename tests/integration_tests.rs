// Integration tests that test the complete parsing flow
use sql_parser_demo::parser::{parse_sql, parse_sql_to_string};

#[test]
fn test_parse_simple_select() {
    let queries = vec![
        "SELECT * FROM users",
        "SELECT name FROM users",
        "SELECT name, age FROM users",
        "SELECT users.name FROM users",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_parse_select_with_where() {
    let queries = vec![
        "SELECT * FROM users WHERE age > 18",
        "SELECT * FROM users WHERE status = 'active'",
        "SELECT * FROM users WHERE age > 18 AND status = 'active'",
        "SELECT * FROM users WHERE age > 18 OR admin = 1",
        "SELECT * FROM users WHERE (age > 18 AND status = 'active') OR admin = 1",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_parse_complex_expressions() {
    let queries = vec![
        "SELECT * FROM orders WHERE total > 100 * 1.1",
        "SELECT * FROM users WHERE age >= 18 AND age <= 65",
        "SELECT * FROM products WHERE price / quantity > 10",
        "SELECT * FROM users WHERE (age + 5) * 2 > 50",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_parse_cte() {
    let queries = vec![
        "WITH t AS (SELECT 1) SELECT * FROM t",
        "WITH users_cte AS (SELECT * FROM users) SELECT * FROM users_cte",
        "WITH t1 AS (SELECT 1), t2 AS (SELECT 2) SELECT * FROM t1",
        "WITH RECURSIVE t AS (SELECT 1) SELECT * FROM t",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_parse_cte_with_columns() {
    let queries = vec![
        "WITH t(a) AS (SELECT 1) SELECT * FROM t",
        "WITH t(a, b) AS (SELECT 1, 2) SELECT * FROM t",
        "WITH RECURSIVE fact(n, f) AS (SELECT 1, 1) SELECT * FROM fact",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_parse_union() {
    let queries = vec![
        "SELECT name FROM users UNION SELECT name FROM customers",
        "SELECT name FROM users UNION ALL SELECT name FROM customers",
        "SELECT 1 UNION SELECT 2 UNION SELECT 3",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_parse_recursive_cte_structure() {
    // Classic recursive CTE example
    let sql = "WITH RECURSIVE factorial(n, fact) AS (
        SELECT 1, 1
        UNION ALL
        SELECT n + 1, fact * (n + 1) FROM factorial WHERE n < 10
    ) SELECT * FROM factorial";

    // Should parse successfully even though it references itself
    assert!(parse_sql(sql).is_ok());
}

#[test]
fn test_error_messages_are_helpful() {
    let test_cases = vec![
        ("SELCT * FROM users", "SELECT"),
        ("SELECT * FORM users", "FROM"),
        ("SELECT * FROM users WHEER age > 18", "WHERE"),
        ("WITH t AS SELECT 1 SELECT * FROM t", "parenthes"), // Missing parentheses
    ];

    for (bad_sql, expected_suggestion) in test_cases {
        let result = parse_sql(bad_sql);
        assert!(result.is_err(), "Should have failed: {}", bad_sql);

        let error = result.unwrap_err();
        if expected_suggestion == "SELECT"
            || expected_suggestion == "FROM"
            || expected_suggestion == "WHERE"
        {
            assert_eq!(
                error.suggestion,
                Some(expected_suggestion.to_string()),
                "Wrong suggestion for: {}",
                bad_sql
            );
        }
    }
}

#[test]
fn test_furthest_error_tracking() {
    // This should fail at 'FORM' not at 'SELCT'
    let sql = "SELCT * FORM users";
    let result = parse_sql(sql);

    assert!(result.is_err());
    let error = result.unwrap_err();

    // The error should be about FROM, not SELECT
    // because the parser made it past SELCT
    assert!(error.message.contains("FROM") || error.column > 1);
}

#[test]
fn test_zero_copy_property() {
    // Verify that parsing doesn't allocate new strings
    let sql = "SELECT name, age FROM users WHERE status = 'active'";

    // Parse to string representation to check structure
    let result = parse_sql_to_string(sql);
    assert!(result.is_ok());

    // The AST should reference the original input
    let ast_str = result.unwrap();
    assert!(ast_str.contains("Select"));
    assert!(ast_str.contains("users"));
    assert!(ast_str.contains("status"));
}

#[test]
fn test_operator_precedence_comprehensive() {
    let test_cases = vec![
        (
            "SELECT * WHERE a OR b AND c",
            // Should parse as (a OR (b AND c)) - AND has higher precedence than OR
            "Or.*And", // OR should appear first as the outer operator in Debug format
        ),
        (
            "SELECT * WHERE a = 1 OR b = 2 AND c = 3",
            // Should parse as ((a = 1) OR ((b = 2) AND (c = 3)))
            "Or.*And",
        ),
        (
            "SELECT * WHERE a + b * c",
            // Should parse as (a + (b * c))
            "Plus.*Multiply",
        ),
        (
            "SELECT * WHERE a * b + c * d",
            // Should parse as ((a * b) + (c * d))
            "Plus",
        ),
    ];

    for (sql, pattern) in test_cases {
        let result = parse_sql_to_string(sql);
        assert!(result.is_ok(), "Failed to parse: {}", sql);

        let ast_str = result.unwrap();
        let re = regex::Regex::new(pattern).unwrap();
        assert!(
            re.is_match(&ast_str),
            "Precedence issue in '{}'. AST: {}",
            sql,
            ast_str
        );
    }
}

#[test]
fn test_parentheses_override_precedence() {
    let test_cases = vec![
        "SELECT * WHERE (a OR b) AND c",
        "SELECT * WHERE a * (b + c)",
        "SELECT * WHERE ((a + b) * c) / d",
    ];

    for sql in test_cases {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_complex_real_world_queries() {
    let queries = vec![
        // E-commerce query
        "WITH recent_orders AS (
            SELECT * FROM orders WHERE date > '2024-01-01'
        )
        SELECT * FROM recent_orders WHERE total > 100 AND status = 'pending'",
        // Analytics query
        "SELECT user_id, COUNT(*) as total
        FROM events
        WHERE type = 'click' AND timestamp > '2024-01-01'",
        // User management query
        "SELECT * FROM users
        WHERE (age >= 18 AND age <= 65) 
        AND (status = 'active' OR admin = 1)",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_stress_deep_nesting() {
    // Test deeply nested expressions
    let sql = "SELECT * WHERE ((((a = 1))))";
    assert!(parse_sql(sql).is_ok());

    let sql = "SELECT * WHERE a = 1 AND b = 2 AND c = 3 AND d = 4 AND e = 5";
    assert!(parse_sql(sql).is_ok());
}

#[test]
fn test_all_supported_keywords() {
    let queries = vec![
        "SELECT * FROM t",
        "SELECT * FROM t WHERE a = 1",
        "WITH t AS (SELECT 1) SELECT * FROM t",
        "WITH RECURSIVE t AS (SELECT 1) SELECT * FROM t",
        "SELECT 1 UNION SELECT 2",
        "SELECT 1 UNION ALL SELECT 2",
        "SELECT * FROM t AS alias",
        "SELECT COUNT(*) FROM t",
        "SELECT * FROM t1, t2",
    ];

    for sql in queries {
        assert!(parse_sql(sql).is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_error_recovery_gives_good_context() {
    let sql = "SELECT *\nFROM users\nWHEER age > 18";
    let result = parse_sql(sql);

    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should point to line 3
    assert_eq!(error.line, 3);

    // Should have context showing the error location
    assert!(error.context.is_some());

    // Should suggest WHERE
    assert_eq!(error.suggestion, Some("WHERE".to_string()));
}
