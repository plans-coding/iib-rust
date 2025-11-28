use wasm_bindgen::JsValue;
use web_sys::{window, Element};
use serde::Deserialize; // <-- important

#[derive(Debug, Deserialize)]
pub struct OverviewRow {
    pub InnerId: Option<String>,
    pub ParticipantGroup: Option<String>,
    pub OuterId: Option<String>,
    pub OverallDestination: Option<String>,
    pub DepartureDate: Option<String>,
    pub ReturnDate: Option<String>,
    pub MapPins: Option<String>,
    pub TripDescription: Option<String>,
    pub StartNode: Option<String>,
    pub EndNode: Option<String>,
    pub PhotoStarttime: Option<String>,
    pub PhotoEndtime: Option<String>,
    pub PhotoAlbums: Option<String>,
    pub CoverPhoto: Option<String>,
    pub DocumentationNote: Option<String>,
}

pub fn render_overview(json_text: String) -> Result<Element, JsValue> {
    let document = window().unwrap().document().unwrap();
    let div = document.create_element("div")?;

    // Parse JSON array into Vec<OverviewRow>
    let rows: Vec<OverviewRow> = serde_json::from_str(&json_text)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Build HTML table rows
    let mut html_rows = String::new();
    for row in rows {
        html_rows.push_str(&format!(
            r#"
            <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            </tr>
            "#,
            row.InnerId.clone().unwrap_or_default(),
            row.OuterId.clone().unwrap_or_default(),
            row.ParticipantGroup.clone().unwrap_or_default(),
            row.OverallDestination.clone().unwrap_or_default(),
            row.DepartureDate.clone().unwrap_or_default(),
            row.ReturnDate.clone().unwrap_or_default(),
        ));
    }

    // Wrap inside a full table
    let html = format!(
        r#"
        <h2>Overview</h2>
        <table border="1" cellspacing="0" cellpadding="6">
        <thead>
        <tr>
        <th>InnerId</th>
        <th>OuterId</th>
        <th>Participant</th>
        <th>Destination</th>
        <th>Departure</th>
        <th>Return</th>
        </tr>
        </thead>
        <tbody>
        {}
        </tbody>
        </table>
        "#,
        html_rows
    );

    div.set_inner_html(&html);
    Ok(div)
}
