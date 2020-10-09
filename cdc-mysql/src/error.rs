use std::io::Error as IoError;
use thiserror::Error;

use fluvio::FluvioError;
use serde_json::Error as JsonError;
use mysql_binlog::errors::BinlogParseError;
use mysql::Error as MysqlError;

#[derive(Error, Debug)]
pub enum CdcError {
    #[error(transparent)]
    IoError {
        #[from]
        source: IoError,
    },
    #[error("Fluvio client error")]
    Fluvio {
        #[from]
        source: FluvioError,
    },
    #[error("Json error")]
    Json {
        #[from]
        source: JsonError,
    },
    #[error("Mysql Binlog error")]
    Binlog {
        #[from]
        source: BinlogParseError,
    },
    #[error("Mysql client error")]
    MySql {
        #[from]
        source: MysqlError,
    },
    #[error("Resume file error")]
    ResumeError {
        source: IoError,
    },
    #[error("Binlog file error")]
    BinlogFileError {
        source: IoError,
    },
    #[error("CDC config error")]
    ConfigError {
        source: IoError,
    }
}
