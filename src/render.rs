use tera::{Value, Context, Tera};

pub fn render2dom(
    template_content: &str,
    json_object: &Value,
    element_id: &str,
) -> Result<String, String> {
    let context = Context::from_serialize(json_object)
        .map_err(|e| format!("Context error: {e:?}"))?;

    let rendered = Tera::one_off(template_content, &context, true)
        .map_err(|e| format!("Tera render error: {e:?}"))?;

    let window = web_sys::window()
        .ok_or("No window available")?;

    let document = window
        .document()
        .ok_or("No document available")?;

    let element = document
        .get_element_by_id(element_id)
        .ok_or(format!("Element #{element_id} not found"))?;

    element.set_inner_html(&rendered);

    Ok(rendered)
}