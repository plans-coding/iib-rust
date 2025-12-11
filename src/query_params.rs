use web_sys::{window, Url};
use serde_json::Value;
use wasm_bindgen::JsValue;

// Single-value params (p, s, ...) become strings and `f` becomes an object of key-value pairs if present

pub fn get_query_params() -> Value {
    let window = window().expect("no global `window` exists");
    let location = window.location();
    let search = location.search().unwrap_or_default(); // "?p=xxx&s=yyy&f=(aaa=xxx,bbb=yyy)"

    let mut result = serde_json::Map::new();

    if !search.is_empty() {
        let query = &search[1..]; // strip leading '?'

        for pair in query.split('&') {
            let mut iter = pair.splitn(2, '=');
            if let Some(key) = iter.next() {
                if key.is_empty() { continue; }
                if let Some(value) = iter.next() {
                    if key == "f" {
                        // Parse f=(aaa=xxx,bbb=yyy)
                        let obj = if value.starts_with('(') && value.ends_with(')') {
                            let inner = &value[1..value.len()-1];
                            let mut map = serde_json::Map::new();
                            for kv in inner.split(',') {
                                let mut kv_iter = kv.splitn(2, '=');
                                if let (Some(k), Some(v)) = (kv_iter.next(), kv_iter.next()) {
                                    map.insert(k.to_string(), Value::String(v.to_string()));
                                }
                            }
                            Value::Object(map)
                        } else {
                            Value::String(value.to_string())
                        };
                        result.insert(key.to_string(), obj);
                    } else {
                        result.insert(key.to_string(), Value::String(value.to_string()));
                    }
                }
            }
        }
    }

    Value::Object(result)
}

// Single-value params become `key=value` and `f` becomes `f=(key=value,...)` if present

pub fn set_query_params(params: &Value) {
    let window = window().expect("no global `window` exists");
    let location = window.location();
    let href = location.href().unwrap_or_default();

    // Parse current URL
    let url = Url::new(&href).expect("Invalid URL");

    // Remove all current search params
    url.set_search("");

    if let Value::Object(map) = params {
        let mut search_parts = vec![];

        for (key, value) in map {
            if key == "f" {
                // Serialize f as f=(aaa=xxx,bbb=yyy)
                if let Value::Object(f_map) = value {
                    let inner = f_map.iter()
                    .map(|(k,v)| format!("{}={}", k, v.as_str().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join(",");
                    search_parts.push(format!("f=({})", inner));
                } else if let Value::String(s) = value {
                    search_parts.push(format!("f={}", s));
                }
            } else if let Value::String(s) = value {
                search_parts.push(format!("{}={}", key, s));
            }
        }

        let search = search_parts.join("&");
        url.set_search(&search);
    }

    // Update URL without reloading
    window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(url.href().as_str())).unwrap();
}
