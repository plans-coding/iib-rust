use web_sys::{UrlSearchParams};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::window;
use wasm_bindgen::prelude::*;


pub fn get_all_query_params() -> HashMap<String, String> {
    let mut result = HashMap::new();

    // Get window.location.search
    let search = window()
    .and_then(|w| w.location().search().ok())
    .unwrap_or_default();

    // Build UrlSearchParams safely
    let params = UrlSearchParams::new_with_str(&search)
    .unwrap_or_else(|_| UrlSearchParams::new().unwrap());

    // Iterate over all params
    let iter = params.entries();

    // Convert JS iterator → Rust HashMap
    loop {
        let next = js_sys::Reflect::get(&iter, &"next".into())
        .ok()
        .and_then(|n| n.dyn_into::<js_sys::Function>().ok())
        .and_then(|next_fn| next_fn.call0(&iter).ok());

        if next.is_none() {
            break;
        }

        let next_obj = next.unwrap();
        let done = js_sys::Reflect::get(&next_obj, &"done".into())
        .ok()
        .and_then(|d| d.as_bool())
        .unwrap_or(true);

        if done {
            break;
        }

        let value = js_sys::Reflect::get(&next_obj, &"value".into()).unwrap();
        let arr = value.dyn_into::<js_sys::Array>().unwrap();

        let key = arr.get(0).as_string().unwrap_or_default();
        let val = arr.get(1).as_string().unwrap_or_default();

        result.insert(key, val);
    }

    result
}
