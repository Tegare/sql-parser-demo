// Chapter 4: The CTE Solution - Parse != Analyze
// This shows how CTEs are just structure, not recursive parsing

use crate::expr::Expr;
use colored::*;
use std::fmt;

/// SQL Statement types
#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Query(Query<'a>),
    // Can add Insert, Update, Delete etc.
}

/// Query types
#[derive(Debug, Clone)]
pub enum Query<'a> {
    Select(Box<SelectStmt<'a>>),
    With {
        with: With<'a>,
        query: Box<Query<'a>>,
    },
    Union {
        left: Box<Query<'a>>,
        all: bool,
        right: Box<Query<'a>>,
    },
}

/// SELECT statement
#[derive(Debug, Clone)]
pub struct SelectStmt<'a> {
    pub projection: Vec<Expr<'a>>,
    pub from: Option<TableRef<'a>>,
    pub where_clause: Option<Expr<'a>>,
}

/// Table reference
#[derive(Debug, Clone)]
pub struct TableRef<'a> {
    pub name: &'a str,
    pub alias: Option<&'a str>,
}

/// WITH clause containing CTEs
#[derive(Debug, Clone)]
pub struct With<'a> {
    pub recursive: bool,
    pub ctes: Vec<CTE<'a>>,
}

/// Common Table Expression (CTE)
/// The key insight: This is just structure, no recursive parsing needed!
#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct CTE<'a> {
    pub name: &'a str,
    pub columns: Option<Vec<&'a str>>,
    pub query: Box<Query<'a>>, // Just a normal query!
}

/// Demonstrate the CTE insight
pub fn demonstrate_cte_insight() {
    println!("\n=== CTE Parser Insight ===\n");

    println!("{}", "The Problem:".red());
    println!("  WITH RECURSIVE t AS (");
    println!("    SELECT 1 as n");
    println!("    UNION ALL");
    println!("    SELECT n + 1 FROM t WHERE n < 10  -- t references itself!");
    println!("  )");

    println!("\n{}", "What We Thought (WRONG):".yellow());
    println!("  \"CTEs need recursive parsing!\"");
    println!("  \"Create phantom tables during parsing!\"");
    println!("  \"Track self-references in parser!\"");
    println!("  Result: 2000+ lines of complex code");

    println!("\n{}", "The Insight:".green().bold());
    println!("  CTEs aren't recursive during PARSING!");
    println!("  They're recursive during EXECUTION!");

    println!("\n{}", "The Simple Solution:".cyan());
    println!("```rust");
    println!("struct CTE<'a> {{");
    println!("    {} bool,      // Just a flag!", "recursive:".yellow());
    println!("    name: &'a str,");
    println!("    query: Box<Query<'a>>,  // Parse normally!");
    println!("}}");
    println!("```");

    println!("\n{}", "Parsing vs Analysis:".green());
    println!("  {}: Just record the structure", "Parser".cyan());
    println!("  {}: Handle the self-references", "Analyzer".yellow());

    println!(
        "\n💡 {} Lifetimes made us realize this!",
        "Key insight:".cyan()
    );
    println!("   How can a query reference something that doesn't exist yet?");
    println!("   Answer: It can't! So it's not a parsing problem.");
}

// Display implementations for pretty printing

impl<'a> fmt::Display for Statement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Query(q) => write!(f, "{}", q),
        }
    }
}

impl<'a> fmt::Display for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Query::Select(s) => write!(f, "{}", s),
            Query::With { with, query } => {
                write!(f, "{} {}", with, query)
            }
            Query::Union { left, all, right } => {
                write!(
                    f,
                    "{} UNION {} {}",
                    left,
                    if *all { "ALL" } else { "" },
                    right
                )
            }
        }
    }
}

impl<'a> fmt::Display for SelectStmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SELECT ")?;

        for (i, expr) in self.projection.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", expr)?;
        }

        if let Some(from) = &self.from {
            write!(f, " FROM {}", from)?;
        }

        if let Some(where_clause) = &self.where_clause {
            write!(f, " WHERE {}", where_clause)?;
        }

        Ok(())
    }
}

impl<'a> fmt::Display for TableRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(alias) = self.alias {
            write!(f, " AS {}", alias)?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for With<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WITH ")?;
        if self.recursive {
            write!(f, "RECURSIVE ")?;
        }

        for (i, cte) in self.ctes.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", cte)?;
        }

        Ok(())
    }
}

impl<'a> fmt::Display for CTE<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;

        if let Some(columns) = &self.columns {
            write!(f, "(")?;
            for (i, col) in columns.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", col)?;
            }
            write!(f, ")")?;
        }

        write!(f, " AS ({})", self.query)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_sql_to_string;

    #[test]
    fn test_simple_cte() {
        let sql = "WITH t AS (SELECT 1) SELECT * FROM t";
        // For testing, we'll parse and check the string representation
        let result = parse_sql_to_string(sql);
        assert!(result.is_ok());
        let ast_str = result.unwrap();
        assert!(ast_str.contains("With"));
        assert!(ast_str.contains("ctes"));
    }

    #[test]
    fn test_recursive_cte() {
        let sql = "WITH RECURSIVE t(n) AS (SELECT 1) SELECT * FROM t";
        let result = parse_sql_to_string(sql);
        assert!(result.is_ok());
        let ast_str = result.unwrap();
        assert!(ast_str.contains("recursive: true"));
    }
}
