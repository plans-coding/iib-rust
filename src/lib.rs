use wasm_bindgen::prelude::*;
use web_sys::window;
use liquid::ParserBuilder;
use liquid::object;
use wasm_bindgen_futures::spawn_local;

// Include the template as a string at compile time
const OVERVIEW: &str = include_str!("../templates/overview.liquid");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {

    let page = get_page();


    match page.as_str() {
        "" | "overview" => {

            /*let template = ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse(OVERVIEW)
            .unwrap();

            let globals = object!({ "num": 4f64, "jek": "56" });
            let output = template.render(&globals).unwrap();

            let document = window().unwrap().document().unwrap();
            let app_div = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("#app not found"))?;

            app_div.set_inner_html(&output);*/

            //let test = { "num": 4f64, "jek": "56" };
            //let globals = object!(&test);
            //render_to_dom(OVERVIEW, &globals, "app");
            let test = r#"{ "num": 4, "jek": "576" }"#;
            let globals = json_to_object(test);
            render_to_dom(OVERVIEW, &globals, "app");

        }
        "map" => {

        }
        "statistics" => {

        }
        _ => {
            //content.set_inner_html("<p>Page not found</p>");
        }
    }






    Ok(())
}

pub fn render_to_dom(template_content: &str, json_data: &liquid::Object, element_id: &str) -> Result<(), JsValue> {
    let template = ParserBuilder::with_stdlib()
    .build()
    .unwrap()
    .parse(template_content)
    .unwrap();

    let output = template.render(&json_data).unwrap();

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

use serde_json::Value as JsonValue;
use liquid::model::Value;
use liquid::Object;


fn json_to_object(json_str: &str) -> Object {
    let parsed: JsonValue = serde_json::from_str(json_str).unwrap();
    let mut obj = Object::new();
    if let JsonValue::Object(map) = parsed {
        for (k, v) in map {
            let val = match v {
                JsonValue::Number(n) => Value::scalar(n.as_f64().unwrap()),
                JsonValue::String(s) => Value::scalar(s),
                _ => continue,
            };
            obj.insert(k.into(), val);
        }
    }
    obj
}

/*
#[wasm_bindgen(start)]
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        open_db().await;
    });
}

use sqlite_wasm_rs::{
    self as ffi
};

unsafe extern "C" fn callback(
    _data: *mut std::ffi::c_void,
    argc: i32,
    argv: *mut *mut i8,
    col_names: *mut *mut i8,
) -> i32 {
    for i in 0..argc {
        let val = std::ffi::CStr::from_ptr(*argv.add(i as usize))
        .to_string_lossy()
        .into_owned();

        let col = std::ffi::CStr::from_ptr(*col_names.add(i as usize))
        .to_string_lossy()
        .into_owned();

        web_sys::console::log_1(
            &format!("{} = {}", col, val).into()
        );
    }
    0
}


async fn open_db() {

    let mut db = std::ptr::null_mut();
    let ret = unsafe {
        ffi::sqlite3_open_v2(
            c"chronik.db".as_ptr().cast(),
                             &mut db as *mut _,
                             ffi::SQLITE_OPEN_READONLY,
                             std::ptr::null()
        )
    };
    if ret == ffi::SQLITE_OK {
        web_sys::console::log_1(&"sqlite3_exec succeeded".into());
    } else {
        web_sys::console::log_1(&format!("sqlite3_exec ERROR: {}", ret).into());
    }

    // SQL statement
    let sql = c"PRAGMA database_list;";

    // Execute SQL
    let ret = unsafe {
        ffi::sqlite3_exec(
            db,
            sql.as_ptr().cast(),
                          Some(callback),
                          std::ptr::null_mut(),
                          std::ptr::null_mut(), // ignore error string
        )};
        if ret == ffi::SQLITE_OK {
            web_sys::console::log_1(&"sqlite3_exec succeeded".into());
        } else {
            web_sys::console::log_1(&format!("sqlite3_exec ERROR: {}", ret).into());
        }

}*/
