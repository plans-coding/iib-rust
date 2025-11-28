//Shorter command to write to DOM
use web_sys::{window, Element};
use wasm_bindgen::JsValue;

pub fn el(tag: &str) -> Result<Element, JsValue> {
    window().unwrap().document().unwrap().create_element(tag)
}

// Get UrlSearchParams
pub fn get_page() -> String {
    let w = window().unwrap();
    let loc = w.location();
    let search = loc.search().unwrap_or_default();

    let params = web_sys::UrlSearchParams::new_with_str(&search)
    .unwrap_or_else(|_| web_sys::UrlSearchParams::new().unwrap());

    params.get("p").unwrap_or_default()
}


use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemDirectoryHandle, FileSystemFileHandle};

pub async fn read_from_opfs(name: &str) -> Result<Vec<u8>, JsValue> {
    let storage = window().unwrap().navigator().storage();
    let root_js = JsFuture::from(storage.get_directory()).await?;
    let root: FileSystemDirectoryHandle = root_js.into();

    let file_handle_js = JsFuture::from(root.get_file_handle(name)).await?;
    let file_handle: FileSystemFileHandle = file_handle_js.into();

    let file_js = JsFuture::from(file_handle.get_file()).await?;
    let file: web_sys::File = file_js.into();

    let buf = JsFuture::from(file.array_buffer()).await?;
    let array = js_sys::Uint8Array::new(&buf);
    Ok(array.to_vec())
}
