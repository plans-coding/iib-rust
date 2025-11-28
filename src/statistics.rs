use wasm_bindgen::JsValue;
use web_sys::{window, Element};

pub fn render_statistics() -> Result<Element, JsValue> {
    let document = window().unwrap().document().unwrap();
    let div = document.create_element("div")?;

    div.set_inner_html(r#"
    <h2>Statistics</h2>
    <p>This is the overview page.</p>
    "#);

    Ok(div)
}
