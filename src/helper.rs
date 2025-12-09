use liquid::model::Object;
use liquid::model::Value;
use std::collections::HashMap;

pub fn insert_translation(liquid_obj: &mut Object, translation: &Option<serde_json::Value>) {
    let liquid_value = translation
    .as_ref()
    .and_then(|content| {
        // serde_json → liquid object conversion
        liquid::model::to_object(content).ok()
    })
    .map(Value::Object)
    .unwrap_or(Value::Nil);

    liquid_obj.insert("translation".into(), liquid_value);
}

pub fn insert_time(liquid_obj: &mut Object) {
    use kstring::KString;
    use chrono::Local;

    let now = Local::now();

    let time: Object = [
        ("now_year", now.format("%Y").to_string()),
        ("now_date", now.format("%Y-%m-%d").to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (KString::from(k), Value::scalar(v)))
    .collect();

    liquid_obj.insert("time".into(), Value::Object(time));
}

pub fn insert_query_params(liquid_obj: &mut Object, query_params: &HashMap<String, String>) {
    let mut obj = Object::new();

    for (k, v) in query_params {
        obj.insert(k.clone().into(), Value::scalar(v.clone()));
    }

    liquid_obj.insert("queryParams".into(), Value::Object(obj));
}
