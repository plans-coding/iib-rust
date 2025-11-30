use wasm_bindgen::prelude::*;
use web_sys::window;
use liquid::ParserBuilder;
use liquid::object;
use wasm_bindgen_futures::spawn_local;

use serde_json::Value as JsonValue;
use liquid::model::Value;
use liquid::Object;

use std::ffi::CStr;
use serde_json::{json, Value as ValueJ};
use wasm_bindgen::JsValue;

use opfs::persistent::{DirectoryHandle, FileHandle, WritableFileStream, app_specific_dir};
use opfs::{GetFileHandleOptions, CreateWritableOptions};
use opfs::persistent;

// you must import the traits to call methods on the types
use opfs::{DirectoryHandle as _, FileHandle as _, WritableFileStream as _};

// Include the template as a string at compile time
const TEMPLATE_OVERVIEW: &str = include_str!("../templates/overview.liquid");
const QUERY_OVERVIEW_YEAR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(concat!(include_str!("../queries/overview_year.sql"), "\0").as_bytes()) };
const QUERY_OVERVIEW_COUNTRY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(concat!(include_str!("../queries/overview_country.sql"), "\0").as_bytes()) };

//const DB_BYTES: &[u8] = include_bytes!("chronik_8.db");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {

    // Get db from OPFS
    spawn_local(async {

        let dir = app_specific_dir().await.unwrap();
        let mut parameters = Object::new();

        //let mut db_vec: Option<Vec<u8>> = None;
        //let mut db_bytes: Option<&[u8]> = None;

         let mut db_vec: Vec<u8> = Vec::new();


        match dir.get_file_handle_with_options(
            "chronik.db",
            &GetFileHandleOptions { create: false },
        ).await {
            Ok(file) => {

                db_vec = file.read().await.unwrap();

                //web_sys::console::log_1(&format!("Loaded chronik.db, {} bytes", db_bytes.len()).into());

            }
            Err(err) => {
                web_sys::console::log_1(&format!("File not found or could not be opened: {:?}", err).into());
            }
        }





        let page = get_page();


        match page.as_str() {
            "" | "overview" => {

                parameters = Object::new();

                // OVERVIEW YEAR
                let json_table = open_and_read_db(&db_vec, QUERY_OVERVIEW_YEAR).await;
                let json_object = json_to_object(&json_table);
                let array_value = json_object.into_iter().next().unwrap().1;
                parameters.insert("overviewYear".into(), array_value);

                // OVERVIEW COUNTRY
                let json_table = open_and_read_db(&db_vec, QUERY_OVERVIEW_COUNTRY).await;
                let json_object = json_to_object(&json_table);
                let array_value = json_object.into_iter().next().unwrap().1;
                parameters.insert("overviewCountry".into(), array_value);



                //web_sys::console::log_1(&JsValue::from_str("---"));
                web_sys::console::log_1(&JsValue::from_str(&serde_json::to_string(&parameters).unwrap()));

                render_to_dom(TEMPLATE_OVERVIEW, &parameters, "app").unwrap();

            }
            "map" => {

            }
            "statistics" => {

            }
            _ => {
                //content.set_inner_html("<p>Page not found</p>");
            }
        }




    });







    Ok(())
}

pub fn render_to_dom(template_content: &str, json_data: &liquid::Object, element_id: &str) -> Result<(), JsValue> {
    let template = ParserBuilder::with_stdlib()
    .build()
    .unwrap()
    .parse(template_content)
    .unwrap();

//    let output = template.render(&json_data).unwrap();

    let output = template
    .render(&json_data)
    .map_err(|e| {
        let msg = format!("Template render error: {}", e);
        web_sys::console::error_1(&msg.clone().into());
        JsValue::from_str(&msg)
    })?;



    let document = window().unwrap().document().unwrap();
    let app_div = document
    .get_element_by_id(element_id)
    .ok_or_else(|| JsValue::from_str("#app not found"))?;

    app_div.set_inner_html(&output);

    Ok(())
}

// Get UrlSearchParams
pub fn get_page() -> String {
    let w = window().unwrap();
    let loc = w.location();
    let search = loc.search().unwrap_or_default();

    let params = web_sys::UrlSearchParams::new_with_str(&search)
    .unwrap_or_else(|_| web_sys::UrlSearchParams::new().unwrap());

    params.get("p").unwrap_or_default()
}

fn json_to_object(json_str: &Option<String>) -> Object {
    let mut obj = Object::new();

    // If Option is None, return empty Object
    let json_str = match json_str {
        Some(s) => s, // s is &String
        None => return obj,
    };

    // Parse JSON from &String
    let parsed: JsonValue = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(_) => return obj, // Return empty object on parse error
    };

    match parsed {
        JsonValue::Array(arr) => {
            let mut rows_vec: Vec<Value> = Vec::new();
            for row in arr {
                if let JsonValue::Object(map) = row {
                    let mut row_obj = Object::new();
                    for (k, v) in map {
                        let val = match v {
                            JsonValue::Number(n) => Value::scalar(n.as_f64().unwrap_or(0.0)),
                            JsonValue::String(s) => Value::scalar(s),
                            JsonValue::Bool(b) => Value::scalar(b),
                            _ => continue,
                        };
                        row_obj.insert(k.into(), val);
                    }
                    rows_vec.push(Value::Object(row_obj));
                }
            }
            obj.insert("rows".into(), Value::Array(rows_vec));
        }
        JsonValue::Object(map) => {
            for (k, v) in map {
                let val = match v {
                    JsonValue::Number(n) => Value::scalar(n.as_f64().unwrap_or(0.0)),
                    JsonValue::String(s) => Value::scalar(s),
                    JsonValue::Bool(b) => Value::scalar(b),
                    _ => continue,
                };
                obj.insert(k.into(), val);
            }
        }
        _ => {}
    }

    obj
}




use sqlite_wasm_rs::{
    self as ffi
};

// Callback
unsafe extern "C" fn callback(
    data: *mut std::ffi::c_void,
    argc: i32,
    argv: *mut *mut i8,
    col_names: *mut *mut i8,
) -> i32 {
    let rows: &mut Vec<ValueJ> = &mut *(data as *mut Vec<ValueJ>);

    let mut row = serde_json::Map::new();
    for i in 0..argc {
        let val = CStr::from_ptr(*argv.add(i as usize))
        .to_string_lossy()
        .into_owned();
        let col = CStr::from_ptr(*col_names.add(i as usize))
        .to_string_lossy()
        .into_owned();
        row.insert(col, ValueJ::String(val));
    }

    rows.push(ValueJ::Object(row));
    0
}

// OPEN DB FROM BINARY
pub async fn open_and_read_db(DB_BYTES: &[u8], SQL_QUERY: &CStr) -> Option<String> {

    // Suppose you have this Vec somewhere
    let mut rows: Vec<ValueJ> = Vec::new();

    // Pass a pointer to rows as _data when calling sqlite3_exec
    let rows_ptr: *mut std::ffi::c_void = &mut rows as *mut _ as *mut _;



    unsafe {


        let mut db: *mut ffi::sqlite3 = std::ptr::null_mut();

        // Open an EMPTY ephemeral DB
        let ret = ffi::sqlite3_open_v2(
            c":memory:".as_ptr(),
                                       &mut db,
                                       ffi::SQLITE_OPEN_READWRITE
                                       | ffi::SQLITE_OPEN_CREATE
                                       | ffi::SQLITE_OPEN_MEMORY,
                                       std::ptr::null(),
        );

        if ret != ffi::SQLITE_OK {
            web_sys::console::log_1(&format!("open failed: {}", ret).into());
            return None;
        }

        // Allocate SQLite-owned memory for the DB image
        let size = DB_BYTES.len() as i64;
        let buf = ffi::sqlite3_malloc64(size as u64) as *mut u8;
        std::ptr::copy_nonoverlapping(DB_BYTES.as_ptr(), buf, size as usize);

        // Now deserialize the DB into the opened connection
        let ret = ffi::sqlite3_deserialize(
            db,
            c"main".as_ptr() as *const i8,
                                           buf,
                                           size,
                                           size,
                                           ffi::SQLITE_DESERIALIZE_READONLY, // or 0 if you want writeable
        );

        if ret != ffi::SQLITE_OK {
            web_sys::console::log_1(&format!("deserialize failed: {}", ret).into());
            return None;
        }

        let sql: &CStr = &SQL_QUERY;
        //let sql = c"PRAGMA database_list;";
        //let sql = c"SELECT name FROM sqlite_master WHERE type='table';";

        let ret = ffi::sqlite3_exec(
            db,
            sql.as_ptr(),
                                    Some(callback),
                                    rows_ptr, //std::ptr::null_mut(),
                                    std::ptr::null_mut(),
        );

        if ret == ffi::SQLITE_OK {
            web_sys::console::log_1(&"sqlite3_exec succeeded".into());
            // After sqlite3_exec, convert the full table to JSON
            let json_table = serde_json::to_string(&rows).unwrap();
            //web_sys::console::log_1(&JsValue::from_str(&json_table));
            return Some(json_table);
        } else {
            web_sys::console::log_1(&format!("sqlite3_exec ERROR: {}", ret).into());
            return None;
        }
    }


}
