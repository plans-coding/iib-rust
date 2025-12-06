use liquid::{ParserBuilder, Object};
use web_sys::window;

pub fn render_to_dom(
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
