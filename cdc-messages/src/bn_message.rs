use http::uri::Parts;
use http::Uri;
use serde::{Deserialize, Serialize};

use crate::{BnFile, Operation};

#[derive(Serialize, Deserialize, Debug)]
pub struct BinLogMessage {
    pub uri: String,
    pub bn_file: BnFile,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,

    pub operation: Operation,
}

impl BinLogMessage {
    pub fn new(
        mysql_resource_name: &String,
        db_name: &String,
        table_name: &Option<String>,
        file_name: &String,
        offset: Option<u64>,
        columns: Option<Vec<String>>,
        operation: Operation,
    ) -> Self {
        Self {
            uri: make_uri(mysql_resource_name, db_name, table_name),
            bn_file: BnFile {
                file_name: file_name.clone(),
                offset: offset.clone(),
            },
            columns,
            operation,
        }
    }
}

fn make_uri(mysql_resource_name: &String, db_name: &String, table_name: &Option<String>) -> String {
    let mut link = "/".to_string();

    let mut parts = Parts::default();
    parts.scheme = Some("flv".parse().unwrap());
    parts.authority = Some(mysql_resource_name.parse().unwrap());

    link.push_str(db_name);
    if let Some(table_name) = table_name {
        link.push_str(&"/");
        link.push_str(table_name);
    }
    parts.path_and_query = Some(link.parse().unwrap());

    if let Ok(uri) = Uri::from_parts(parts) {
        return uri.to_string();
    };

    return "".to_owned();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_uri() {
        // test without table
        let mysql_resource_name = "mysql.local".to_owned();
        let db_name = "myDB".to_owned();
        let table_name = None;
        let uri = make_uri(&mysql_resource_name, &db_name, &table_name);
        let expected_uri = "flv://mysql.local/myDB";

        assert_eq!(uri, expected_uri);

        // test with table
        let db_name = "myDB".to_owned();
        let table_name = Some("myTable".to_owned());
        let uri = make_uri(&mysql_resource_name, &db_name, &table_name);
        let expected_uri = "flv://mysql.local/myDB/myTable";
        assert_eq!(uri, expected_uri);
    }
}
