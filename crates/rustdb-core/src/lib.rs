// SPDX-License-Identifier: AGPL-3.0-only
use serde::{Deserialize, Serialize};

/// Transport-neutral operations reserved for a future agent protocol.
#[derive(Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum Operation {
    TestConnection,
    ListSchemas,
    ListTables { schema: String },
    ListColumns { schema: String, table: String },
    ExecuteQuery { sql: String },
    CancelQuery { query_id: uuid::Uuid },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_contract() -> Result<(), serde_json::Error> {
        let operation = Operation::ExecuteQuery {
            sql: "SELECT 1".into(),
        };
        let value = serde_json::to_value(operation)?;
        assert_eq!(value["operation"], "execute_query");
        assert!(serde_json::from_value::<Operation>(value).is_ok());
        Ok(())
    }
}
