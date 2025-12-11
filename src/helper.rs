use chrono::Local;
use serde_json::{json, Value};
use crate::filecontent;

pub fn build_time_json() -> Value {
    let now = Local::now();
    json!({
          "now_year": now.format("%Y").to_string(),
          "now_date": now.format("%Y-%m-%d").to_string(),
    })
}

pub async fn get_latest_version_number() -> String {
    let latest_version_number = filecontent::fetch_text(
        "https://raw.githubusercontent.com/plans-coding/immer-in-bewegung/refs/heads/main/version"
    ).await;

    latest_version_number.expect("No version number found")
}
