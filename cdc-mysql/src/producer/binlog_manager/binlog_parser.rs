use crossbeam_channel::Sender;
use mysql_binlog::event::TypeCode;
use mysql_binlog::{parse_file, BinlogEvent};
use std::io::{Error, ErrorKind};
use tracing::{debug, instrument, trace};

use crate::error::CdcError;
use crate::messages::{BeforeAfterCols, BinLogMessage, Cols, Operation};
use crate::messages::{DeleteRows, UpdateRows, WriteRows};
use crate::producer::Filters;

use super::parse_query;
use super::LocalStore;

#[instrument(skip(sender, log_file, offset, filters, local_store))]
pub fn parse_records_from_file(
    sender: &Sender<String>,
    log_file: &str,
    file_name: &str,
    offset: Option<u64>,
    filters: Option<&Filters>,
    local_store: &mut LocalStore,
    urn: &str,
) -> Result<Option<u64>, CdcError> {
    let mut latest_offset = None;

    for event in parse_file(&log_file, offset)? {
        debug!(?event, "Event from binlog parser:");
        if let Ok(event) = event {
            latest_offset = Some(event.offset);
            process_event(sender, file_name, event, offset, filters, local_store, urn)?;
        }
    }

    Ok(latest_offset)
}

#[instrument(skip(sender, file_name, event, offset, filters, local_store, urn))]
fn process_event(
    sender: &Sender<String>,
    file_name: &str,
    event: BinlogEvent,
    offset: Option<u64>,
    filters: Option<&Filters>,
    local_store: &mut LocalStore,
    urn: &str,
) -> Result<(), CdcError> {
    let allowed = allowed_by_filters(
        filters,
        event.schema.as_deref(),
        event.schema_name.as_deref(),
    );

    if !allowed {
        return Ok(());
    }
    if same_offset(offset, event.offset) {
        return Ok(());
    }

    let msg = event_to_message(event, file_name, local_store, urn)?;
    if !msg.is_empty() {
        debug!("Sending message: {}", &msg);
        sender.send(msg).expect("Send message error");
    }

    Ok(())
}

fn event_to_message(
    event: BinlogEvent,
    file_name: &str,
    local_store: &mut LocalStore,
    urn: &str,
) -> Result<String, CdcError> {
    debug!("{:?}", event);
    match event.type_code {
        TypeCode::QueryEvent => process_query_event(event, file_name, local_store, urn),
        TypeCode::WriteRowsEventV2 => process_write_rows_event(event, file_name, local_store, urn),
        TypeCode::UpdateRowsEventV2 => {
            process_update_rows_event(event, file_name, local_store, urn)
        }
        TypeCode::DeleteRowsEventV2 => {
            process_delete_rows_event(event, file_name, local_store, urn)
        }
        _ => Err(to_err(format!(
            "Warning: Event '{:?}' skipped (evt2msg)",
            event.type_code
        ))
        .into()),
    }
}

fn process_query_event(
    event: BinlogEvent,
    file_name: &str,
    local_store: &mut LocalStore,
    urn: &str,
) -> Result<String, CdcError> {
    if event.schema.is_none() {
        return Err(to_err(format!(
            "Error: '{:?}' missing 'schema' field.",
            event.type_code
        ))
        .into());
    }
    let schema = event.schema.as_ref().unwrap();
    let table_ops = parse_query(&event.query)?;

    local_store.update_store(schema, table_ops)?;

    if skip_query_event(&event.query) {
        return Ok("".to_owned());
    }

    // generate message
    let offset = Some(event.offset);

    let query = event.query.as_ref().unwrap_or(&"".to_owned()).clone();
    let op = Operation::Query(query);

    let msg = BinLogMessage::new(urn, schema, None, file_name, offset, None, op);

    // encode
    let encoded = serde_json::to_string_pretty(&msg).unwrap();

    Ok(encoded)
}

fn process_write_rows_event(
    event: BinlogEvent,
    file_name: &str,
    local_store: &mut LocalStore,
    urn: &str,
) -> Result<String, CdcError> {
    let (schema, table) = get_schema_table(&event)?;
    let columns = local_store.get_columns(&schema, &table)?;

    // generate message
    let offset = Some(event.offset);

    let rows_json_str = serde_json::to_string(&event.rows)?;
    let rows: Vec<Cols> = serde_json::from_str(&rows_json_str)?;
    let op = Operation::Add(WriteRows { rows });

    let msg = BinLogMessage::new(
        urn,
        &schema,
        Some(&table),
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
    file_name: &str,
    local_store: &mut LocalStore,
    urn: &str,
) -> Result<String, CdcError> {
    let (schema, table) = get_schema_table(&event)?;
    let columns = local_store.get_columns(&schema, &table)?;

    // generate message
    let offset = Some(event.offset);

    let rows_json_str = serde_json::to_string(&event.rows)?;
    let rows: Vec<BeforeAfterCols> = serde_json::from_str(&rows_json_str)?;
    let op = Operation::Update(UpdateRows { rows });

    let msg = BinLogMessage::new(
        urn,
        &schema,
        Some(&table),
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
    file_name: &str,
    local_store: &mut LocalStore,
    urn: &str,
) -> Result<String, CdcError> {
    let (schema, table) = get_schema_table(&event)?;
    let columns = local_store.get_columns(&schema, &table)?;

    // generate message
    let offset = Some(event.offset);

    let rows_json_str = serde_json::to_string(&event.rows)?;
    let rows: Vec<Cols> = serde_json::from_str(&rows_json_str)?;
    let op = Operation::Delete(DeleteRows { rows });

    let msg = BinLogMessage::new(
        urn,
        &schema,
        Some(&table),
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
#[instrument(skip(filters, schema, schema_name))]
fn allowed_by_filters(
    filters: Option<&Filters>,
    schema: Option<&str>,
    schema_name: Option<&str>,
) -> bool {
    trace!(?schema, ?schema_name, "Checking filters");
    // set db name
    let db_name = if let Some(schema) = schema {
        schema
    } else if let Some(schema) = schema_name {
        schema
    } else {
        return true;
    };
    let db_name = db_name.to_ascii_lowercase();
    trace!(?db_name, "Checking DB name");

    if let Some(filters) = filters {
        match filters {
            Filters::Include { include_dbs: dbs } => {
                let dbs: Vec<_> = dbs.iter().map(|s| &**s).collect();
                trace!("Checking if {:?} includes {}", &dbs, &db_name);
                dbs.contains(&&*db_name)
            }
            Filters::Exclude { exclude_dbs: dbs } => {
                let dbs: Vec<_> = dbs.iter().map(|s| &**s).collect();
                trace!("Checking if {:?} excludes {}", &dbs, &db_name);
                !dbs.contains(&&*db_name)
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

fn skip_query_event(query: &Option<String>) -> bool {
    if let Some(query) = query {
        return query.to_lowercase().as_str() == "begin";
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
