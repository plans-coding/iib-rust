#![cfg(all(target_family = "wasm", target_os = "unknown"))]

mod topbar;
mod overview;
mod map;
mod statistics;
mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Response};

// Init webpage
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let document = window().unwrap().document().unwrap();
    let app = document.get_element_by_id("app").unwrap();
    app.set_inner_html("");

    let container = document.create_element("div")?;
    container.set_attribute("id", "container")?;
    app.append_child(&container)?;

    let top = topbar::render_topbar()?;
    container.append_child(&top)?;

    let page = utils::get_page();

    let content = document.create_element("div")?;
    content.set_attribute("id", "content")?;

    match page.as_str() {
        "" | "overview" => {
            // pass db_bytes into render_overview
            let json_bytes = utils::read_from_opfs("baOverview.json").await?;
            let json_text = String::from_utf8(json_bytes).unwrap();
            content.append_child(&overview::render_overview(json_text)?.into())?;
        }
        "map" => {
            content.append_child(&map::render_map()?.into())?;
        }
        "statistics" => {
            content.append_child(&statistics::render_statistics()?.into())?;
        }
        _ => {
            content.set_inner_html("<p>Page not found</p>");
        }
    }

    container.append_child(&content)?;
    Ok(())
}
