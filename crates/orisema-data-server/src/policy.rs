use orisema_data_core::driver::DriverError;
use sqlparser::{
    ast::Statement,
    dialect::{GenericDialect, MsSqlDialect, MySqlDialect, PostgreSqlDialect},
    parser::Parser,
};

/// Narrow cross-dialect allowlist, not a SQL security sandbox. Least-privilege target accounts remain essential.
pub fn validate(sql: &str, read_only: bool) -> Result<bool, DriverError> {
    if sql.trim().is_empty() || sql.len() > 100_000 {
        return Err(DriverError::Policy);
    }
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, sql)
        .or_else(|_| Parser::parse_sql(&MySqlDialect {}, sql))
        .or_else(|_| Parser::parse_sql(&MsSqlDialect {}, sql))
        .or_else(|_| Parser::parse_sql(&GenericDialect {}, sql))
        .map_err(|_| DriverError::Policy)?;
    if statements.len() != 1 {
        return Err(DriverError::Policy);
    }
    match &statements[0] {
        Statement::Query(_) => Ok(true),
        Statement::Insert(_) | Statement::Update { .. } | Statement::Delete(_) if !read_only => {
            Ok(false)
        }
        _ => Err(DriverError::Policy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_control_and_multiple_statements() {
        for sql in [
            "",
            "SELECT 1; DELETE FROM t",
            "COMMIT",
            "SET transaction_read_only=off",
            "DROP TABLE t",
            "COPY t TO PROGRAM 'id'",
            "UPDATE t SET a=1",
        ] {
            assert!(validate(sql, true).is_err(), "{sql}");
        }
        assert!(validate("SELECT ';' /* ; */", true).is_ok());
        assert!(validate("WITH x AS (SELECT 1) SELECT * FROM x", true).is_ok());
        assert!(validate("UPDATE t SET a=1", false).is_ok());
        assert!(validate("SELECT * FROM `items` LIMIT 10", true).is_ok());
        assert!(validate("SELECT TOP (10) * FROM [items]", true).is_ok());
    }
}
