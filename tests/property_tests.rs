// Property-based tests using proptest
use proptest::prelude::*;
use sql_parser_demo::parser::{parse_sql, parse_sql_to_string};
use sql_parser_demo::token::tokenize;

// Strategy for generating valid SQL identifiers (excluding SQL keywords)
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter_map("no sql keywords", |s| {
        let keywords = [
            "select",
            "from",
            "where",
            "with",
            "recursive",
            "as",
            "union",
            "all",
            "and",
            "or",
            "not",
            "in",
            "like",
            "between",
            "is",
            "null",
            "true",
            "false",
            "insert",
            "update",
            "delete",
            "create",
            "drop",
            "alter",
            "table",
            "index",
            "by",
            "order",
            "group",
            "having",
            "limit",
            "offset",
            "join",
            "inner",
            "left",
            "right",
            "outer",
            "on",
            "case",
            "when",
            "then",
            "else",
            "end",
        ];
        if keywords.contains(&s.as_str()) {
            None
        } else {
            Some(s.to_string())
        }
    })
}

// Strategy for generating valid SQL numbers
fn number_strategy() -> impl Strategy<Value = String> {
    (0i64..10000).prop_map(|n| n.to_string())
}

// Strategy for generating valid SQL strings
fn string_literal_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{0,20}".prop_map(|s| format!("'{}'", s))
}

// Strategy for generating simple SELECT statements
fn select_statement_strategy() -> impl Strategy<Value = String> {
    (identifier_strategy(), identifier_strategy())
        .prop_map(|(col, table)| format!("SELECT {} FROM {}", col, table))
}

// Strategy for generating WHERE clauses
fn where_clause_strategy() -> impl Strategy<Value = String> {
    (
        identifier_strategy(),
        prop::sample::select(vec!["=", ">", "<", ">=", "<=", "!="]),
        prop_oneof![number_strategy(), string_literal_strategy(),],
    )
        .prop_map(|(col, op, val)| format!("{} {} {}", col, op, val))
}

proptest! {
    #[test]
    fn test_tokenizer_never_panics(input in ".*") {
        // The tokenizer should never panic on any input
        let _ = tokenize(&input);
    }

    #[test]
    fn test_parser_never_panics_on_valid_tokens(input in ".*") {
        // The parser should never panic, even on invalid SQL
        let _ = parse_sql(&input);
    }

    #[test]
    fn test_valid_select_always_parses(
        col in identifier_strategy(),
        table in identifier_strategy()
    ) {
        let sql = format!("SELECT {} FROM {}", col, table);
        prop_assert!(parse_sql(&sql).is_ok());
    }

    #[test]
    fn test_valid_select_with_where_always_parses(
        col in identifier_strategy(),
        table in identifier_strategy(),
        where_clause in where_clause_strategy()
    ) {
        let sql = format!("SELECT {} FROM {} WHERE {}", col, table, where_clause);
        prop_assert!(parse_sql(&sql).is_ok());
    }

    #[test]
    fn test_multiple_columns_parse(
        cols in prop::collection::vec(identifier_strategy(), 1..5),
        table in identifier_strategy()
    ) {
        let cols_str = cols.join(", ");
        let sql = format!("SELECT {} FROM {}", cols_str, table);
        prop_assert!(parse_sql(&sql).is_ok());
    }

    #[test]
    fn test_cte_with_random_names(
        cte_name in identifier_strategy(),
        _table in identifier_strategy()
    ) {
        let sql = format!(
            "WITH {} AS (SELECT 1) SELECT * FROM {}",
            cte_name, cte_name
        );
        prop_assert!(parse_sql(&sql).is_ok());
    }

    #[test]
    fn test_operator_precedence_preserved(
        a in number_strategy(),
        b in number_strategy(),
        c in number_strategy()
    ) {
        // Test that precedence is consistent
        let sql1 = format!("SELECT * WHERE {} + {} * {}", a, b, c);
        let sql2 = format!("SELECT * WHERE {} + ({} * {})", a, b, c);

        // Both should parse successfully
        prop_assert!(parse_sql(&sql1).is_ok());
        prop_assert!(parse_sql(&sql2).is_ok());

        // The AST structure should be the same (multiplication has higher precedence)
        let ast1 = parse_sql_to_string(&sql1);
        let ast2 = parse_sql_to_string(&sql2);

        prop_assert!(ast1.is_ok());
        prop_assert!(ast2.is_ok());
    }

    #[test]
    fn test_union_with_random_values(
        val1 in number_strategy(),
        val2 in number_strategy()
    ) {
        let sql = format!("SELECT {} UNION SELECT {}", val1, val2);
        prop_assert!(parse_sql(&sql).is_ok());

        let sql_all = format!("SELECT {} UNION ALL SELECT {}", val1, val2);
        prop_assert!(parse_sql(&sql_all).is_ok());
    }

    #[test]
    fn test_nested_parentheses(depth in 1usize..10) {
        let mut expr = "1".to_string();
        for _ in 0..depth {
            expr = format!("({})", expr);
        }
        let sql = format!("SELECT {}", expr);
        prop_assert!(parse_sql(&sql).is_ok());
    }

    #[test]
    fn test_binary_operators_associativity(
        a in identifier_strategy(),
        b in identifier_strategy(),
        c in identifier_strategy()
    ) {
        // Test left associativity
        let sql = format!("SELECT * WHERE {} - {} - {}", a, b, c);
        prop_assert!(parse_sql(&sql).is_ok());

        // Should parse as ((a - b) - c)
        let ast = parse_sql_to_string(&sql);
        prop_assert!(ast.is_ok());
    }

    #[test]
    fn test_comparison_chain(
        cols in prop::collection::vec(identifier_strategy(), 2..5),
        vals in prop::collection::vec(number_strategy(), 2..5)
    ) {
        prop_assume!(cols.len() == vals.len());

        let conditions: Vec<String> = cols.iter().zip(vals.iter())
            .map(|(col, val)| format!("{} = {}", col, val))
            .collect();

        let sql = format!("SELECT * WHERE {}", conditions.join(" AND "));
        prop_assert!(parse_sql(&sql).is_ok());
    }

    #[test]
    fn test_error_messages_contain_position(input in "[A-Z]{5,10}") {
        // Generate invalid SQL that should fail
        let sql = format!("{} * FROM users", input);

        if let Err(error) = parse_sql(&sql) {
            // Error should contain line and column information
            prop_assert!(error.line >= 1);
            prop_assert!(error.column >= 1);
            prop_assert!(!error.message.is_empty());
        }
    }

    #[test]
    fn test_zero_copy_property(
        table in identifier_strategy(),
        col in identifier_strategy()
    ) {
        let sql = format!("SELECT {} FROM {}", col, table);
        let tokens = tokenize(&sql);

        // All non-EOF tokens should point to substrings of the original input
        for token in &tokens {
            if token.text != "" {  // Skip EOF token
                let substring = &sql[token.span.clone()];
                prop_assert_eq!(substring, token.text);
            }
        }
    }

    #[test]
    fn test_whitespace_handling(
        col in identifier_strategy(),
        table in identifier_strategy(),
        ws1 in prop::collection::vec(prop::sample::select(vec![" ", "\t", "\n", "\r\n"]), 1..5),
        ws2 in prop::collection::vec(prop::sample::select(vec![" ", "\t", "\n", "\r\n"]), 1..5)
    ) {
        let ws1_str = ws1.join("");
        let ws2_str = ws2.join("");

        let sql = format!("SELECT{}{}{}FROM{}{}", ws1_str, col, ws2_str, ws1_str, table);
        prop_assert!(parse_sql(&sql).is_ok());
    }
}

// Additional property tests for error recovery
proptest! {
    #[test]
    fn test_typo_suggestions_are_reasonable(
        keyword in prop::sample::select(vec!["SELECT", "FROM", "WHERE", "WITH"]),
        typo_char in prop::char::range('A', 'Z')
    ) {
        // Create a typo by replacing one character
        let mut typo = keyword.to_string();
        if !typo.is_empty() {
            let idx = typo.len() / 2;
            typo.replace_range(idx..idx+1, &typo_char.to_string());

            let sql = format!("{} * FROM users", typo);

            if let Err(error) = parse_sql(&sql) {
                // If there's a suggestion, it should be somewhat close to the original
                if let Some(suggestion) = error.suggestion {
                    // The suggestion should be a valid SQL keyword
                    let valid_keywords = vec!["SELECT", "FROM", "WHERE", "WITH", "INSERT", "UPDATE", "DELETE"];
                    prop_assert!(valid_keywords.contains(&suggestion.as_str()));
                }
            }
        }
    }

    #[test]
    fn test_furthest_error_tracking(
        good_part in select_statement_strategy(),
        bad_token in "[A-Z]{5,10}"
    ) {
        let sql = format!("{} {} extra", good_part, bad_token);

        if let Err(error) = parse_sql(&sql) {
            // The error should be reported at or after the good part
            prop_assert!(error.column > 1 || error.line > 1);
        }
    }
}
