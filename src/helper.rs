use chrono::Local;
use serde_json::{json, Value, Map};
use crate::filecontent;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast; // needed for `unchecked_into`
use wasm_bindgen::closure::Closure; // needed for event listener closures
use web_sys::{HtmlSelectElement, HtmlOptionElement, window, console};
use wasm_bindgen_futures::spawn_local;

use js_sys::Array;

pub fn attach_select_listener() {
    let document = window().unwrap().document().unwrap();

    // List of select IDs
    let ids = ["TripDomain", "ParticipantGroup"];

    for id in ids.iter() {
        let element = document
            .get_element_by_id(id)
            .unwrap()
            .unchecked_into::<HtmlSelectElement>();

        // Create closure for change event
        let closure = Closure::wrap(Box::new(move || {
            let document = window().unwrap().document().unwrap();

            let get_selected = |id: &str| -> Vec<String> {
                let select: HtmlSelectElement = document
                    .get_element_by_id(id)
                    .unwrap()
                    .unchecked_into();

                (0..select.length())
                    .filter_map(|i| {
                        let option: HtmlOptionElement = select.item(i).unwrap().unchecked_into();
                        if option.selected() { Some(option.value()) } else { None }
                    })
                    .collect()
            };

            let json_value = json!({
                "TripDomain": get_selected("TripDomain"),
                "ParticipantGroup": get_selected("ParticipantGroup")
            });

            // Convert JSON to string for logging
            let json_str = serde_json::to_string(&json_value).unwrap();
            
            // PRINT ALL CHANGES IN FILTERS
            //console::log_1(&wasm_bindgen::JsValue::from_str(&json_str));
            
            let value = json_str.clone();
            spawn_local(async move {
                match filecontent::save_filter2opfs(&value).await {
                    Ok(()) => web_sys::console::log_1(&"filter.json saved".into()),
                    Err(e) => web_sys::console::log_1(&format!("save failed: {e}").into()),
                }
            });
            
            
        }) as Box<dyn Fn()>);

        element
            .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
            .unwrap();

        // Prevent closure from being dropped
        closure.forget();
    }
}

pub fn apply_filter_from_opfs_to_selects() {
    spawn_local(async move {
        let bytes = match filecontent::load_filter_from_opfs().await {
            Some(b) if !b.is_empty() => b,
            _ => return, // no file (or empty)
        };

        let json_str = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                web_sys::console::log_1(&format!("filter.json is not valid UTF-8: {e:?}").into());
                return;
            }
        };

        let v: Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                web_sys::console::log_1(&format!("Failed to parse filter.json: {e:?}").into());
                return;
            }
        };

        let document = window().unwrap().document().unwrap();

        // Helper: set selected options in a <select> by values
        let set_selected = |select_id: &str, selected_values: &[String]| {
            let select: HtmlSelectElement = document
                .get_element_by_id(select_id)
                .unwrap()
                .unchecked_into();

            // Clear current selection
            for i in 0..select.length() {
                let opt: HtmlOptionElement = select.item(i).unwrap().unchecked_into();
                opt.set_selected(false);
            }

            // Apply selection
            for i in 0..select.length() {
                let opt: HtmlOptionElement = select.item(i).unwrap().unchecked_into();
                if selected_values.iter().any(|s| s == &opt.value()) {
                    opt.set_selected(true);
                }
            }

            // Optional: fire "change" so any dependent UI updates run
            let evt = web_sys::Event::new("change").unwrap();
            let _ = select.dispatch_event(&evt);
        };

        // Pull arrays out of JSON
        let trip_domain: Vec<String> = v
            .get("TripDomain")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|it| it.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let participant_group: Vec<String> = v
            .get("ParticipantGroup")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|it| it.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        set_selected("TripDomain", &trip_domain);
        set_selected("ParticipantGroup", &participant_group);

        web_sys::console::log_1(&"Applied filter.json to selects".into());
    });
}

pub fn build_time_json() -> Value {
    let now = Local::now();
    json!({
          "now_year": now.format("%Y").to_string(),
          "now_date": now.format("%Y-%m-%d").to_string(),
    })
}

pub async fn get_latest_version_number() -> String {
    let latest_version_number = filecontent::fetch_text(
        "https://raw.githubusercontent.com/plans-coding/immer-in-bewegung/refs/heads/main/version"
    ).await;

    latest_version_number.expect("No version number found")
}

pub fn transform_settings(settings_array: &Vec<Value>) -> Value {
    let mut result = Map::new();

    for setting in settings_array {
        let attribute = setting["Attribute"].as_str().unwrap();
        let group = setting["AttributeGroup"].as_str().unwrap();
        let value_str = setting["Value"].as_str().unwrap();

        // Parse the value if it's a JSON object string, else keep as string
        let value: Value = if value_str.starts_with('{') || value_str.starts_with('[') {
            serde_json::from_str(value_str).unwrap_or(Value::String(value_str.to_string()))
        } else {
            Value::String(value_str.to_string())
        };

        // Insert into the correct group
        result
            .entry(group)
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .unwrap()
            .insert(attribute.to_string(), value);
    }

    Value::Object(result)
}

pub trait SqlFilterReplace {
    fn replace_filter(self, placeholder: &str, data: &Value) -> String;
}

impl SqlFilterReplace for String {
    fn replace_filter(mut self, placeholder: &str, data: &Value) -> String {
        // Map placeholder → key inside the f-object
        let key = match placeholder {
            "(TripDomain)" => "TripDomain",
            "(ParticipantGroup)" => "ParticipantGroup",
            _ => return self, // ignore unknown placeholders
        };

        // Now data is render_structure["all"]["query_params"]["f"]
        // so simply check data[key]
        let arr = match data.get(key).and_then(|v| v.as_array()) {
            Some(a) if !a.is_empty() => a,
            _ => return self, // missing or empty → leave unchanged
        };

        // Build SQL IN list: ["A","B"] → ('A','B')
        let parts: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| format!("'{}'", s))
        .collect();

        let replacement = format!("({})", parts.join(","));
        self = self.replace(placeholder, &replacement);

        self
    }
}
