use web_sys::{window, Url};
use serde_json::{Value};
use wasm_bindgen::JsValue;

pub fn get_query_params() -> Value {
    let window = window().expect("no global `window` exists");
    let location = window.location();
    let search = location.search().unwrap_or_default(); // e.g. "?p=xxx&s=yyy&f=(aaa=xxx,zzz;bbb=yyy,qqq)"

    let mut result = serde_json::Map::new();

    if !search.is_empty() {
        let query = &search[1..]; // remove '?'

        for pair in query.split('&') {
            let mut iter = pair.splitn(2, '=');
            let Some(key) = iter.next() else { continue };
            if key.is_empty() { continue; }

            let value = iter.next().unwrap_or("");

            if key == "f" {
                // Expected format: f=(aaa=xxx,zzz;bbb=yyy,qqq)
                let obj = if value.starts_with('(') && value.ends_with(')') {
                    let inner = &value[1..value.len()-1]; // strip "()"
                    let mut fmap = serde_json::Map::new();

                    // Split by semicolon between keys
                    for kv_block in inner.split(';') {
                        if kv_block.trim().is_empty() { continue; }

                        let mut kv_iter = kv_block.splitn(2, '=');
                        let Some(k) = kv_iter.next() else { continue };
                        let Some(vlist) = kv_iter.next() else { continue };

                        // Values separated by commas
                        let vals: Vec<Value> = vlist
                        .split(',')
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| Value::String(s.to_string()))
                        .collect();

                        fmap.insert(k.to_string(), Value::Array(vals));
                    }

                    Value::Object(fmap)
                } else {
                    Value::String(value.to_string())
                };

                result.insert("f".to_string(), obj);

            } else {
                result.insert(key.to_string(), Value::String(value.to_string()));
            }
        }
    }

    Value::Object(result)
}


pub fn set_query_params(params: &Value) {
    let window = window().expect("no global `window` exists");
    let location = window.location();
    let href = location.href().unwrap_or_default();

    let url = Url::new(&href).expect("Invalid URL");
    url.set_search("");

    let mut search_parts = vec![];

    if let Value::Object(map) = params {
        for (key, value) in map {
            if key == "f" {
                // Expected internal JSON format:
                // f: { aaa: ["xxx","zzz"], bbb: ["yyy","qqq"] }
                if let Value::Object(fmap) = value {
                    let mut encoded = vec![];

                    for (k, v) in fmap {
                        if let Value::Array(arr) = v {
                            let vs = arr
                            .iter()
                            .filter_map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join(",");

                            encoded.push(format!("{}={}", k, vs));
                        }
                    }

                    let inner = encoded.join(";"); // key blocks separated by ";"
                    search_parts.push(format!("f=({})", inner));

                } else if let Value::String(s) = value {
                    search_parts.push(format!("f={}", s));
                }

            } else if let Value::String(s) = value {
                search_parts.push(format!("{}={}", key, s));
            }
        }
    }

    let search = search_parts.join("&");
    url.set_search(&search);

    window.history().unwrap().replace_state_with_url(
        &JsValue::NULL,
        "",
        Some(&url.href()),
    ).unwrap();
}
