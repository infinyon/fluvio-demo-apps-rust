use mysql::prelude::*;
use mysql::Error as MysqlError;
use mysql::{from_value, Conn, OptsBuilder, Row};
use std::io::{Error, ErrorKind};

use crate::cli::Database;

pub fn get_opts(db_params: &Database) -> OptsBuilder {
    OptsBuilder::new()
        .ip_or_hostname(db_params.ip_or_host())
        .tcp_port(db_params.port())
        .user(db_params.user())
        .pass(db_params.password())
}

pub fn get_table_columns(
    db_name: &String,
    table_name: &String,
    db_params: &Database,
) -> Result<Vec<String>, Error> {
    _get_table_columns(db_name, table_name, db_params)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Mysql error: {}", e)))
}

fn _get_table_columns(
    db_name: &String,
    table_name: &String,
    db_params: &Database,
) -> Result<Vec<String>, MysqlError> {
    let opts = get_opts(db_params);
    let mut conn = Conn::new(opts)?;
    let query = format!(
        r"SELECT COLUMN_NAME FROM information_schema.columns
            WHERE table_schema='{}' AND table_name='{}' 
            ORDER BY ORDINAL_POSITION",
        db_name, table_name
    );

    let rows: Vec<Row> = conn.query(query)?;
    let columns = rows
        .into_iter()
        .map(|r| from_value(r.unwrap()[0].clone()))
        .collect();

    Ok(columns)
}
