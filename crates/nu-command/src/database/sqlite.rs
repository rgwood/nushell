use std::path::{Path, PathBuf};

use nu_protocol::{CustomValue, ShellError, Span, Value};
use rusqlite::{types::ValueRef, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SQLiteDatabase {
    // TODO: should this be a path or an open connection?
    pub path: PathBuf,
}

impl CustomValue for SQLiteDatabase {
    fn clone_value(&self, span: Span) -> Value {
        let cloned = SQLiteDatabase {
            path: self.path.clone(),
        };

        Value::CustomValue {
            val: Box::new(cloned),
            span,
        }
    }

    fn value_string(&self) -> String {
        self.typetag_name().to_string()
    }

    // TODO: write a test that exercises this
    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {

        eprintln!("to_base_value()...");

        let db = open_sqlite_db(&self.path, span)?;

        match read_sqlite_db(db, span) {
            Ok(data) => Ok(data),
            Err(err) => Err(ShellError::GenericError(
                "Failed to read from SQLite database".into(),
                err.to_string(),
                Some(span),
                None,
                Vec::new(),
            )),
        }

        // let p = &self.path;
        // Ok(Value::string(format!("a connection for {p:?}"), span))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn follow_path_int(&self, _count: usize, span: Span) -> Result<Value, ShellError> {
        eprintln!("Path requested: '{_count}'");
        
        todo!("path int not implemented")
    }

    fn follow_path_string(&self, _column_name: String, span: Span) -> Result<Value, ShellError> {
        eprintln!("Path requested: '{_column_name}'");

        todo!("path string not implemented")
    }

    fn typetag_name(&self) -> &'static str {
        // TODO: I don't really understand how this is used
        "SQLiteConnection"
    }

    fn typetag_deserialize(&self) {
        // TODO: this is what the other custom_value implementations do but is it right?
        unimplemented!("typetag_deserialize")
    }
}

pub fn open_sqlite_db(path: &Path, call_span: Span) -> Result<Connection, nu_protocol::ShellError> {
    let path = path.to_string_lossy().to_string();

    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(err) => Err(ShellError::GenericError(
            "Failed to open SQLite database".into(),
            err.to_string(),
            Some(call_span),
            None,
            Vec::new(),
        )),
    }
}

pub fn open_and_read_sqlite_db(
    path: &Path,
    call_span: Span,
) -> Result<Value, nu_protocol::ShellError> {
    let path = path.to_string_lossy().to_string();

    match Connection::open(path) {
        Ok(conn) => match read_sqlite_db(conn, call_span) {
            Ok(data) => Ok(data),
            Err(err) => Err(ShellError::GenericError(
                "Failed to read from SQLite database".into(),
                err.to_string(),
                Some(call_span),
                None,
                Vec::new(),
            )),
        },
        Err(err) => Err(ShellError::GenericError(
            "Failed to open SQLite database".into(),
            err.to_string(),
            Some(call_span),
            None,
            Vec::new(),
        )),
    }
}

pub fn read_sqlite_db(conn: Connection, call_span: Span) -> Result<Value, rusqlite::Error> {
    let mut table_names: Vec<String> = Vec::new();
    let mut tables: Vec<Value> = Vec::new();

    let mut get_table_names =
        conn.prepare("SELECT name from sqlite_master where type = 'table'")?;
    let rows = get_table_names.query_map([], |row| row.get(0))?;

    for row in rows {
        let table_name: String = row?;
        table_names.push(table_name.clone());

        let mut rows = Vec::new();
        let mut table_stmt = conn.prepare(&format!("select * from [{}]", table_name))?;
        let mut table_rows = table_stmt.query([])?;
        while let Some(table_row) = table_rows.next()? {
            rows.push(convert_sqlite_row_to_nu_value(table_row, call_span))
        }

        let table_record = Value::List {
            vals: rows,
            span: call_span,
        };

        tables.push(table_record);
    }

    Ok(Value::Record {
        cols: table_names,
        vals: tables,
        span: call_span,
    })
}

fn convert_sqlite_row_to_nu_value(row: &Row, span: Span) -> Value {
    let mut vals = Vec::new();
    let colnamestr = row.as_ref().column_names().to_vec();
    let colnames = colnamestr.iter().map(|s| s.to_string()).collect();

    for (i, c) in row.as_ref().column_names().iter().enumerate() {
        let _column = c.to_string();
        let val = convert_sqlite_value_to_nu_value(row.get_ref_unwrap(i), span);
        vals.push(val);
    }

    Value::Record {
        cols: colnames,
        vals,
        span,
    }
}

fn convert_sqlite_value_to_nu_value(value: ValueRef, span: Span) -> Value {
    match value {
        ValueRef::Null => Value::Nothing { span },
        ValueRef::Integer(i) => Value::Int { val: i, span },
        ValueRef::Real(f) => Value::Float { val: f, span },
        ValueRef::Text(buf) => {
            let s = match std::str::from_utf8(buf) {
                Ok(v) => v,
                Err(_) => {
                    return Value::Error {
                        error: ShellError::NonUtf8(span),
                    }
                }
            };
            Value::String {
                val: s.to_string(),
                span,
            }
        }
        ValueRef::Blob(u) => Value::Binary {
            val: u.to_vec(),
            span,
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_read_empty_db() {
        let db = Connection::open_in_memory().unwrap();
        let converted_db = read_sqlite_db(db, Span::test_data()).unwrap();

        let expected = Value::Record {
            cols: vec![],
            vals: vec![],
            span: Span::test_data(),
        };

        assert_eq!(converted_db, expected);
    }

    #[test]
    fn can_read_empty_table() {
        let db = Connection::open_in_memory().unwrap();

        db.execute(
            "CREATE TABLE person (
                    id     INTEGER PRIMARY KEY,
                    name   TEXT NOT NULL,
                    data   BLOB
                    )",
            [],
        )
        .unwrap();
        let converted_db = read_sqlite_db(db, Span::test_data()).unwrap();

        let expected = Value::Record {
            cols: vec!["person".to_string()],
            vals: vec![Value::List {
                vals: vec![],
                span: Span::test_data(),
            }],
            span: Span::test_data(),
        };

        assert_eq!(converted_db, expected);
    }

    #[test]
    fn can_read_null_and_non_null_data() {
        let span = Span::test_data();
        let db = Connection::open_in_memory().unwrap();

        db.execute(
            "CREATE TABLE item (
                    id     INTEGER PRIMARY KEY,
                    name   TEXT
                    )",
            [],
        )
        .unwrap();

        db.execute("INSERT INTO item (id, name) VALUES (123, NULL)", [])
            .unwrap();

        db.execute("INSERT INTO item (id, name) VALUES (456, 'foo bar')", [])
            .unwrap();

        let converted_db = read_sqlite_db(db, span).unwrap();

        let expected = Value::Record {
            cols: vec!["item".to_string()],
            vals: vec![Value::List {
                vals: vec![
                    Value::Record {
                        cols: vec!["id".to_string(), "name".to_string()],
                        vals: vec![Value::Int { val: 123, span }, Value::Nothing { span }],
                        span,
                    },
                    Value::Record {
                        cols: vec!["id".to_string(), "name".to_string()],
                        vals: vec![
                            Value::Int { val: 456, span },
                            Value::String {
                                val: "foo bar".to_string(),
                                span,
                            },
                        ],
                        span,
                    },
                ],
                span,
            }],
            span,
        };

        assert_eq!(converted_db, expected);
    }
}
