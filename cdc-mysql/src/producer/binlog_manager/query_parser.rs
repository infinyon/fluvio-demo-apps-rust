use sqlparser::ast::{AlterTableOperation, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
/// query_parser.rs
///
/// The columns must be cashed as they are needed to construct ROW UPDATE operations.
///
/// Parses column information from the following MySQL commands:
///     CREATE TABLE
///     ALTER TABLE
///     DROP TABLE
///
/// Parser return an Enum storing the operations with addition metadata.
///
use std::fmt;

use crate::error::CdcError;

type Column = String;
type Name = String;
type OldName = String;
type NewName = String;

pub enum TableOp {
    CreateTable(Name, Vec<Column>),
    AlterTable(Name, ColumnOp),
    DropTable(Vec<Name>),
}

pub enum ColumnOp {
    Add(Name),
    Rename(OldName, NewName),
    Drop(Name),
}

impl fmt::Display for TableOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TableOp::CreateTable(table, columns) => write!(
                f,
                "Create Table {} - add columns ({})",
                table,
                columns.join(", ")
            ),
            TableOp::AlterTable(table, column_op) => match column_op {
                ColumnOp::Add(name) => write!(f, "Alter Table {} - add column {}", table, name),
                ColumnOp::Rename(old_name, new_name) => write!(
                    f,
                    "Alter Table {} - rename column {} to {}",
                    table, old_name, new_name
                ),
                ColumnOp::Drop(name) => write!(f, "Alter Table {} - remove column {}", table, name),
            },
            TableOp::DropTable(tables) => write!(f, "Drop Tables ({})", tables.join(", ")),
        }
    }
}

pub fn parse_query(query: &Option<String>) -> Result<Vec<TableOp>, CdcError> {
    let mut table_ops = vec![];

    if let Some(query) = query {
        // skip database operations
        if query.to_lowercase().contains("database") {
            return Ok(table_ops);
        }

        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, query)
            .map_err(|source| CdcError::SqlParserError { source })?;

        for statement in ast {
            match statement {
                Statement::CreateTable { name, columns, .. } => {
                    table_ops.push(TableOp::CreateTable(
                        name.to_string(),
                        columns
                            .iter()
                            .map(|x| x.name.to_string())
                            .collect::<Vec<String>>(),
                    ))
                }
                Statement::AlterTable { name, operation } => match operation {
                    AlterTableOperation::AddColumn { column_def } => {
                        table_ops.push(TableOp::AlterTable(
                            name.to_string(),
                            ColumnOp::Add(column_def.name.to_string()),
                        ))
                    }
                    AlterTableOperation::RenameColumn {
                        old_column_name,
                        new_column_name,
                    } => table_ops.push(TableOp::AlterTable(
                        name.to_string(),
                        ColumnOp::Rename(old_column_name.to_string(), new_column_name.to_string()),
                    )),
                    AlterTableOperation::DropColumn { column_name, .. } => {
                        table_ops.push(TableOp::AlterTable(
                            name.to_string(),
                            ColumnOp::Drop(column_name.to_string()),
                        ))
                    }
                    _ => {}
                },
                Statement::Drop {
                    object_type, names, ..
                } => {
                    if object_type.to_string() == "TABLE" {
                        table_ops.push(TableOp::DropTable(
                            names.iter().map(|x| x.to_string()).collect::<Vec<String>>(),
                        ))
                    }
                }
                _ => {}
            }
        }
    }

    Ok(table_ops)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_query_create_table() {
        let query =
            "CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE)";
        let ops_result = parse_query(&Some(query.to_string()));
        let ops = ops_result.unwrap();
        assert_eq!(ops.len(), 1);

        let result = &ops[0];
        let expected = "Create Table pet - add columns (name, owner, species, sex, birth)";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn test_parse_query_alter_table_add_column() {
        let query = "ALTER TABLE pet ADD hello DATE";
        let ops_result = parse_query(&Some(query.to_string()));
        let ops = ops_result.unwrap();
        assert_eq!(ops.len(), 1);

        let result = &ops[0];
        let expected = "Alter Table pet - add column hello";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn test_parse_query_alter_table_add_column_with_name() {
        let query = "ALTER TABLE pet ADD COLUMN hello DATE";
        let ops_result = parse_query(&Some(query.to_string()));
        let ops = ops_result.unwrap();
        assert_eq!(ops.len(), 1);

        let result = &ops[0];
        let expected = "Alter Table pet - add column hello";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn test_parse_query_alter_table_rename_column() {
        let query = "ALTER TABLE pet RENAME COLUMN hello TO bye";
        let ops_result = parse_query(&Some(query.to_string()));
        let ops = ops_result.unwrap();
        assert_eq!(ops.len(), 1);

        let result = &ops[0];
        let expected = "Alter Table pet - rename column hello to bye";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn test_parse_query_alter_table_drop_column() {
        let query = "ALTER TABLE pet DROP COLUMN hello";
        let ops_result = parse_query(&Some(query.to_string()));
        let ops = ops_result.unwrap();
        assert_eq!(ops.len(), 1);

        let result = &ops[0];
        let expected = "Alter Table pet - remove column hello";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn test_parse_query_drop_tables() {
        let query = "DROP TABLE pet";
        let ops_result = parse_query(&Some(query.to_string()));
        let ops = ops_result.unwrap();
        assert_eq!(ops.len(), 1);

        let result = &ops[0];
        let expected = "Drop Tables (pet)";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn test_other_parse_query_ops() {
        // test - none
        let ops_result = parse_query(&None);
        assert_eq!(ops_result.unwrap().len(), 0);

        // test - "BEGIN"
        let query = "BEGIN";
        let ops_result = parse_query(&Some(query.to_string()));
        assert_eq!(ops_result.unwrap().len(), 0);

        // test - "create database flvTest"
        let query = "create database flvTest";
        let ops_result = parse_query(&Some(query.to_string()));
        assert_eq!(ops_result.unwrap().len(), 0);

        // test - "alter table people add col1 int"
        let query = "alter table people add col1 int";
        let ops_result = parse_query(&Some(query.to_string()));
        let result = &ops_result.unwrap()[0];
        let expected = "Alter Table people - add column col1";
        assert_eq!(result.to_string(), expected);

        // test - "CREATE TABLE species (name VARCHAR(20), type VARCHAR(20),  age SMALLINT)"
        let query = "CREATE TABLE species (name VARCHAR(20), type VARCHAR(20),  age SMALLINT)";
        let ops_result = parse_query(&Some(query.to_string()));
        let result = &ops_result.unwrap()[0];
        let expected = "Create Table species - add columns (name, type, age)";
        assert_eq!(result.to_string(), expected);

        // test - "CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE)"
        let query = "CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE)";
        let ops_result = parse_query(&Some(query.to_string()));
        let result = &ops_result.unwrap()[0];
        let expected = "Create Table pet - add columns (name, owner, species, sex, birth)";
        assert_eq!(result.to_string(), expected);

        // test - "DROP TABLE `species` /* generated by server */"
        let query = "DROP TABLE species /* generated by server */";
        let ops_result = parse_query(&Some(query.to_string()));
        let result = &ops_result.unwrap()[0];
        let expected = "Drop Tables (species)";
        assert_eq!(result.to_string(), expected);
    }
}
