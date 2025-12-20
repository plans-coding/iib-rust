use web_sys::{window, Url};
use serde_json::{Value};
use wasm_bindgen::JsValue;
use urlencoding::decode;

pub fn get_query_params() -> Value {
    let window = window().expect("no global `window` exists");
    let location = window.location();
    let search = location.search().unwrap_or_default();

    let mut result = serde_json::Map::new();

    if !search.is_empty() {
        let query = &search[1..]; // remove '?'

        for pair in query.split('&') {
            let mut iter = pair.splitn(2, '=');

            let Some(raw_key) = iter.next() else { continue };
            if raw_key.is_empty() { continue; }

            let raw_value = iter.next().unwrap_or("");

            // URL-decode key and value
            let key = decode(raw_key).unwrap_or_else(|_| raw_key.into());
            let value = decode(raw_value).unwrap_or_else(|_| raw_value.into());

            if key == "f" {
                let obj = if value.starts_with('(') && value.ends_with(')') {
                    let inner = &value[1..value.len() - 1];
                    let mut fmap = serde_json::Map::new();

                    for kv_block in inner.split(';') {
                        if kv_block.trim().is_empty() {
                            continue;
                        }

                        let mut kv_iter = kv_block.splitn(2, '=');
                        let Some(k) = kv_iter.next() else { continue };
                        let Some(vlist) = kv_iter.next() else { continue };

                        let vals: Vec<Value> = vlist
                            .split(',')
                            .filter(|s| !s.trim().is_empty())
                            .map(|s| Value::String(s.to_string()))
                            .collect();

                        fmap.insert(k.to_string(), Value::Array(vals));
                    }

                    Value::Object(fmap)
                } else {
                    Value::String(value.into_owned())
                };

                result.insert("f".to_string(), obj);
            } else {
                result.insert(key.into_owned(), Value::String(value.into_owned()));
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
