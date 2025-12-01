use wasm_bindgen::prelude::*;
use web_sys::window;
use liquid::ParserBuilder;
use liquid::object;
use wasm_bindgen_futures::spawn_local;

use serde_json::Value as JsonValue;
use liquid::model::Value;
use liquid::Object;
use liquid::model::Value as LiquidValue;
use std::ffi::CStr;
use serde_json::{json, Value as ValueJ};
use wasm_bindgen::JsValue;
use include_json::include_json;

use opfs::persistent::{DirectoryHandle, FileHandle, WritableFileStream, app_specific_dir};
use opfs::{GetFileHandleOptions, CreateWritableOptions};
use opfs::persistent;

// you must import the traits to call methods on the types
use opfs::{DirectoryHandle as _, FileHandle as _, WritableFileStream as _};

// Include the template as a string at compile time
const TEMPLATE_OVERVIEW: &str = include_str!("../templates/overview.liquid");
const QUERY_OVERVIEW_YEAR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(concat!(include_str!("../queries/overview_year.sql"), "\0").as_bytes()) };
const QUERY_OVERVIEW_COUNTRY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(concat!(include_str!("../queries/overview_country.sql"), "\0").as_bytes()) };

//const TRANSLATION: &str = include_str!("../languages/swedish.json");

//const DB_BYTES: &[u8] = include_bytes!("chronik_8.db");




#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {

    // Get db from OPFS
    spawn_local(async {

        // translation
        let dir = app_specific_dir().await.unwrap();
        let mut translation: Option<String> = None;

        match dir
        .get_file_handle_with_options(
            "translation.json",
            &GetFileHandleOptions { create: false },
        )
        .await
        {
            Ok(file) => {
                // Read bytes from file
                let bytes = file.read().await.unwrap();

                // Convert bytes to String
                translation = Some(String::from_utf8(bytes).unwrap());

                // Optional debug
                // web_sys::console::log_1(&format!("Loaded translation.json, {} bytes", bytes.len()).into());
            }
            Err(err) => {
                web_sys::console::log_1(
                    &format!("Translation file (translation.json) not found or could not be opened from OPFS: {:?}", err).into(),
                );
            }
        }


       let mut parameters = Object::new();


        let dir = app_specific_dir().await.unwrap();
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
                web_sys::console::log_1(&format!("Database file (chronik.db) not found or could not be opened from OPFS: {:?}", err).into());
            }
        }





        let page = get_page();


        match page.as_str() {
            "" | "overview" => {

                let json_object = json_to_liquid_from_option(&translation);
                parameters.insert("translation".into(), json_object);

                // TRIP DOMAINS
                let json_table = open_and_read_db(&db_vec, c"SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';").await;
                let json_object = json_to_object(&json_table);
                let array_value = json_object.into_iter().next().unwrap().1;
                parameters.insert("tripDomains".into(), array_value);

                // PARTICIPANT GROUPS
                let json_table = open_and_read_db(&db_vec, c"SELECT * FROM bewx_ParticipantGroups;").await;
                let json_object = json_to_object(&json_table);
                let array_value = json_object.into_iter().next().unwrap().1;
                parameters.insert("participantGroups".into(), array_value);

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
                setup_filter_listener("tripDomain");
                setup_filter_listener("participantGroup");

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

fn json_to_liquid_from_option(opt: &Option<String>) -> LiquidValue {
    match opt {
        None => LiquidValue::Nil,
        Some(s) => {
            // Try parsing the string as JSON
            match serde_json::from_str::<JsonValue>(s) {
                Ok(json) => json_to_liquid(&json),
                Err(_) => LiquidValue::Scalar(s.clone().into()), // fallback to string
            }
        }
    }
}

fn json_to_liquid(value: &JsonValue) -> LiquidValue {
    match value {
        JsonValue::Null => LiquidValue::Nil,
        JsonValue::Bool(b) => LiquidValue::Scalar((*b).into()),
        JsonValue::Number(n) => LiquidValue::Scalar(n.to_string().into()),
        JsonValue::String(s) => LiquidValue::Scalar(s.clone().into()),
        JsonValue::Array(arr) => {
            LiquidValue::Array(arr.iter().map(|v| json_to_liquid(v)).collect())
        }
        JsonValue::Object(obj) => {
            let mut liquid_obj = Object::new();
            for (k, v) in obj {
                liquid_obj.insert(k.clone().into(), json_to_liquid(v));
            }
            LiquidValue::Object(liquid_obj)
        }
    }
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

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlSelectElement, Url};

fn setup_filter_listener(select_id: &str) {
    let window = web_sys::window().expect("window missing");
    let document: Document = window.document().expect("document missing");

    let elem = match document.get_element_by_id(select_id) {
        Some(e) => e,
        None => {
            web_sys::console::log_1(
                &format!("ERROR: missing select element '{}'", select_id).into(),
            );
            return;
        }
    };

    let select: HtmlSelectElement = match elem.dyn_into() {
        Ok(s) => s,
        Err(e) => {
            web_sys::console::log_1(
                &format!("ERROR: element '{}' is not a <select>: {:?}", select_id, e).into(),
            );
            return;
        }
    };

    let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
        update_filter_param();
    });

    if let Err(e) =
        select.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
        {
            web_sys::console::log_1(
                &format!(
                    "ERROR: add_event_listener failed on '{}' => {:?}",
                    select_id, e
                )
                .into(),
            );
            return;
        }

        closure.forget();
}

fn update_filter_param() {
    let window = web_sys::window().expect("window missing");
    let document = window.document().expect("document missing");

    let mut parts: Vec<String> = Vec::new();
    let selects = vec!["tripDomain", "participantGroup"];

    for id in selects {
        match document.get_element_by_id(id) {
            None => {
                web_sys::console::log_1(
                    &format!("WARN: select '{}' not found", id).into(),
                );
            }
            Some(elem) => {
                let select: HtmlSelectElement = match elem.dyn_into() {
                    Ok(s) => s,
                    Err(e) => {
                        web_sys::console::log_1(
                            &format!("ERROR: element '{}' is not <select>: {:?}", id, e).into(),
                        );
                        continue;
                    }
                };

                let selected = extract_selected(&select);
                if !selected.is_empty() {
                    parts.push(format!("{}:{};", id, selected.join(",")));
                }
            }
        }
    }

    let href = match window.location().href() {
        Ok(h) => h,
        Err(e) => {
            web_sys::console::log_1(&format!("ERROR: window.location.href => {:?}", e).into());
            return;
        }
    };

    let url = match Url::new(&href) {
        Ok(u) => u,
        Err(e) => {
            web_sys::console::log_1(
                &format!("ERROR: Url::new failed for href '{}': {:?}", href, e).into(),
            );
            return;
        }
    };

    let search_params = url.search_params();

    if parts.is_empty() {
        search_params.delete("filter");
    } else {
        let filter_value = format!("{{{}}}", parts.join(""));
        search_params.set("filter", &filter_value);
    }

    if let Err(e) = window
        .history()
        .expect("history")
        .replace_state_with_url(&JsValue::NULL, "", Some(&url.href()))
        {
            web_sys::console::log_1(
                &format!("ERROR: replace_state_with_url failed => {:?}", e).into(),
            );
        }
}

fn extract_selected(select: &HtmlSelectElement) -> Vec<String> {
    let selected_options = select.selected_options();
    let mut out = Vec::new();

    for i in 0..selected_options.length() {
        if let Some(option) = selected_options.item(i) {
            if let Some(v) = option.get_attribute("value") {
                out.push(v);
            }
        }
    }
    out
}
