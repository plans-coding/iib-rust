use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use crate::sqlite_query;

use wasm_bindgen::prelude::*;
use web_sys::console;
use serde_wasm_bindgen::from_value;
use std::collections::HashSet;
use serde_json::Value;

pub async fn sync_cover_photo_list_internal(db_vec: &[u8])-> Result<(), JsValue> {


    // GET CURRENT IMMICH COVER PHOTOS -----------------------------------------------------------------------
    let mut opts = RequestInit::new();
    opts.method("GET");

    let req = Request::new_with_str_and_init(
        "/api/albums/601e664f-d51c-4b4d-be3c-ae75fae726db",
        &opts,
    )?;

    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&req)).await?;
    let resp: Response = resp.dyn_into()?;

    let json = JsFuture::from(resp.json()?).await?;
    //web_sys::console::log_1(&json);


    // GET CURRENT DB COVER PHOTOS -----------------------------------------------------------------------
    let cover_photos_query = vec![
        ("cover_photos".to_string(), "SELECT OuterId, CoverPhoto FROM bewa_Overview WHERE CoverPhoto IS NOT NULL;".to_string())
    ];
    let cover_photos_response = sqlite_query::get_query_data(&db_vec, cover_photos_query).await;
    let cover_photos_json_obj: serde_json::Value = serde_json::to_value(&cover_photos_response).expect("ERROR");
    //web_sys::console::log_1(&serde_json::to_string(&cover_photos_json_obj["cover_photos"]).expect("ERROR").into());

    // FIND DIFFERENCES -----------------------------------------------------------------------
    missing_cover_photos(json, cover_photos_json_obj["cover_photos"].clone()).await;

    // ADD TO IMMICH ALBUM -----------------------------------------------------------------------

    // SAVE MAPPING TO OPFS -----------------------------------------------------------------------

    // REPLACE IMAGES -----------------------------------------------------------------------

    Ok(())

}


async fn missing_cover_photos(
    album_json: JsValue,
    cover_photos_json_obj: serde_json::Value,
) -> Result<(), JsValue> {
    let album: Value = from_value(album_json)?;

    // Safe iterator: no temporary references
    let album_paths: HashSet<&str> = album["assets"]
    .as_array()
    .into_iter() // Option<&Vec<Value>> -> Iterator
    .flat_map(|arr| arr.iter()) // iterate over elements if Some
    .filter_map(|a| a["originalPath"].as_str())
    .collect();

    let missing: Vec<&str> = cover_photos_json_obj
    .as_array()
    .into_iter()
    .flat_map(|arr| arr.iter())
    .filter_map(|c| c["CoverPhoto"].as_str())
    .filter(|p| !album_paths.contains(p))
    .collect();

    web_sys::console::log_1(&format!("{:?}", missing).into());
    Ok(())
}
