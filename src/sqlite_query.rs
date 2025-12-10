use sqlite_wasm_rs as ffi;
use liquid::{Object};
use liquid::model::Value as LiquidValue;

use std::ffi::CString;
use std::ptr;

unsafe fn run_query(db: *mut ffi::sqlite3, sql: &str) -> Vec<Object> {
    let mut stmt: *mut ffi::sqlite3_stmt = ptr::null_mut();

    let csql = CString::new(sql).unwrap();
    ffi::sqlite3_prepare_v2(db, csql.as_ptr(), -1, &mut stmt, ptr::null_mut());

    let mut rows = Vec::new();

    loop {
        let rc = ffi::sqlite3_step(stmt);
        if rc == ffi::SQLITE_ROW {
            let mut row = Object::new();
            let col_count = ffi::sqlite3_column_count(stmt);

            for i in 0..col_count {
                let name_ptr = ffi::sqlite3_column_name(stmt, i);
                let name = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().to_string();

                let col_type = ffi::sqlite3_column_type(stmt, i);

                let value = match col_type {
                    ffi::SQLITE_INTEGER =>
                    LiquidValue::scalar(ffi::sqlite3_column_int64(stmt, i)),
                    ffi::SQLITE_FLOAT =>
                    LiquidValue::scalar(ffi::sqlite3_column_double(stmt, i)),
                    ffi::SQLITE_TEXT => {
                        let text_ptr = ffi::sqlite3_column_text(stmt, i);
                        let text = std::ffi::CStr::from_ptr(text_ptr as *const i8)
                        .to_string_lossy()
                        .to_string();

                        LiquidValue::scalar(text)
                    },
                    ffi::SQLITE_NULL => LiquidValue::Nil,
                    _ => LiquidValue::Nil
                };

                row.insert(name.into(), value);
            }

            rows.push(row);
        } else {
            break;
        }
    }

    ffi::sqlite3_finalize(stmt);

    rows
}

pub async fn get_query_data(
    db_vec: &[u8],
    queries: Vec<(String, String)>
) -> Object {
    unsafe {
        let mut db: *mut ffi::sqlite3 = ptr::null_mut();
        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE | ffi::SQLITE_OPEN_MEMORY;

        let ret = ffi::sqlite3_open_v2(
            c"memdb".as_ptr().cast(),
                                       &mut db,
                                       flags,
                                       ptr::null(),
        );
        assert_eq!(ret, ffi::SQLITE_OK, "Failed to open SQLite");

        // ✅ Directly deserialize the database
        let ret = ffi::sqlite3_deserialize(
            db,
            b"main\0".as_ptr() as *const _,
                                           db_vec.as_ptr() as *mut u8,
                                           db_vec.len() as i64,
                                           db_vec.len() as i64,
                                           ffi::SQLITE_DESERIALIZE_READONLY,
        );
        assert_eq!(ret, ffi::SQLITE_OK, "Failed to deserialize DB");

        // Run queries
        let mut out = Object::new();
        for (name, sql) in queries {
            let rows = run_query(db, &sql);
            out.insert(name.into(), LiquidValue::Array(rows.into_iter().map(LiquidValue::Object).collect()));
        }

        ffi::sqlite3_close(db);
        out
    }
}
