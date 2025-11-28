use wasm_bindgen::JsValue;
use web_sys::{window, Element};

pub fn render_topbar() -> Result<Element, JsValue> {
    let document = window().unwrap().document().unwrap();
    let nav = document.create_element("div")?;

    nav.set_attribute("id", "topbar")?;
    nav.set_inner_html(r#"
    <style>
    #topbar {
    display: flex;
    padding: 10px;
    background: #333;
    color: #fff;
    gap: 20px;
}
#topbar a {
color: white;
text-decoration: none;
font-weight: bold;
}
</style>

<a href="?p=overview">Overview</a>
<a href="?p=map">Map</a>
<a href="?p=statistics">Statistics</a>
"#);

    Ok(nav)
}
