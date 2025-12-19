use chrono::Local;
use serde_json::{json, Value, Map};
use crate::filecontent;
use crate::query_params;
use crate::start;

use wasm_bindgen::JsCast;
use web_sys::{window, Document, HtmlSelectElement, HtmlOptionElement, Event};
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::*;

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


pub fn apply_preselected(query_params: &Value) {
    let document = window().unwrap().document().unwrap();
    let filter_keys = ["TripDomain", "ParticipantGroup"];

    for key in filter_keys {
        let Some(select_el) = document.get_element_by_id(key) else { continue };
        let select_el: HtmlSelectElement = select_el.dyn_into().unwrap();

        if let Some(arr) = query_params.get(key).and_then(|v| v.as_array()) {
            let target_values: Vec<String> =
            arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();

            for i in 0..select_el.length() {
                if let Some(option_el) = select_el.item(i) {
                    // Convert Element → HtmlOptionElement
                    let option: web_sys::HtmlOptionElement = option_el
                    .dyn_into()
                    .expect("Expected <option> element");

                    let value = option.value(); // uses .value(), safer than attribute
                    option.set_selected(target_values.contains(&value));
                }
            }
        }
    }
}






pub fn attach_select_listeners() {
    let document: Document = window().unwrap().document().unwrap();
    let filter_keys = ["TripDomain", "ParticipantGroup"];

    for key in filter_keys {
        let Some(element) = document.get_element_by_id(key) else { continue };
        let select_el: HtmlSelectElement = element.dyn_into().unwrap();
        let key_clone = key.to_string();

        let closure = Closure::<dyn FnMut(Event)>::new(move |_| {
            let document = window().unwrap().document().unwrap();

            // 1. Get current query params (top-level object)
            let params = query_params::get_query_params(); // &Value

            let mut full_updated_map = match params.as_object() {
                Some(map) => map.clone(),
                                                       None => serde_json::Map::new(),
            };

            // 2. Read current selections for this <select>
            if let Some(el) = document.get_element_by_id(&key_clone) {
                let select: HtmlSelectElement = el.dyn_into().unwrap();
                let mut selected_values = Vec::new();

                for i in 0..select.length() {
                    if let Some(option_el) = select.item(i) {
                        let option: HtmlOptionElement = option_el.dyn_into().unwrap();
                        if option.selected() {
                            selected_values.push(option.value());
                        }
                    }
                }

                // 3. Update only the key inside "f"
                let mut updated_f = full_updated_map
                .get("f")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();

                updated_f.insert(
                    key_clone.clone(),
                                 Value::Array(selected_values.into_iter().map(Value::String).collect()),
                );

                full_updated_map.insert("f".to_string(), Value::Object(updated_f));

                // 4. Call set_query_params
                query_params::set_query_params(&Value::Object(full_updated_map));

                // optional: rerender
                start();
            }
        });

        select_el.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).unwrap();

        closure.forget();
    }
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
