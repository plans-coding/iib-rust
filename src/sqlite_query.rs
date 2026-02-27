use crate::DB_BYTES;
use sqlite_wasm_rs as ffi;
use once_cell::sync::OnceCell;
use serde_json::{Map, Value};
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Mutex;

pub struct SqliteDb {
    pub db: *mut ffi::sqlite3,
}

unsafe impl Send for SqliteDb {}

impl Drop for SqliteDb {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_close(self.db); }
    }
}

static SQLITE_DB: OnceCell<Mutex<SqliteDb>> = OnceCell::new();

unsafe fn get_column_value(stmt: *mut ffi::sqlite3_stmt, col: i32) -> Value {
    unsafe {
        match ffi::sqlite3_column_type(stmt, col) {
            ffi::SQLITE_INTEGER => Value::from(ffi::sqlite3_column_int64(stmt, col)),
            ffi::SQLITE_FLOAT => Value::from(ffi::sqlite3_column_double(stmt, col)),
            ffi::SQLITE_TEXT => {
                let ptr = ffi::sqlite3_column_text(stmt, col);
                if ptr.is_null() {
                    Value::Null
                } else {
                    CStr::from_ptr(ptr as *const i8).to_string_lossy().into_owned().into()
                }
            }
            _ => Value::Null,
        }
    }
}

pub fn get_or_init_db(db_vec: &[u8]) -> Option<&'static Mutex<SqliteDb>> {
    if let Some(db) = SQLITE_DB.get() {
        return Some(db);
    }
    if db_vec.is_empty() {
        return None;
    }

    Some(SQLITE_DB.get_or_init(|| unsafe {
        let mut db = ptr::null_mut();
        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE | ffi::SQLITE_OPEN_MEMORY;
        
        ffi::sqlite3_open_v2(b"memdb\0".as_ptr().cast(), &mut db, flags, ptr::null());
        ffi::sqlite3_deserialize(
            db, b"main\0".as_ptr().cast(), db_vec.as_ptr() as *mut u8,
            db_vec.len() as i64, db_vec.len() as i64, ffi::SQLITE_DESERIALIZE_READONLY,
        );
        
        Mutex::new(SqliteDb { db })
    }))
}

pub async fn get_query_data_universal(db_vec: &[u8], queries: Vec<(String, String)>, preserve_order: bool) -> Value {
    let mut out = Map::new();
    let Some(db_lock) = get_or_init_db(db_vec) else { return Value::Object(out) };
    let db_guard = db_lock.lock().expect("Lock poisoned");

    for (name, sql) in queries {
        let mut results = Map::new();
        let mut rows = Vec::new();
        let mut columns = Vec::new();

        unsafe {
            let c_sql = CString::new(sql).unwrap_or_default();
            let mut stmt = ptr::null_mut();
            
            if ffi::sqlite3_prepare_v2(db_guard.db, c_sql.as_ptr(), -1, &mut stmt, ptr::null_mut()) == ffi::SQLITE_OK {
                let col_count = ffi::sqlite3_column_count(stmt);
                
                for i in 0..col_count {
                    let name_ptr = ffi::sqlite3_column_name(stmt, i);
                    let col_name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
                    columns.push(col_name);
                }

                while ffi::sqlite3_step(stmt) == ffi::SQLITE_ROW {
                    if preserve_order {
                        let row_vals: Vec<Value> = (0..col_count).map(|i| get_column_value(stmt, i)).collect();
                        rows.push(Value::Array(row_vals));
                    } else {
                        let mut row_map = Map::new();
                        for i in 0..col_count {
                            row_map.insert(columns[i as usize].clone(), get_column_value(stmt, i));
                        }
                        rows.push(Value::Object(row_map));
                    }
                }
                ffi::sqlite3_finalize(stmt);
            }
        }

        if preserve_order {
            results.insert("columns".to_string(), Value::Array(columns.into_iter().map(Value::String).collect()));
            results.insert("rows".to_string(), Value::Array(rows));
            out.insert(name, Value::Object(results));
        } else {
            out.insert(name, Value::Array(rows));
        }
    }
    Value::Object(out)
}
pub async fn user_run_sql_internal(sql: String) {

    let Some(db_bytes) = DB_BYTES.get() else {
        web_sys::console::error_1(&"DB_BYTES not initialized".into());
        return;
    };

    // Call the new universal function with 'true' to get columns + rows
    let query_response = get_query_data_universal(
        db_bytes, 
        vec![("user_sql".to_string(), sql)], 
        true
    ).await;

    let result = &query_response["user_sql"];
    let json = serde_json::to_string(result).unwrap_or_else(|_| "[]".to_string());

    // Update the DOM
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(element) = document.get_element_by_id("sql_output_data") {
                element.set_text_content(Some(&json));
            } else {
                web_sys::console::error_1(&"Element #sql_output_data not found".into());
            }
        }
    }
}