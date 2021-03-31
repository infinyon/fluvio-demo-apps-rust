use async_std::fs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use tracing::debug;

use super::{ColumnOp, TableOp};
use crate::util::expand_tilde;

type DbName = String;
type TableName = String;
type Column = String;
#[derive(Debug)]
pub struct LocalStore {
    path: PathBuf,
    store: DbStore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbStore {
    dbs: BTreeMap<DbName, TableStore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableStore {
    tables: BTreeMap<TableName, Vec<Column>>,
}

impl Default for DbStore {
    fn default() -> Self {
        Self {
            dbs: BTreeMap::new(),
        }
    }
}

impl Default for TableStore {
    fn default() -> Self {
        Self {
            tables: BTreeMap::new(),
        }
    }
}

impl LocalStore {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self, Error> {
        let path_buf = path.into();
        let path = match expand_tilde(&path_buf) {
            Some(resolved) => resolved,
            None => path_buf,
        };

        let store = load_from_file(&path)?.unwrap_or_default();

        Ok(Self { path, store })
    }

    pub fn update_store(&mut self, db_name: &str, table_ops: Vec<TableOp>) -> Result<(), Error> {
        self.store.update_store(db_name, table_ops)?;
        save_to_file(&self.path, &self.store)?;

        Ok(())
    }

    pub fn get_columns(&mut self, db_name: &str, table_name: &str) -> Result<Vec<String>, Error> {
        self.store.get_columns(db_name, table_name)
    }
}

impl DbStore {
    fn update_store(&mut self, db_name: &str, table_ops: Vec<TableOp>) -> Result<(), Error> {
        for table_op in table_ops {
            match table_op {
                TableOp::CreateTable(table_name, columns) => {
                    self.create_table(db_name, table_name, columns)?;
                }
                TableOp::AlterTable(table_name, column_op) => {
                    self.alter_table(db_name, table_name, column_op);
                }
                TableOp::DropTable(table_names) => {
                    self.drop_tables(db_name, table_names);
                }
            }
        }
        Ok(())
    }

    fn create_table(
        &mut self,
        db_name: &str,
        table_name: String,
        columns: Vec<String>,
    ) -> Result<(), Error> {
        let table_store = match self.dbs.get_mut(db_name) {
            Some(table_store) => table_store,
            None => {
                self.dbs.insert(db_name.to_string(), TableStore::default());
                self.dbs.get_mut(db_name).unwrap()
            }
        };

        if table_store.tables.contains_key(&table_name) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Table {} already exists", table_name),
            ));
        }

        table_store.tables.insert(table_name, columns);

        Ok(())
    }

    fn alter_table(&mut self, db_name: &str, table_name: String, column_op: ColumnOp) {
        match column_op {
            ColumnOp::Add(column_name) => self.add_table_column(db_name, table_name, column_name),
            ColumnOp::Rename(old_column, new_column) => {
                self.rename_table_column(db_name, table_name, old_column, new_column)
            }
            ColumnOp::Drop(column_name) => self.drop_table_column(db_name, table_name, column_name),
        }
    }

    fn drop_tables(&mut self, db_name: &str, table_names: Vec<String>) {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            for table_name in &table_names {
                table_store.tables.remove(table_name);
            }

            if table_store.tables.is_empty() {
                self.dbs.remove(db_name);
            }
        }
    }

    fn add_table_column(&mut self, db_name: &str, table_name: String, column: String) {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            if let Some(columns) = table_store.tables.get_mut(&table_name) {
                columns.push(column);
            }
        }
    }

    fn rename_table_column(
        &mut self,
        db_name: &str,
        table_name: String,
        old_column: String,
        new_column: String,
    ) {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            if let Some(columns) = table_store.tables.get_mut(&table_name) {
                for column in columns.iter_mut() {
                    if *column == old_column {
                        *column = new_column.clone();
                    }
                }
            }
        }
    }

    fn drop_table_column(&mut self, db_name: &str, table_name: String, column: String) {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            if let Some(columns) = table_store.tables.get_mut(&table_name) {
                columns.retain(|x| *x != column);
            }
        }
    }

    fn get_columns(&mut self, db_name: &str, table_name: &str) -> Result<Vec<String>, Error> {
        if let Some(table_store) = self.dbs.get(db_name) {
            if let Some(cols) = table_store.tables.get(table_name) {
                return Ok(cols.clone());
            }
        };

        Err(Error::new(
            ErrorKind::InvalidData,
            format!("cannot find columns for table {}::{}", db_name, table_name),
        ))
    }
}

fn save_to_file(path: &Path, db_store: &DbStore) -> Result<(), Error> {
    let serialized = serde_json::to_string(&db_store).unwrap();
    debug!("Writing Store: {}", serialized);
    async_std::task::block_on(async { fs::write(&path, serialized).await })
}

fn load_from_file(path: &Path) -> Result<Option<DbStore>, Error> {
    let path = async_std::path::PathBuf::from(path);
    async_std::task::block_on(async {
        if !path.exists().await {
            let parent = path.parent().unwrap();
            fs::create_dir_all(&parent).await?;
            fs::File::create(&path).await?;
            println!("Read Store {}: ", path.to_str().unwrap_or(""));
            Ok(None)
        } else {
            let serialized = fs::read_to_string(&path).await?;
            println!("Read Store {}: {}", path.to_str().unwrap_or(""), serialized);
            Ok(serde_json::from_str::<DbStore>(&serialized).ok())
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_update_store() {
        let mut db_store = DbStore::default();

        // db: create pet values(c1, c2, c3) => ok
        let op = TableOp::CreateTable(
            "pet".to_owned(),
            vec!["c1".to_owned(), "c2".to_owned(), "c3".to_owned()],
        );
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result =
            "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db: create pet values(c1) => error (table already exists)
        let op = TableOp::CreateTable("pet".to_owned(), vec!["c1".to_owned()]);
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        assert!(result.is_err());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db: create pet2 values(c1) => ok
        let op = TableOp::CreateTable("pet2".to_owned(), vec!["c1".to_owned()]);
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\"], \"pet2\": [\"c1\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db2: create pet2 values(c1) => ok
        let op = TableOp::CreateTable("pet2".to_owned(), vec!["c1".to_owned()]);
        let result = db_store.update_store(&"db2".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\"], \"pet2\": [\"c1\"]} }, \"db2\": TableStore { tables: {\"pet2\": [\"c1\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db2: drop pet2  => ok
        let op = TableOp::DropTable(vec!["pet2".to_owned()]);
        let result = db_store.update_store(&"db2".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\"], \"pet2\": [\"c1\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db: drop pet2  => ok
        let op = TableOp::DropTable(vec!["pet2".to_owned()]);
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result =
            "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db: alter pet (add column c4)  => ok
        let op = TableOp::AlterTable("pet".to_owned(), ColumnOp::Add("c4".to_owned()));
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result =
            "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\", \"c4\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db: alter pet (rename column c3 to c55)  => ok
        let op = TableOp::AlterTable(
            "pet".to_owned(),
            ColumnOp::Rename("c3".to_owned(), "c55".to_owned()),
        );
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result =
            "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c55\", \"c4\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db: alter pet (drop column c4)  => ok
        let op = TableOp::AlterTable("pet".to_owned(), ColumnOp::Drop("c4".to_owned()));
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result =
            "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c55\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);

        // db: alter pet (drop column c2)  => ok
        let op = TableOp::AlterTable("pet".to_owned(), ColumnOp::Drop("c2".to_owned()));
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c55\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);
    }
}
