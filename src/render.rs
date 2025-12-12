use web_sys::window;
use tera::{Value, Context, Tera};


pub fn render2dom(
    template_content: &str,
    json_object: &Value,
    element_id: &str,
) -> String {
    // Convert the JSON object into Tera Context
    let context = Context::from_serialize(json_object)
    .expect("Failed to create Tera context from JSON");

    let rendered = match Tera::one_off(template_content, &context, true) {
        Ok(s) => s,
        Err(e) => {
            web_sys::console::error_1(&format!("Tera render error: {:?}", e).into());
            return String::new();
        }
    };

    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.get_element_by_id(element_id).unwrap();
    element.set_inner_html(&rendered);

    rendered
}
