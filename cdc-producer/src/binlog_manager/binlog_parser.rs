use cdc_messages::{BeforeAfterCols, BinLogMessage, Cols, Operation};
use cdc_messages::{DeleteRows, UpdateRows, WriteRows};
use crossbeam_channel::Sender;
use mysql_binlog::event::TypeCode;
use mysql_binlog::{parse_file, BinlogEvent};
use serde_json;
use std::error::Error as StdError;
use std::io::{Error, ErrorKind};

use crate::cli::Filters;
use crate::store::DbStore;

pub fn parse_records_from_file(
    sender: &Sender<String>,
    log_file: &String,
    file_name: &String,
    offset: Option<u64>,
    filters: &Option<Filters>,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<Option<u64>, Error> {
    _parse_log_file(sender, log_file, file_name, offset, filters, db_store, urn)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Parser error: {}", e)))
}

fn _parse_log_file(
    sender: &Sender<String>,
    log_file: &String,
    file_name: &String,
    offset: Option<u64>,
    filters: &Option<Filters>,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<Option<u64>, Box<dyn StdError>> {
    let mut latest_offset = None;

    for event in parse_file(&log_file, offset)? {
        if let Ok(event) = event {
            latest_offset = Some(event.offset);

            // print error and continue
            if let Err(err) =
                process_event(sender, file_name, event, offset, filters, db_store, urn)
            {
                println!("{:?}", err);
            }
        }
    }

    Ok(latest_offset)
}

fn process_event(
    sender: &Sender<String>,
    file_name: &String,
    event: BinlogEvent,
    offset: Option<u64>,
    filters: &Option<Filters>,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<(), Error> {
    if !allowed_by_filters(filters, &event.schema, &event.schema_name) {
        return Ok(());
    }
    if same_offset(offset, event.offset) {
        return Ok(());
    }

    let msg = event_to_message(event, file_name, db_store, urn)?;
    if msg.len() > 0 {
        sender.send(msg).expect("Send message error");
    }

    Ok(())
}

fn event_to_message(
    event: BinlogEvent,
    file_name: &String,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<String, Error> {
    match event.type_code {
        TypeCode::QueryEvent => process_query_event(event, file_name, db_store, urn),
        TypeCode::WriteRowsEventV2 => process_write_rows_event(event, file_name, db_store, urn),
        TypeCode::UpdateRowsEventV2 => process_update_rows_event(event, file_name, db_store, urn),
        TypeCode::DeleteRowsEventV2 => process_delete_rows_event(event, file_name, db_store, urn),
        _ => Err(to_err(format!(
            "Warning: Event '{:?}' skipped (evt2msg)",
            event.type_code
        ))),
    }
}

fn process_query_event(
    event: BinlogEvent,
    file_name: &String,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<String, Error> {
    if event.schema.is_none() {
        return Err(to_err(format!(
            "Error: '{:?}' missing 'schema' field.",
            event.type_code
        )));
    }
    let schema = event.schema.as_ref().unwrap();

    if let Some(table) = parse_table_name(&event.query) {
        // clear columns (to be regenerated on next table row update)
        db_store.clear_columns(&schema, &table);
    }

    if skip_query_event(&event.query) {
        return Ok("".to_owned());
    }

    // generate message
    let offset = Some(event.offset);

    let query = event.query.as_ref().unwrap_or(&"".to_owned()).clone();
    let op = Operation::Query(query);

    let msg = BinLogMessage::new(urn, schema, &None, file_name, offset, None, op);

    // encode
    let encoded = serde_json::to_string_pretty(&msg).unwrap();

    Ok(encoded)
}

fn process_write_rows_event(
    event: BinlogEvent,
    file_name: &String,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<String, Error> {
    let (schema, table) = get_schema_table(&event)?;
    let columns = db_store.get_columns(&schema, &table)?;

    // generate message
    let offset = Some(event.offset);

    let rows_json_str = serde_json::to_string(&event.rows)?;
    let rows: Vec<Cols> = serde_json::from_str(&rows_json_str)?;
    let op = Operation::Add(WriteRows { rows });

    let msg = BinLogMessage::new(
        urn,
        &schema,
        &Some(table),
        file_name,
        offset,
        Some(columns),
        op,
    );

    // encode
    let encoded = serde_json::to_string_pretty(&msg).unwrap();

    Ok(encoded)
}

fn process_update_rows_event(
    event: BinlogEvent,
    file_name: &String,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<String, Error> {
    let (schema, table) = get_schema_table(&event)?;
    let columns = db_store.get_columns(&schema, &table)?;

    // generate message
    let offset = Some(event.offset);

    let rows_json_str = serde_json::to_string(&event.rows)?;
    let rows: Vec<BeforeAfterCols> = serde_json::from_str(&rows_json_str)?;
    let op = Operation::Update(UpdateRows { rows });

    let msg = BinLogMessage::new(
        urn,
        &schema,
        &Some(table),
        file_name,
        offset,
        Some(columns),
        op,
    );

    // encode
    let encoded = serde_json::to_string_pretty(&msg).unwrap();

    Ok(encoded)
}

fn process_delete_rows_event(
    event: BinlogEvent,
    file_name: &String,
    db_store: &mut DbStore,
    urn: &String,
) -> Result<String, Error> {
    let (schema, table) = get_schema_table(&event)?;
    let columns = db_store.get_columns(&schema, &table)?;

    // generate message
    let offset = Some(event.offset);

    let rows_json_str = serde_json::to_string(&event.rows)?;
    let rows: Vec<Cols> = serde_json::from_str(&rows_json_str)?;
    let op = Operation::Delete(DeleteRows { rows });

    let msg = BinLogMessage::new(
        urn,
        &schema,
        &Some(table),
        file_name,
        offset,
        Some(columns),
        op,
    );

    // encode
    let encoded = serde_json::to_string_pretty(&msg).unwrap();

    Ok(encoded)
}

/// Allowed by filter algorithm applies to schema or schema_name.
///  - no schema or schema_name => true
///  - no filters => true
///  - include filters matched => true
///  - exclude filters matched => false
fn allowed_by_filters(
    filters: &Option<Filters>,
    schema: &Option<String>,
    schema_name: &Option<String>,
) -> bool {
    // set db name
    let db_name = if let Some(schema) = schema {
        schema
    } else if let Some(schema) = schema_name {
        schema
    } else {
        return true;
    };

    if let Some(filters) = filters {
        match filters {
            Filters::Include { include_dbs: dbs } => {
                if dbs.contains(db_name) {
                    true
                } else {
                    false
                }
            }
            Filters::Exclude { exclude_dbs: dbs } => {
                if dbs.contains(db_name) {
                    false
                } else {
                    true
                }
            }
        }
    } else {
        true
    }
}

fn same_offset(local_offset: Option<u64>, event_offset: u64) -> bool {
    if let Some(local_offset) = local_offset {
        if local_offset == event_offset {
            return true;
        }
    }
    false
}

fn parse_table_name(query: &Option<String>) -> Option<String> {
    if let Some(query) = query {
        let mut table_found = false;
        let words = query.split_whitespace();

        for word in words {
            if table_found {
                return Some(word.replace(&[',', '\"', '`', '\''][..], ""));
            }
            if word.to_lowercase() == "table" {
                table_found = true;
            }
        }
    }
    None
}

fn skip_query_event(query: &Option<String>) -> bool {
    if let Some(query) = query {
        return match query.to_lowercase().as_str() {
            "begin" => true,
            _ => false,
        };
    }
    true
}

fn to_err(err_msg: String) -> Error {
    Error::new(ErrorKind::Other, err_msg)
}

fn get_schema_table(event: &BinlogEvent) -> Result<(String, String), Error> {
    if let Some(schema) = &event.schema_name {
        if let Some(table) = &event.table_name {
            return Ok((schema.clone(), table.clone()));
        }
    }

    Err(to_err(format!(
        "Error: '{:?}' missing table or schema",
        event.type_code
    )))
}

#[cfg(test)]
mod test {
    use super::parse_table_name;

    #[test]
    fn test_parse_table_name() {
        // test - none
        let result = parse_table_name(&None);
        assert_eq!(result, None);

        // test - "BEGIN"
        let query = Some("BEGIN".to_owned());
        let result = parse_table_name(&query);
        assert_eq!(result, None);

        // test - "create database flvTest"
        let query = Some("create database flvTest".to_owned());
        let result = parse_table_name(&query);
        assert_eq!(result, None);

        // test - "alter table people add col1 int"
        let query = Some("alter table people add col1 int".to_owned());
        let result = parse_table_name(&query);
        assert_eq!(result, Some("people".to_owned()));

        // test - "CREATE TABLE species (name VARCHAR(20), type VARCHAR(20),  age SMALLINT)"
        let query = Some(
            "CREATE TABLE species (name VARCHAR(20), type VARCHAR(20),  age SMALLINT)".to_owned(),
        );
        let result = parse_table_name(&query);
        assert_eq!(result, Some("species".to_owned()));

        // test - "CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE)"
        let query = Some(
            "CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE)".to_owned(),
        );
        let result = parse_table_name(&query);
        assert_eq!(result, Some("pet".to_owned()));

        // test -  "DROP TABLE `species` /* generated by server */"
        let query = Some("DROP TABLE `species` /* generated by server */".to_owned());
        let result = parse_table_name(&query);
        assert_eq!(result, Some("species".to_owned()));
    }
}
