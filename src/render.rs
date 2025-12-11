use liquid::{ParserBuilder, Object};
use web_sys::window;
use kstring::KString;

pub fn render2dom(
    template_content: &str,
    liquid_object: &Object,
    element_id: &str,
) {
    // 1. Create a Liquid parser
    let parser = ParserBuilder::with_stdlib().build().unwrap();

    // 2. Parse the template
    let template = match parser.parse(template_content) {
        Ok(t) => t,
        Err(err) => {
            web_sys::console::error_1(&format!("Failed to parse template: {}", err).into());
            return;
        }
    };

    // 3. Render template with provided object
    let rendered = match template.render(liquid_object) {
        Ok(s) => s,
        Err(err) => {
            web_sys::console::error_1(&format!("Failed to render template: {}", err).into());
            return;
        }
    };

    // 4. Get the document and target element
    let document = window().unwrap().document().unwrap();
    if let Some(element) = document.get_element_by_id(element_id) {
        element.set_inner_html(&rendered);
    } else {
        web_sys::console::error_1(
            &format!("Element with id '{}' not found", element_id).into()
        );
    }
}

pub fn json_to_liquid_object(v: &serde_json::Value) -> liquid::model::Object {
    match v {
        serde_json::Value::Object(obj) => obj.iter()
        .map(|(k, v)| (KString::from(k.clone()), json_value_to_liquid(v)))
        .collect(),
        _ => liquid::model::Object::new(), // if not an object, return empty
    }
}

pub fn json_value_to_liquid(v: &serde_json::Value) -> liquid::model::Value {
    match v {
        serde_json::Value::Null => liquid::model::Value::Nil,
        serde_json::Value::Bool(b) => liquid::model::Value::Scalar((*b).into()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                liquid::model::Value::Scalar(i.into())
            } else if let Some(f) = n.as_f64() {
                liquid::model::Value::Scalar(f.into())
            } else {
                liquid::model::Value::Nil
            }
        }
        serde_json::Value::String(s) => liquid::model::Value::Scalar(s.clone().into()),
        serde_json::Value::Array(arr) => liquid::model::Value::Array(
            arr.iter().map(json_value_to_liquid).collect()
        ),
        serde_json::Value::Object(obj) => {
            let map: liquid::model::Object = obj.iter()
            .map(|(k, v)| (KString::from(k.clone()), json_value_to_liquid(v)))
            .collect();
            liquid::model::Value::Object(map)
        }
    }
}
