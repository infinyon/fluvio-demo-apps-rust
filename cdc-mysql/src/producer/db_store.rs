use std::collections::BTreeMap;
use std::io::{Error, ErrorKind};

use super::binlog_manager::{TableOp, ColumnOp};

type DbName = String;
type TableName = String;
type Column = String;
#[derive(Debug)]
pub struct DbStore {
    dbs: BTreeMap<DbName, TableStore>,
}

#[derive(Debug)]
pub struct TableStore {
    tables: BTreeMap<TableName, Vec<Column>>,
}

impl TableStore {
    pub fn new() -> Self {
        Self {
            tables: BTreeMap::new(),
        }
    }
}

impl Default for TableStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DbStore {
    pub fn new() -> Self {
        Self {
            dbs: BTreeMap::new(),
        }
    }

    pub fn update_store(&mut self, db_name: &String, table_ops: Vec<TableOp>) -> Result<(), Error> {
        for table_op in table_ops {
            match table_op {
                TableOp::CreateTable(table_name, columns) => {
                    self.create_table(db_name, table_name, columns)?;
                }
                TableOp::AlterTable(table_name, column_op) => {
                    self.alter_table(db_name, table_name, column_op)?;
                }
                TableOp::DropTable(table_names) => {
                    self.drop_tables(db_name, table_names)?;
                }
            }
        }
        Ok(())
    }

    pub fn create_table(&mut self, db_name: &String, table_name: String, columns: Vec<String>) -> Result<(), Error> {
        let table_store = match self.dbs.get_mut(db_name) {
            Some(table_store) => table_store,
            None => {
                self.dbs.insert(db_name.to_string(), TableStore::new());
                self.dbs.get_mut(db_name).unwrap()
            }
        };

        if table_store.tables.contains_key(&table_name) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Table {} already exists", table_name)
            ))
        }

        table_store
            .tables
            .insert(table_name.to_string(), columns.to_owned());

        Ok(())
    }

    pub fn alter_table(&mut self, db_name: &String, table_name: String, column_op: ColumnOp) -> Result<(), Error> {
        match column_op {
            ColumnOp::Add(column_name) => self.add_table_column(db_name, table_name, column_name),
            ColumnOp::Rename(old_column, new_column)=> self.rename_table_column(db_name, table_name, old_column, new_column),
            ColumnOp::Drop(column_name) => self.drop_table_column(db_name, table_name, column_name),
        }
    }

    pub fn drop_tables(&mut self, db_name: &String, table_names: Vec<String>) -> Result<(), Error> {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            for table_name in &table_names {
                table_store.tables.remove(table_name);
            }

            if table_store.tables.is_empty() {
                self.dbs.remove(db_name);
            }
        }

        Ok(())
    }

    pub fn add_table_column(&mut self, db_name: &String, table_name: String, column: String) -> Result<(), Error> {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            if let Some(columns) = table_store.tables.get_mut(&table_name) {
                columns.push(column);
            }
        }
        Ok(())
    }

    pub fn rename_table_column(&mut self, db_name: &String, table_name: String, old_column: String, new_column: String) -> Result<(), Error> {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            if let Some(columns) = table_store.tables.get_mut(&table_name) {
                for column in columns.iter_mut() {
                    if *column == old_column {
                        *column = new_column.clone();
                    }
                }
            }
        }        
        Ok(())
    }

    pub fn drop_table_column(&mut self, db_name: &String, table_name: String, column: String) -> Result<(), Error> {
        if let Some(table_store) = self.dbs.get_mut(db_name) {
            if let Some(columns) = table_store.tables.get_mut(&table_name) {
                columns.retain(|x| *x != column);
            }
        }        
        Ok(())
    }

    pub fn get_columns(
        &mut self,
        db_name: &str,
        table_name: &str,
    ) -> Result<Vec<String>, Error> {
        if let Some(table_store) = self.dbs.get(db_name) {
            if let Some(cols) = table_store.tables.get(table_name) {
                return Ok(cols.clone());
            }
        };

        Err(Error::new(
            ErrorKind::InvalidData,
            format!("cannot find columns for table {}::{}", db_name, table_name)
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_update_store() {
        let mut db_store = DbStore::new();

        // db: create pet values(c1, c2, c3) => ok
        let op = TableOp::CreateTable("pet".to_owned(), vec!["c1".to_owned(), "c2".to_owned(), "c3".to_owned()]);
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\"]} }}";
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
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result); 

        // db: alter pet (add column c4)  => ok
        let op = TableOp::AlterTable("pet".to_owned(), ColumnOp::Add("c4".to_owned()));
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c3\", \"c4\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);     
        
        // db: alter pet (rename column c3 to c55)  => ok
        let op = TableOp::AlterTable("pet".to_owned(), ColumnOp::Rename("c3".to_owned(), "c55".to_owned()));
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c55\", \"c4\"]} }}";
        assert!(result.is_ok());
        assert_eq!(format!("{:?}", db_store.dbs), expected_result);   
        
        // db: alter pet (drop column c4)  => ok
        let op = TableOp::AlterTable("pet".to_owned(), ColumnOp::Drop("c4".to_owned()));
        let result = db_store.update_store(&"db".to_owned(), vec![op]);
        let expected_result = "{\"db\": TableStore { tables: {\"pet\": [\"c1\", \"c2\", \"c55\"]} }}";
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